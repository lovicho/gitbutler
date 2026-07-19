//! Explain why some changes could not be committed or amended.
//!
//! A commit or amend can partially succeed: GitButler allows several independent
//! branches to be checked out at once, and a change that depends on a commit in
//! another branch cannot be committed on its own. The commit/amend outcome lists
//! the rejected changes and a coarse [`RejectionReason`], but it does not say
//! which commit or branch a change depends on.
//!
//! This module joins the rejected changes with the workspace hunk dependencies
//! (the same data the desktop app uses to render its "commit failed" modal) so
//! the CLI can tell the user precisely which hunk depends on which branch, and
//! suggest stacking the branches to resolve it.

use bstr::{BString, ByteSlice};
use but_core::{DiffSpec, HunkHeader, ref_metadata::StackId, tree::create_tree::RejectionReason};
use but_graph::Workspace;
use but_hunk_dependency::ui::{
    HunkDependencies, HunkLockTarget, hunk_dependencies_for_workspace_changes_by_worktree_dir,
};

use crate::theme::{self, Paint};

/// A single change that could not be committed, enriched with the workspace
/// dependencies that explain why.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RejectedChange {
    /// The worktree-relative path of the rejected change.
    #[serde(serialize_with = "but_serde::bstring_lossy::serialize")]
    pub path: BString,
    /// The coarse reason the change was rejected.
    pub reason: RejectionReason,
    /// The hunks of this change that depend on a commit in the workspace,
    /// together with the commit/branch they depend on.
    ///
    /// Empty when the rejection was not caused by a dependency, or when the
    /// dependency could not be pinpointed to a specific hunk.
    pub dependencies: Vec<HunkDependency>,
    /// When a dependency rejection cannot be pinpointed to a hunk (adjacent
    /// insertions, for example, do not intersect as ranges), the branches
    /// whose workspace commits also touch this path — the likely dependency.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub suspected_branches: Vec<String>,
}

/// A hunk that depends on one or more commits in the workspace.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HunkDependency {
    /// The dependent hunk, in worktree coordinates.
    pub hunk: HunkHeader,
    /// The commits (and their branches, when known) this hunk depends on.
    pub commits: Vec<DependencyCommit>,
}

/// A commit a hunk depends on, resolved to a human-friendly branch name.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyCommit {
    /// The commit the hunk depends on.
    #[serde(serialize_with = "but_serde::object_id::serialize")]
    pub commit_id: gix::ObjectId,
    /// The short name of the branch owning [`Self::commit_id`], if it could be
    /// resolved from the workspace.
    pub branch: Option<String>,
}

/// Explain `rejected_specs` by attaching the workspace dependency information
/// (which commit/branch each rejected hunk depends on) for dependency-related
/// rejections.
///
/// The hunk dependencies are computed lazily: only when at least one rejection
/// looks like a dependency conflict, to avoid the cost on unrelated failures.
/// Failure to compute dependencies is not fatal — the change is still reported,
/// just without the dependency detail — because the commit/amend itself already
/// happened and explaining it must never break the command.
pub fn explain_rejections(
    repo: &gix::Repository,
    ws: &Workspace,
    rejected_specs: &[(RejectionReason, DiffSpec)],
    target_branch: Option<&str>,
) -> Vec<RejectedChange> {
    let needs_dependencies = rejected_specs
        .iter()
        .any(|(reason, _)| is_dependency_reason(*reason));

    let dependencies = if needs_dependencies {
        match hunk_dependencies_for_workspace_changes_by_worktree_dir(repo, ws, None) {
            Ok(dependencies) => Some(dependencies),
            Err(err) => {
                tracing::warn!(
                    ?err,
                    "Failed to compute hunk dependencies for rejected changes"
                );
                None
            }
        }
    } else {
        None
    };

    rejected_specs
        .iter()
        .map(|(reason, spec)| {
            let dependencies = match &dependencies {
                Some(deps) if is_dependency_reason(*reason) => {
                    dependencies_for_spec(ws, deps, spec)
                }
                _ => Vec::new(),
            };
            // The branch being committed to never explains its own rejection,
            // so it is excluded from the suspects.
            let suspected_branches = if dependencies.is_empty() && is_dependency_reason(*reason) {
                branches_touching_path(repo, ws, spec.path.as_bstr())
                    .into_iter()
                    .filter(|branch| Some(branch.as_str()) != target_branch)
                    .collect()
            } else {
                Vec::new()
            };
            RejectedChange {
                path: spec.path.clone(),
                reason: *reason,
                dependencies,
                suspected_branches,
            }
        })
        .collect()
}

/// Write a human-readable explanation of `rejected` to `out`.
///
/// When every dependency points at a single branch and `target_branch` (the
/// branch being committed/amended to) differs from it, suggests stacking the two
/// with an exact `but move` command. Does nothing when there is nothing to
/// report.
pub fn write_rejection_report<W: std::fmt::Write + ?Sized>(
    out: &mut W,
    rejected: &[RejectedChange],
    target_branch: Option<&str>,
) -> std::fmt::Result {
    if rejected.is_empty() {
        return Ok(());
    }
    let t = theme::get();
    let noun = if rejected.len() == 1 {
        "change"
    } else {
        "changes"
    };
    writeln!(
        out,
        "{} {} {} could not be applied:",
        t.attention.paint("Note:"),
        rejected.len(),
        noun,
    )?;
    write_rejection_body(out, rejected, target_branch)
}

/// The per-change lines and the stacking suggestion shared by the
/// partial-rejection report and the hard commit-failure error.
fn write_rejection_body<W: std::fmt::Write + ?Sized>(
    out: &mut W,
    rejected: &[RejectedChange],
    target_branch: Option<&str>,
) -> std::fmt::Result {
    let t = theme::get();
    for change in rejected {
        writeln!(out, "  {}", change.path.to_str_lossy())?;
        if change.dependencies.is_empty() {
            if change.suspected_branches.is_empty() {
                writeln!(out, "    {}", reason_summary(change.reason))?;
            } else {
                let branches = change
                    .suspected_branches
                    .iter()
                    .map(|branch| t.local_branch.paint(branch).to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                writeln!(
                    out,
                    "    conflicts with commits on {branches} (edits touch the same file)",
                )?;
            }
            continue;
        }
        for dependency in &change.dependencies {
            let range = hunk_range_label(&dependency.hunk);
            for commit in &dependency.commits {
                let id = theme::Commit(commit.commit_id);
                match &commit.branch {
                    Some(branch) => writeln!(
                        out,
                        "    {range} depends on {} ({id})",
                        t.local_branch.paint(branch),
                    )?,
                    None => writeln!(out, "    {range} depends on commit {id}")?,
                }
            }
        }
    }

    // If everything depends on a single other branch, mention stacking onto it
    // as an option — it's situational, so frame it as a hint, not a directive.
    if let (Some(target), Some(dependency)) = (target_branch, sole_dependency_branch(rejected))
        && target != dependency
    {
        writeln!(out)?;
        writeln!(
            out,
            "{} to apply these changes, stack {} on top of {} and commit them again — commits already on the branch move with it:",
            t.hint.paint("Hint:"),
            t.local_branch.paint(target),
            t.local_branch.paint(dependency),
        )?;
        writeln!(
            out,
            "  {}",
            t.command_suggestion.paint(format!(
                "but move {} {}",
                shell_quote(target),
                shell_quote(dependency)
            )),
        )?;
    }
    Ok(())
}

/// Quote `name` for the suggested shell command if it contains anything a shell
/// would treat specially. Git permits characters like `;` and `&` in ref names,
/// so an unquoted name could otherwise break the copy-pasteable command.
fn shell_quote(name: &str) -> std::borrow::Cow<'_, str> {
    let safe = !name.is_empty()
        && name.bytes().all(|b| {
            b.is_ascii_alphanumeric()
                || matches!(
                    b,
                    b'-' | b'_' | b'/' | b'.' | b'@' | b'+' | b'=' | b':' | b','
                )
        });
    if safe {
        std::borrow::Cow::Borrowed(name)
    } else {
        std::borrow::Cow::Owned(format!("'{}'", name.replace('\'', r"'\''")))
    }
}

/// The two rejection reasons that indicate a change depends on another branch in
/// the workspace, and are therefore worth resolving to a commit/branch.
fn is_dependency_reason(reason: RejectionReason) -> bool {
    matches!(
        reason,
        RejectionReason::CherryPickMergeConflict | RejectionReason::WorkspaceMergeConflict
    )
}

/// The single branch every dependency points at, or `None` if it wouldn't be a
/// complete fix: some rejected change isn't dependency-related (stacking can't
/// resolve it), a dependency couldn't be resolved to a branch, or dependencies
/// span more than one branch.
fn sole_dependency_branch(rejected: &[RejectedChange]) -> Option<&str> {
    let mut branch: Option<&str> = None;
    for change in rejected {
        // Every rejected change must point at a dependency; otherwise a single
        // stack command wouldn't apply all of them. When no hunk-level lock
        // exists, the path-level suspects stand in.
        if change.dependencies.is_empty() && change.suspected_branches.is_empty() {
            return None;
        }
        let names = change
            .dependencies
            .iter()
            .flat_map(|dependency| &dependency.commits)
            .map(|commit| commit.branch.as_deref())
            .chain(
                change
                    .suspected_branches
                    .iter()
                    .map(|name| Some(name.as_str())),
            );
        for name in names {
            let name = name?;
            match branch {
                None => branch = Some(name),
                Some(existing) if existing != name => return None,
                _ => {}
            }
        }
    }
    branch
}

/// Find the hunks of `dependencies` that belong to `spec`.
///
/// Matching is by path, then by hunk overlap: a rejected spec covering specific
/// hunks only reports the dependent hunks that overlap them, while a whole-file
/// spec (no hunks) reports every dependent hunk for the path. Overlap is used
/// rather than exact equality because the dependency hunks are computed without
/// context lines, so their boundaries rarely match the spec's hunks exactly.
fn dependencies_for_spec(
    ws: &Workspace,
    dependencies: &HunkDependencies,
    spec: &DiffSpec,
) -> Vec<HunkDependency> {
    let spec_path = spec.path.as_bstr();
    let mut result = Vec::new();
    for (dep_path, dep_hunk, locks) in &dependencies.diffs {
        // `dep_path` is a lossy-UTF8 key, so a path with invalid UTF8 simply
        // won't match and falls back to the reason-only summary.
        if dep_path.as_bytes().as_bstr() != spec_path {
            continue;
        }
        if locks.is_empty() {
            continue;
        }
        let hunk = HunkHeader::from(dep_hunk);
        let overlaps = spec.hunk_headers.is_empty()
            || spec
                .hunk_headers
                .iter()
                .any(|spec_hunk| hunks_overlap(spec_hunk, &hunk));
        if !overlaps {
            continue;
        }
        let commits = locks
            .iter()
            .map(|lock| DependencyCommit {
                commit_id: lock.commit_id,
                branch: branch_of_commit(ws, lock.commit_id, stack_of(lock.target)),
            })
            .collect();
        result.push(HunkDependency { hunk, commits });
    }
    result
}

/// The stack a lock points at, if it is identifiable.
fn stack_of(target: HunkLockTarget) -> Option<StackId> {
    match target {
        HunkLockTarget::Stack(id) => Some(id),
        HunkLockTarget::Unidentified => None,
    }
}

/// Resolve the short branch name owning `commit_id`, searching `prefer` first
/// (the stack a lock points at) and then the remaining workspace stacks.
pub fn branch_of_commit(
    ws: &Workspace,
    commit_id: gix::ObjectId,
    prefer: Option<StackId>,
) -> Option<String> {
    let ordered = ws
        .stacks
        .iter()
        .filter(|stack| prefer.is_none() || stack.id == prefer)
        .chain(
            ws.stacks
                .iter()
                .filter(|stack| prefer.is_some() && stack.id != prefer),
        );
    for stack in ordered {
        for segment in &stack.segments {
            if segment.commits.iter().any(|commit| commit.id == commit_id)
                && let Some(ref_name) = segment.ref_name()
            {
                return Some(ref_name.shorten().to_string());
            }
        }
    }
    None
}

/// The short names of all branches whose workspace commits touch `path` —
/// the dependency suspects when hunk-level locks cannot pinpoint one. Callers
/// filter out the branch being committed to. Only runs on failure paths, so
/// the per-commit tree diffs are acceptable.
fn branches_touching_path(
    repo: &gix::Repository,
    ws: &Workspace,
    path: &bstr::BStr,
) -> Vec<String> {
    let mut branches = Vec::new();
    for stack in &ws.stacks {
        for segment in &stack.segments {
            let touches = segment.commits.iter().any(|commit| {
                but_core::diff::tree_changes(repo, commit.parent_ids.first().copied(), commit.id)
                    .map(|changes| changes.iter().any(|change| change.path == path))
                    .unwrap_or(false)
            });
            if touches && let Some(ref_name) = segment.ref_name() {
                let name = ref_name.shorten().to_string();
                if !branches.contains(&name) {
                    branches.push(name);
                }
            }
        }
    }
    branches
}

/// A targeted error for a commit that failed outright (not just rejected
/// specs): when the attempted changes conflict with commits on another
/// workspace branch, name that branch and the stacking recovery instead of
/// letting internal cherry-pick errors surface. Returns `None` when no other
/// branch can be blamed, so the caller keeps the original error.
pub fn commit_failure_error(
    repo: &gix::Repository,
    ws: &Workspace,
    specs: &[DiffSpec],
    target_branch: &str,
) -> Option<anyhow::Error> {
    // Every attempted change is treated as a dependency-shaped rejection and
    // runs through the standard explanation pipeline: hunk-level locks first,
    // path-level suspects otherwise.
    let assumed: Vec<(RejectionReason, DiffSpec)> = specs
        .iter()
        .map(|spec| (RejectionReason::WorkspaceMergeConflict, spec.clone()))
        .collect();
    let rejected = explain_rejections(repo, ws, &assumed, Some(target_branch));
    if rejected
        .iter()
        .all(|change| change.dependencies.is_empty() && change.suspected_branches.is_empty())
    {
        return None;
    }
    let mut message = format!(
        "Cannot commit to '{target_branch}': the selected changes conflict with commits on another branch.\n"
    );
    write_rejection_body(&mut message, &rejected, Some(target_branch)).ok()?;
    Some(anyhow::anyhow!(message.trim_end().to_string()))
}

/// Whether two hunks touch the same lines on the new (worktree) side, which is
/// the coordinate space both the spec hunk and the dependency hunk share.
///
/// Zero-length ranges (pure deletions) are treated as covering a single line so
/// that they still match an overlapping hunk.
fn hunks_overlap(a: &HunkHeader, b: &HunkHeader) -> bool {
    let end_a = a.new_start.saturating_add(a.new_lines.max(1));
    let end_b = b.new_start.saturating_add(b.new_lines.max(1));
    a.new_start < end_b && b.new_start < end_a
}

/// A short, new-side line-range label for a hunk, e.g. `line 5` or `lines 5–9`.
fn hunk_range_label(hunk: &HunkHeader) -> String {
    match hunk.new_lines {
        0 => format!("around line {}", hunk.new_start),
        1 => format!("line {}", hunk.new_start),
        lines => format!("lines {}–{}", hunk.new_start, hunk.new_start + lines - 1),
    }
}

/// A plain-language summary for rejections that are not dependency-related.
fn reason_summary(reason: RejectionReason) -> &'static str {
    match reason {
        RejectionReason::NoEffectiveChanges => "no effective change to commit",
        RejectionReason::CherryPickMergeConflict | RejectionReason::WorkspaceMergeConflict => {
            "depends on changes in another branch"
        }
        RejectionReason::WorkspaceMergeConflictOfUnrelatedFile => {
            "conflicts with another change in the workspace"
        }
        RejectionReason::WorktreeFileMissingForObjectConversion => {
            "the file went missing while committing"
        }
        RejectionReason::FileToLargeOrBinary => "the file is too large or binary",
        RejectionReason::PathNotFoundInBaseTree => "the change was not found in the base tree",
        RejectionReason::UnsupportedDirectoryEntry => "unsupported directory entry",
        RejectionReason::UnsupportedTreeEntry => "unsupported file type",
        RejectionReason::MissingDiffSpecAssociation => {
            "the selected hunks no longer match the worktree"
        }
    }
}
