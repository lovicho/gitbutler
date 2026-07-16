use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use but_error::Code;
use but_meta::virtual_branches_legacy_types;
use itertools::Itertools;

use crate::stack::{Stack, StackId};

/// A handle to the state of virtual branches.
///
/// For all operations, if the state file does not exist, it will be created.
#[deprecated(note = "use ctx.workspace_* helpers instead of VirtualBranchesHandle")]
pub struct VirtualBranchesHandle {
    /// The path to the file containing the virtual branches state.
    file_path: PathBuf,
}

#[expect(
    deprecated,
    reason = "VirtualBranchesHandle should be replaced with ctx.workspace_* helpers"
)]
impl VirtualBranchesHandle {
    /// Creates a new concurrency-safe handle to the state of virtual branches.
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        let file_path = base_path.as_ref().join("virtual_branches.toml");
        Self { file_path }
    }

    /// Sets the state of the given virtual branch.
    ///
    /// Errors if the file cannot be read or written.
    pub fn set_stack(&mut self, stack: Stack) -> Result<()> {
        let mut virtual_branches = self.read_file()?;
        virtual_branches.branches.insert(stack.id, stack.into());
        self.write_file(&virtual_branches)?;
        Ok(())
    }

    /// Gets the state of the given virtual branch.
    ///
    /// Errors if the file cannot be read or written.
    pub fn get_stack(&self, id: StackId) -> Result<Stack> {
        self.read_file()?
            .branches
            .get(&id)
            .cloned()
            .map(Into::into)
            .ok_or_else(|| anyhow!("branch with ID {id} not found").context(Code::BranchNotFound))
    }

    /// Lists all virtual branches that are in the user's workspace.
    ///
    /// Errors if the file cannot be read or written.
    pub fn list_stacks_in_workspace(&self) -> Result<Vec<Stack>> {
        Ok(self
            .read_file()?
            .branches
            .into_values()
            .filter(|branch| branch.in_workspace)
            .map(Into::into)
            .collect())
    }

    /// Reads and parses the state file.
    ///
    /// If the file does not exist, it will be created.
    fn read_file(&self) -> Result<virtual_branches_legacy_types::VirtualBranches> {
        but_meta::legacy_storage::read_synced_virtual_branches(&self.file_path)
    }

    /// Write the given `virtual_branches` back to disk in one go.
    fn write_file(
        &mut self,
        virtual_branches: &virtual_branches_legacy_types::VirtualBranches,
    ) -> Result<()> {
        but_meta::legacy_storage::write_virtual_branches_and_sync(&self.file_path, virtual_branches)
    }

    pub fn next_order_index(&mut self) -> Result<usize> {
        let mut virtual_branches = self.read_file()?;
        let mut active_count = 0;
        for (order, stack) in virtual_branches
            .branches
            .values_mut()
            .filter(|stack| stack.in_workspace)
            .sorted_by_key(|stack| stack.order)
            .enumerate()
        {
            stack.order = order;
            active_count += 1;
        }
        self.write_file(&virtual_branches)?;
        Ok(active_count)
    }
}
