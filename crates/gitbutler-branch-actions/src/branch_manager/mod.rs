use but_ctx::Context;

mod branch_creation;

pub struct BranchManager<'l> {
    ctx: &'l Context,
}

pub trait BranchManagerExt {
    fn branch_manager(&self) -> BranchManager<'_>;
}

impl BranchManagerExt for Context {
    fn branch_manager(&self) -> BranchManager<'_> {
        BranchManager { ctx: self }
    }
}
