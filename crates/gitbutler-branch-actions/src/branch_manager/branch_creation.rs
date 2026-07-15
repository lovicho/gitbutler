use anyhow::{Context as _, Result};
use but_ctx::access::RepoExclusive;
use gitbutler_branch::{self, BranchCreateRequest, dedup};
use gitbutler_oplog::SnapshotExt;
use gitbutler_repo_actions::RepoActionsExt;
use gitbutler_stack::Stack;
use tracing::instrument;

use super::BranchManager;
use crate::VirtualBranchesExt;

impl BranchManager<'_> {
    #[instrument(level = "debug", skip(self, perm), err(Debug))]
    pub fn create_virtual_branch(
        &self,
        create: &BranchCreateRequest,
        perm: &mut RepoExclusive,
    ) -> Result<Stack> {
        let mut vb_state = self.ctx.virtual_branches();
        let target_base_oid = self.ctx.project_meta()?.target_commit_id_or_err()?;

        let mut all_stacks = vb_state
            .list_stacks_in_workspace()
            .context("failed to read virtual branches")?;

        let stack_names: Vec<String> = all_stacks.iter().map(|b| b.name()).collect();
        let stack_name_refs: Vec<&str> = stack_names.iter().map(|s| s.as_str()).collect();
        let name = dedup(
            &stack_name_refs,
            create.name.as_ref().unwrap_or(&"Lane".to_string()),
        );

        _ = self.ctx.snapshot_branch_creation(name.clone(), perm);

        all_stacks.sort_by_key(|branch| branch.order);

        let order = create.order.unwrap_or(vb_state.next_order_index()?);

        // make space for the new branch
        for (i, branch) in all_stacks.iter().enumerate() {
            let mut branch = branch.clone();
            let new_order = if i < order { i } else { i + 1 };
            if branch.order != new_order {
                branch.order = new_order;
                vb_state.set_stack(branch.clone())?;
            }
        }

        let branch = Stack::new_empty(self.ctx, name, target_base_oid, order)?;

        vb_state.set_stack(branch.clone())?;
        self.ctx.add_branch_reference(&branch)?;

        crate::integration::update_workspace_commit_with_vb_state(&vb_state, self.ctx, false)?;

        Ok(branch)
    }
}
