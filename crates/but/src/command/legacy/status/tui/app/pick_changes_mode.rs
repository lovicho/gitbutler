use crate::command::legacy::status::tui::{Marks, render::ModeRender};

#[derive(Debug, Default, Clone)]
pub struct PickChangesMode {
    pub marks: Marks,
}

impl ModeRender for PickChangesMode {}
