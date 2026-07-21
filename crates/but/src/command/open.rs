use but_api::open::{list_program_specs, program::open_in_program_unchecked};
use but_ctx::Context;
use but_hunk_assignment::HunkAssignment;
use gix::utils::AsBStr;

use crate::{
    CliResult, IdMap,
    args::atoms::{CliIdArg, Purpose, ResolvedCliIdArg},
    bad_input,
};

pub(crate) fn open(
    ctx: &mut Context,
    cli_id: CliIdArg,
    program_id: Option<String>,
) -> CliResult<()> {
    let guard = ctx.shared_worktree_access();
    let id_map = IdMap::new_from_context(ctx, None, guard.read_permission())?;
    let (repo, _ws, _db) = ctx.workspace_and_db_with_perm(guard.read_permission())?;

    let (path, line_nr) =
        match cli_id.resolve_in_workspace(&repo, &id_map, Purpose::Uncommitted, None)? {
            ResolvedCliIdArg::UncommittedHunkOrFile(uncommitted) => {
                let hunk = uncommitted.hunk_assignments.first();

                let line_nr = if uncommitted.is_entire_file {
                    None
                } else {
                    compute_line_number_to_open_at(hunk)
                };

                let Some(path) = repo.workdir_path(hunk.path_bytes.as_bstr()) else {
                    return Err(anyhow::anyhow!("Failed to resolve path to hunk").into());
                };

                (
                    path,
                    line_nr.map(i32::try_from).transpose().unwrap_or_default(),
                )
            }
            resolved_id => {
                return Err(bad_input(format!(
                    "Expected uncommitted file or hunk, got {}",
                    resolved_id.kind_for_humans()
                ))
                .into());
            }
        };

    let program = if let Some(program_id) = program_id {
        match list_program_specs().iter().find(|ps| ps.id == program_id) {
            Some(program) => program,
            None => {
                return Err(bad_input("No such program")
                    .arg_name("--program-id")
                    .arg_value(program_id)
                    .into());
            }
        }
    } else {
        list_program_specs()
            .first()
            .expect("The list of programs cannot be empty")
    };

    open_in_program_unchecked(program, &path, line_nr)?;

    Ok(())
}

/// Compute the line to place the cursor at, going through a priority order of additions ->
/// deletions -> hunk header start, falling through to the next thing if the prior one is absent.
fn compute_line_number_to_open_at(hunk: &HunkAssignment) -> Option<usize> {
    match (
        hunk.line_nums_added.as_deref(),
        hunk.line_nums_removed.as_deref(),
        hunk.hunk_header,
    ) {
        (Some([first_added, ..]), _, _) => Some(*first_added),
        (_, Some([first_removed, ..]), Some(hunk_header)) => {
            let leading_context = first_removed.saturating_sub(hunk_header.old_start as usize);
            Some(
                (hunk_header.new_start as usize + leading_context)
                    .saturating_sub(1)
                    .max(1),
            )
        }
        (_, _, Some(hunk_header)) => Some(hunk_header.new_start as usize),
        _ => None,
    }
}
