use std::path::PathBuf;

use bstr::BStr;
use but_api::open::{
    list_builtin_program_specs, list_user_defined_program_specs,
    program::{ProgramSpec, open_in_program_unchecked},
};
use but_ctx::Context;
use gix::utils::AsBStr;

use crate::{
    CliError, CliResult, IdMap,
    args::atoms::{CliIdArg, Purpose, ResolvedCliIdArg},
    bad_input,
    id::UncommittedHunkOrFile,
};

#[derive(Debug)]
pub(crate) struct Hunk {
    /// The line numbers that were added in this hunk.
    pub line_nums_added: Vec<u32>,
    /// The line numbers that were removed in this hunk.
    pub line_nums_removed: Vec<u32>,
    /// The start position of the hunk in the old version.
    pub old_start: u32,
    /// The start position of the hunk in the new version.
    pub new_start: u32,
    /// Path to the file containing the hunk.
    pub path: PathBuf,
}

#[derive(Debug)]
pub(crate) enum Openable {
    File(PathBuf),
    Hunk(Hunk),
}

impl Openable {
    /// Try to create an [`Openable`] from an [`UncommittedHunkOrFile`].
    pub fn try_from_uncommitted(
        repo: &gix::Repository,
        uncommitted: &UncommittedHunkOrFile,
    ) -> anyhow::Result<Self> {
        let hunk = uncommitted.hunk_assignments.first();

        let path = repo
            .workdir_path(hunk.path_bytes.as_bstr())
            .ok_or_else(|| anyhow::anyhow!("Failed to resolve path to hunk"))?;

        let openable = match (
            uncommitted.is_entire_file,
            &hunk.hunk_header,
            &hunk.line_nums_added,
            &hunk.line_nums_removed,
        ) {
            (false, Some(hunk_header), Some(line_nums_added), Some(line_nums_removed)) => {
                Openable::Hunk(Hunk {
                    // Truncate line numbers - the probability of exceeding a u32 is infinitesimally small.
                    line_nums_added: line_nums_added
                        .iter()
                        .map(|n| (*n).min(u32::MAX as usize) as u32)
                        .collect(),
                    line_nums_removed: line_nums_removed
                        .iter()
                        .map(|n| (*n).min(u32::MAX as usize) as u32)
                        .collect(),
                    old_start: hunk_header.old_start,
                    new_start: hunk_header.new_start,
                    path,
                })
            }
            _ => Openable::File(path),
        };

        Ok(openable)
    }

    /// Try to create an [`Openable`] from a repository-relative path. Does NOT validate the path
    /// exists in the repository.
    pub fn try_from_relpath(repo: &gix::Repository, relpath: &BStr) -> anyhow::Result<Self> {
        repo.workdir_path(relpath)
            .map(Openable::File)
            .ok_or_else(|| anyhow::anyhow!("Failed to resolve path"))
    }
}

pub(crate) fn open(ctx: &Context, cli_id: CliIdArg, program_id: Option<String>) -> CliResult<()> {
    let guard = ctx.shared_worktree_access();
    let id_map = IdMap::new_from_context(ctx, None, guard.read_permission())?;
    let (repo, _ws, _db) = ctx.workspace_and_db_with_perm(guard.read_permission())?;

    let to_open = match cli_id.resolve_in_workspace(&repo, &id_map, Purpose::Uncommitted, None)? {
        ResolvedCliIdArg::UncommittedHunkOrFile(uncommitted) => {
            Openable::try_from_uncommitted(&repo, &uncommitted)?
        }
        resolved_id => {
            return Err(bad_input(format!(
                "Expected uncommitted file or hunk, got {}",
                resolved_id.kind_for_humans()
            ))
            .into());
        }
    };

    let builtin_program_specs = list_builtin_program_specs();
    let user_defined_program_specs = list_user_defined_program_specs();
    let mut all_program_specs = user_defined_program_specs
        .iter()
        .chain(builtin_program_specs);

    let program = match program_id {
        Some(program_id) => all_program_specs
            .find(|ps| ps.id == program_id)
            .ok_or_else(|| {
                CliError::from(
                    bad_input("No such program found")
                        .arg_name("--program-id")
                        .arg_value(program_id),
                )
            })?,
        None => all_program_specs
            .next()
            .expect("BUG: The internal list of programs should not be empty"),
    };

    run(program, &to_open)?;

    Ok(())
}

pub(crate) fn run(program: &ProgramSpec, to_open: &Openable) -> anyhow::Result<()> {
    let (path, line_number) = match to_open {
        Openable::Hunk(hunk) => (&hunk.path, Some(compute_line_number_to_open_at(hunk))),
        Openable::File(path) => (path, None),
    };

    open_in_program_unchecked(program, path, line_number)
}

/// Compute the line to place the cursor at, going through a priority order of additions ->
/// deletions -> hunk header start, falling through to the next thing if the prior one is absent.
fn compute_line_number_to_open_at(hunk: &Hunk) -> u32 {
    match (
        hunk.line_nums_added.as_slice(),
        hunk.line_nums_removed.as_slice(),
    ) {
        ([first_added, ..], _) => *first_added,
        (_, [first_removed, ..]) => {
            let leading_context = first_removed.saturating_sub(hunk.old_start);
            (hunk.new_start + leading_context).saturating_sub(1).max(1)
        }
        _ => hunk.new_start,
    }
}
