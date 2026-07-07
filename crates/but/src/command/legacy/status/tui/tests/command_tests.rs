use but_testsupport::Sandbox;
use crossterm::event::KeyCode;
use snapbox::{file, str};

use super::utils::test_tui;

#[test]
fn command_mode_runs_successful_command_and_returns_to_normal_mode() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack");
    env.setup_metadata(&["A"]);

    let mut tui = test_tui(env);

    tui.input(':')
        .assert_rendered_term_svg_eq(file!["snapshots/command_mode_success_001.svg"]);

    tui.input("--help")
        .assert_rendered_term_svg_eq(file!["snapshots/command_mode_success_002.svg"]);

    tui.input(KeyCode::Enter)
        .assert_rendered_term_svg_eq(file!["snapshots/command_mode_success_003.svg"]);
}

#[test]
fn command_mode_keeps_input_when_command_exits_non_zero() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack");
    env.setup_metadata(&["A"]);

    let mut tui = test_tui(env);

    tui.input(':');

    tui.input("--definitely-not-a-real-flag");

    tui.input(KeyCode::Enter)
        .assert_rendered_term_svg_eq(file!["snapshots/command_mode_failure_001.svg"])
        .assert_current_line_eq(str!["╭┄zz [uncommitted] (no changes)"]);
}
