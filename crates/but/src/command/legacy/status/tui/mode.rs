use ratatui::style::Color;

use crate::{
    command::legacy::status::tui::{
        InlineRewordMode,
        app::mark::MarksRef,
        app::{
            CommandMode, CommitMode, CommitSource, JumpMode, MoveMode, MoveSource, MoveStackMode,
            NormalMode, PickChangesMode, RubMode, RubSource, StackMode,
        },
        render::ModeRender,
    },
    theme::Theme,
};

#[derive(Debug, Clone, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::EnumIter, Hash))]
#[strum_discriminants(name(ModeDiscriminant))]
pub enum Mode {
    Normal(NormalMode),
    Rub(RubMode),
    InlineReword(InlineRewordMode),
    Command(CommandMode),
    Commit(CommitMode),
    Move(MoveMode),
    Details(DetailsMode),
    Stack(StackMode),
    MoveStack(MoveStackMode),
    PickChanges(PickChangesMode),
    Jump(JumpMode),
}

impl Default for Mode {
    fn default() -> Self {
        Self::Normal(Default::default())
    }
}

impl Mode {
    pub fn bg(&self, theme: &'static Theme) -> Color {
        ModeDiscriminant::from(self).bg(theme)
    }

    #[expect(dead_code)]
    pub fn fg(&self, theme: &'static Theme) -> Color {
        ModeDiscriminant::from(self).fg(theme)
    }

    pub fn marks_ref(&self) -> MarksRef<'_> {
        match self {
            Mode::Normal(normal_mode) => normal_mode.marks.as_ref(),
            Mode::Rub(rub_mode) => match &rub_mode.source {
                RubSource::Marks(marks) => marks.as_ref(),
                RubSource::CliId(..) => MarksRef::Empty,
            },
            Mode::Commit(commit_mode) => match &*commit_mode.source {
                CommitSource::Marks(hunks) => MarksRef::from_hunks(hunks),
                CommitSource::UncommittedArea(..)
                | CommitSource::Uncommitted(..)
                | CommitSource::Stack(..) => MarksRef::Empty,
            },
            Mode::PickChanges(pick_uncommitted_mode) => {
                MarksRef::from_hunk_slice(&pick_uncommitted_mode.marks)
            }
            Mode::Details(details_mode) => details_mode.return_mode.marks(),
            Mode::Move(move_mode) => match &*move_mode.source {
                MoveSource::Marks(commits) => MarksRef::from_commits(commits),
                MoveSource::Commit { .. } | MoveSource::Branch { .. } => MarksRef::Empty,
            },
            Mode::InlineReword(..)
            | Mode::Command(..)
            | Mode::Stack(..)
            | Mode::MoveStack(..)
            | Mode::Jump(..) => MarksRef::Empty,
        }
    }
}

impl ModeDiscriminant {
    pub fn bg(self, theme: &'static Theme) -> Color {
        match self {
            Self::Normal => theme.tui_mode_normal.bg.unwrap_or(Color::DarkGray),
            Self::Commit | Self::PickChanges => theme.tui_mode_commit.bg.unwrap_or(Color::Green),
            Self::Rub | Self::Jump => theme.tui_mode_rub.bg.unwrap_or(Color::Blue),
            Self::InlineReword | Self::Stack => {
                theme.tui_mode_inline_reword.bg.unwrap_or(Color::Magenta)
            }
            Self::Command => theme.tui_mode_command.bg.unwrap_or(Color::Yellow),
            Self::Move | Self::MoveStack => theme.tui_mode_move.bg.unwrap_or(Color::Cyan),
            Self::Details => theme
                .tui_mode_details
                .bg
                .unwrap_or(Color::Rgb(255, 165, 0) /* orange */),
        }
    }

    pub fn fg(self, theme: &'static Theme) -> Color {
        match self {
            Self::Normal => theme.tui_mode_normal.fg.unwrap_or(Color::White),
            Self::Commit | Self::PickChanges => theme.tui_mode_commit.fg.unwrap_or(Color::Black),
            Self::Rub | Self::Jump => theme.tui_mode_rub.fg.unwrap_or(Color::Black),
            Self::InlineReword | Self::Stack => {
                theme.tui_mode_inline_reword.fg.unwrap_or(Color::Black)
            }
            Self::Command => theme.tui_mode_command.fg.unwrap_or(Color::Black),
            Self::Move | Self::MoveStack => theme.tui_mode_move.fg.unwrap_or(Color::Black),
            Self::Details => theme.tui_mode_details.fg.unwrap_or(Color::Black),
        }
    }

    pub fn hotbar_str(self) -> &'static str {
        match self {
            Self::Normal => "  normal  ",
            Self::Rub => "  rub  ",
            Self::InlineReword => "  reword  ",
            Self::Command => "  command  ",
            Self::Commit => "  commit  ",
            Self::PickChanges => "  pick changes  ",
            Self::Move => "  move  ",
            Self::Details => "  details  ",
            Self::Stack => "  stack  ",
            Self::MoveStack => "  move stack  ",
            Self::Jump => "  jump  ",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DetailsMode {
    pub full_screen: bool,
    pub return_mode: DetailsReturnMode,
}

#[derive(Debug, Clone)]
#[expect(clippy::large_enum_variant)]
pub enum DetailsReturnMode {
    Normal(NormalMode),
    PickChanges(PickChangesMode),
}

impl ModeRender for DetailsMode {}

impl DetailsReturnMode {
    fn marks(&self) -> MarksRef<'_> {
        match self {
            DetailsReturnMode::Normal(normal_mode) => normal_mode.marks.as_ref(),
            DetailsReturnMode::PickChanges(pick_uncommitted_mode) => {
                MarksRef::from_hunk_slice(&pick_uncommitted_mode.marks)
            }
        }
    }
}
