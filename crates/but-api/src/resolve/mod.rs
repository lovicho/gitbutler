//! Resolve the conflicts of a conflicted commit with an LLM.
//!
//! GitButler keeps conflicts as first-class committed state: a conflicted
//! commit carries its merge inputs as trees. This module re-merges those trees
//! in memory, sends the conflict hunks to the configured LLM as a single-shot,
//! no-tools request, splices the returned per-hunk resolutions back
//! deterministically, and rewrites the commit into a normal one, rebasing its
//! descendants. The workspace never enters edit mode and the exclusive
//! worktree lock is held only for the final apply step — not while the model
//! is thinking. Disagreement with the result is handled by the oplog: the
//! operation records an undo point.

use anyhow::Context as _;
use but_api_macros::but_api;
use but_core::DryRun;
use but_core::sync::RepoExclusive;
use but_llm::{ChatMessage, LLMProvider};
use but_oplog::legacy::{OperationKind, SnapshotDetails};
use serde::Serialize;
use tracing::instrument;

use crate::WorkspaceState;

mod apply;
mod context;
mod prompt;

pub use context::{ConflictHunk, FileConflict, ResolutionRequest};
pub use prompt::{FileResolution, HunkResolution, ResolutionResponse, SYSTEM_PROMPT};

/// How one conflicted file was resolved, for display to the user.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "export-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ResolvedFile {
    /// The repo-relative path of the file.
    pub path: String,
    /// The content that replaced each conflict block, in file order.
    pub hunks: Vec<String>,
    /// The model's explanation of its decision for this file.
    pub reasoning: String,
}

#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(ResolvedFile);

/// The outcome of resolving a conflicted commit with AI.
#[derive(Debug, Clone)]
pub struct AiResolutionResult {
    /// The conflicted commit that was resolved.
    pub commit_id: gix::ObjectId,
    /// The rewritten, no-longer-conflicted commit.
    pub new_commit: gix::ObjectId,
    /// A short model-authored markdown summary of the conflict and resolution.
    pub summary: Option<String>,
    /// The per-file resolutions that were applied.
    pub files: Vec<ResolvedFile>,
    /// Workspace state after the resolution.
    pub workspace: WorkspaceState,
}

/// Resolve all conflicts of the conflicted commit `commit_id` using the LLM
/// configured in the user's git configuration, apply the result, and rebase
/// descendants.
///
/// This acquires shared worktree access while gathering the conflicts, holds
/// no lock during the model call, and acquires exclusive worktree access for
/// the apply step. An oplog snapshot records an undo point on success. When
/// `dry_run` is enabled, the returned workspace previews the resolution
/// without materializing the rebase and no oplog entry is persisted.
///
/// Fails without changing anything if no AI provider is configured, if the
/// commit is not conflicted, if a conflict cannot be resolved automatically
/// (deletions, binaries, oversized files), or if the model response does not
/// validate against the request after one retry.
#[but_api(try_from = crate::resolve::json::AiResolutionResult)]
#[instrument(err(Debug))]
pub fn resolve_commit_conflicts_ai(
    ctx: &mut but_ctx::Context,
    commit_id: gix::ObjectId,
    dry_run: DryRun,
) -> anyhow::Result<AiResolutionResult> {
    // The repository's resolved configuration, so repo-local AI settings
    // (`but config ai --local`) take precedence over global ones.
    let llm = {
        let repo = ctx.repo.get()?;
        let config = repo.config_snapshot();
        LLMProvider::from_git_config(config.plumbing()).context(
            "AI is not configured. Configure an AI provider in the GitButler settings first.",
        )?
    };
    resolve_commit_conflicts_with(ctx, commit_id, dry_run, |request| {
        let model = llm.model_or_default();
        llm.structured_output::<ResolutionResponse>(
            SYSTEM_PROMPT,
            vec![ChatMessage::User(prompt::render_user_message(request))],
            &model,
        )?
        .context("The AI model returned no response")
    })
}

/// Like [`resolve_commit_conflicts_ai()`], but with the model call injected as
/// `resolve`, so tests and other frontends can supply resolutions without
/// network access.
///
/// `resolve` is called once, and once more if it errors (truncated or
/// malformed model output) or if its response fails validation against the
/// request.
pub fn resolve_commit_conflicts_with(
    ctx: &mut but_ctx::Context,
    commit_id: gix::ObjectId,
    dry_run: DryRun,
    resolve: impl Fn(&ResolutionRequest) -> anyhow::Result<ResolutionResponse>,
) -> anyhow::Result<AiResolutionResult> {
    let request = {
        let _guard = ctx.shared_worktree_access();
        let repo = ctx.repo.get()?;
        context::build_request(&repo, commit_id)?
    };

    // The model call happens without any worktree lock; the request is plain
    // data and the apply step below re-reads the workspace under the exclusive
    // lock, so it fails closed if the commit changed in the meantime.
    let attempt = || -> anyhow::Result<_> {
        let response = resolve(&request)?;
        let validated = apply::validate(&request, &response)?;
        Ok((validated, response.summary))
    };
    let (validated, summary) = match attempt() {
        Ok(outcome) => outcome,
        Err(first_failure) => {
            tracing::warn!(
                error = ?first_failure,
                "AI conflict-resolution attempt failed, retrying once"
            );
            attempt()?
        }
    };

    let mut guard = ctx.exclusive_worktree_access();
    // The workspace graph may have been cached before or during the model
    // call; the commit-presence check in apply must run against the state
    // under this exclusive lock, not a stale cache.
    ctx.invalidate_workspace_cache()?;
    finish_with_perm(
        ctx,
        &request,
        validated,
        summary,
        dry_run,
        guard.write_permission(),
    )
}

fn finish_with_perm(
    ctx: &mut but_ctx::Context,
    request: &ResolutionRequest,
    validated: Vec<apply::ValidatedFile>,
    summary: Option<String>,
    dry_run: DryRun,
    perm: &mut RepoExclusive,
) -> anyhow::Result<AiResolutionResult> {
    let maybe_oplog_entry = but_oplog::UnmaterializedOplogSnapshot::from_details_with_perm(
        ctx,
        SnapshotDetails::new(OperationKind::ResolveConflictsAi),
        perm.read_permission(),
        dry_run,
    );

    let res = apply::apply(ctx, request, &validated, dry_run, perm);
    if let Some(snapshot) = maybe_oplog_entry
        && res.is_ok()
    {
        snapshot.commit(ctx, perm).ok();
    }
    let (new_commit, workspace) = res?;

    let files = request
        .files
        .iter()
        .zip(validated)
        .map(|(file, validated)| ResolvedFile {
            path: file.path.clone(),
            hunks: validated.hunks,
            reasoning: validated.reasoning,
        })
        .collect();

    Ok(AiResolutionResult {
        commit_id: request.commit_id,
        new_commit,
        summary,
        files,
        workspace,
    })
}

/// JSON transport types for this module.
pub mod json {
    use serde::Serialize;

    use crate::json::HexHash;

    /// JSON transport type for the outcome of an AI conflict resolution.
    #[derive(Debug, Serialize)]
    #[cfg_attr(feature = "export-schema", derive(schemars::JsonSchema))]
    #[serde(rename_all = "camelCase")]
    pub struct AiResolutionResult {
        /// The conflicted commit that was resolved.
        #[cfg_attr(feature = "export-schema", schemars(with = "String"))]
        pub commit_id: HexHash,
        /// The rewritten, no-longer-conflicted commit.
        #[cfg_attr(feature = "export-schema", schemars(with = "String"))]
        pub new_commit: HexHash,
        /// A short model-authored markdown summary of the conflict and resolution.
        pub summary: Option<String>,
        /// The per-file resolutions that were applied.
        pub files: Vec<super::ResolvedFile>,
        /// Workspace state after the resolution.
        pub workspace: crate::json::WorkspaceState,
    }

    #[cfg(feature = "export-schema")]
    but_schemars::register_sdk_type!(AiResolutionResult);

    impl TryFrom<super::AiResolutionResult> for AiResolutionResult {
        type Error = anyhow::Error;

        fn try_from(value: super::AiResolutionResult) -> Result<Self, Self::Error> {
            let super::AiResolutionResult {
                commit_id,
                new_commit,
                summary,
                files,
                workspace,
            } = value;

            Ok(Self {
                commit_id: commit_id.into(),
                new_commit: new_commit.into(),
                summary,
                files,
                workspace: workspace.try_into()?,
            })
        }
    }
}
