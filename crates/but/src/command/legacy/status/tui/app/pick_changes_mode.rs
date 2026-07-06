use crate::{
    command::legacy::status::tui::{app::mark::SingleSourceMarks, render::ModeRender},
    id::UncommittedHunkOrFile,
};

#[derive(Debug, Default, Clone)]
pub struct PickChangesMode {
    pub marks: SingleSourceMarks<UncommittedHunkOrFile>,
}

impl ModeRender for PickChangesMode {}
