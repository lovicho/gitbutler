use but_testsupport::Sandbox;
use crossterm::event::{KeyCode, KeyModifiers};
use snapbox::{file, str};

use crate::command::legacy::status::tui::{backstack::BackstackEntry, tests::utils::test_tui};

#[test]
fn discard_prompt_can_be_cancelled() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack");
    env.setup_metadata(&["A"]);

    let mut tui = test_tui(env);

    tui.env().file("test.txt", "content");

    tui.reload()
        .assert_current_line_eq(str!["╭┄zz [uncommitted]"]);

    tui.input_then_render('x')
        .assert_rendered_contains("Discard uncommitted changes?")
        .assert_rendered_contains("<< discard >>");

    tui.input_then_render('n');

    tui.reload()
        .assert_current_line_eq(str!["╭┄zz [uncommitted]"])
        .assert_rendered_term_svg_eq(file!["snapshots/discard_prompt_can_be_cancelled_final.svg"]);
}

#[test]
fn discard_uncommitted_confirm_yes_discards_changes() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack");
    env.setup_metadata(&["A"]);

    let mut tui = test_tui(env);

    tui.env().file("test.txt", "content");

    tui.reload()
        .assert_current_line_eq(str!["╭┄zz [uncommitted]"]);

    tui.input_then_render('x')
        .assert_rendered_contains("Discard uncommitted changes?");

    tui.input_then_render('y');

    tui.reload()
        .assert_current_line_eq(str!["╭┄zz [uncommitted] (no changes)"]);

    let status = tui.env().invoke_git("status --porcelain");
    assert_eq!(status, "");

    tui.reload().assert_rendered_term_svg_eq(file![
        "snapshots/discard_uncommitted_confirm_yes_discards_changes_final.svg"
    ]);
}

#[test]
fn discard_uncommitted_cancel_keeps_changes() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack");
    env.setup_metadata(&["A"]);

    let mut tui = test_tui(env);

    tui.env().file("test.txt", "content");

    tui.reload()
        .assert_current_line_eq(str!["╭┄zz [uncommitted]"]);

    tui.input_then_render('x')
        .assert_rendered_contains("Discard uncommitted changes?");

    tui.input_then_render('n');

    tui.reload()
        .assert_current_line_eq(str!["╭┄zz [uncommitted]"]);

    let status = tui.env().invoke_git("status --porcelain");
    assert!(
        status.contains("test.txt"),
        "expected uncommitted changes to remain, got: {status:?}"
    );

    tui.reload().assert_rendered_term_svg_eq(file![
        "snapshots/discard_uncommitted_cancel_keeps_changes_final.svg"
    ]);
}

#[test]
fn discard_commit_confirm_yes_removes_commit() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack");
    env.setup_metadata(&["A"]);

    let mut tui = test_tui(env);

    tui.input_then_render([KeyCode::Down, KeyCode::Down])
        .assert_current_line_eq(str!["┊●   9477ae7 add A"]);

    tui.input_then_render('x')
        .assert_rendered_contains("Discard commit")
        .assert_rendered_contains("<< discard >>");

    tui.input_then_render('y');
    tui.reload();

    let log = tui.env().invoke_git("log --oneline");
    assert!(
        !log.contains("add A"),
        "expected discarded commit to be removed from history, got:\n{log}"
    );

    tui.reload().assert_rendered_term_svg_eq(file![
        "snapshots/discard_commit_confirm_yes_removes_commit_final.svg"
    ]);
}

#[test]
fn discard_top_commit_selects_next_commit_in_branch() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack");
    env.setup_metadata(&["A"]);

    let mut tui = test_tui(env);

    tui.input_then_render(KeyCode::Down)
        .assert_current_line_eq(str!["┊╭┄g0 [A]"]);

    tui.input_then_render('n')
        .assert_current_line_eq(str!["┊●   f184fc7 (no commit message) (no changes)"]);

    tui.input_then_render('n')
        .assert_current_line_eq(str!["┊●   9638f28 (no commit message) (no changes)"]);

    tui.input_then_render('x')
        .assert_rendered_contains("Discard commit")
        .assert_rendered_contains("<< discard >>");

    tui.input_then_render('y');

    tui.reload()
        .assert_current_line_eq(str!["┊●   f184fc7 (no commit message) (no changes)"]);
}

#[test]
fn discard_stack_confirm_yes_discards_staged_changes() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack");
    env.setup_metadata(&["A"]);

    let mut tui = test_tui(env);

    tui.env().file("test.txt", "content");

    tui.reload()
        .assert_current_line_eq(str!["╭┄zz [uncommitted]"]);

    tui.input_then_render(KeyCode::Down)
        .assert_current_line_eq(str!["┊   vo A test.txt"]);

    tui.input_then_render('r')
        .assert_current_line_eq(str!["┊   << source >> << noop >> vo A test.txt"]);

    tui.input_then_render(KeyCode::Down)
        .assert_current_line_eq(str!["┊●   << amend >> 9477ae7 add A"]);

    tui.input_then_render(KeyCode::Enter)
        .assert_current_line_eq(str!["┊●   8474410 add A"]);

    tui.input_then_render([KeyCode::Up, KeyCode::Up])
        .assert_current_line_eq(str!["╭┄zz [uncommitted] (no changes)"]);

    tui.input_then_render('x')
        .assert_rendered_contains("Discard uncommitted changes?")
        .assert_rendered_contains("<< discard >>");

    tui.input_then_render('y');

    tui.reload()
        .assert_current_line_eq(str!["╭┄zz [uncommitted] (no changes)"]);

    let status = tui.env().invoke_git("status --porcelain");
    assert_eq!(status, "");

    tui.reload().assert_rendered_term_svg_eq(file![
        "snapshots/discard_stack_confirm_yes_discards_staged_changes_final.svg"
    ]);
}

#[test]
fn discard_branch_confirm_yes_removes_branch() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack");
    env.setup_metadata(&["A"]);

    let mut tui = test_tui(env);

    tui.input_then_render(KeyCode::Down)
        .assert_current_line_eq(str!["┊╭┄g0 [A]"]);

    tui.input_then_render('b')
        .assert_current_line_eq(str!["┊╭┄br [c-branch-1] (no commits)"]);

    tui.input_then_render('x')
        .assert_rendered_contains("Discard branch c-branch-1?")
        .assert_rendered_contains("<< discard >>");

    tui.input_then_render('y');

    tui.reload().assert_current_line_eq(str!["┊╭┄g0 [A]"]);

    let branches = tui.env().invoke_git("branch --list");
    assert!(
        !branches.contains("c-branch-1"),
        "expected branch c-branch-1 to be removed, got: {branches:?}"
    );

    tui.reload().assert_rendered_term_svg_eq(file![
        "snapshots/discard_branch_confirm_yes_removes_branch_final.svg"
    ]);
}

#[test]
fn discard_branch_cancel_keeps_branch() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack");
    env.setup_metadata(&["A"]);

    let mut tui = test_tui(env);

    tui.input_then_render(KeyCode::Down)
        .assert_current_line_eq(str!["┊╭┄g0 [A]"]);

    tui.input_then_render('b')
        .assert_current_line_eq(str!["┊╭┄br [c-branch-1] (no commits)"]);

    tui.input_then_render('x')
        .assert_rendered_contains("Discard branch c-branch-1?");

    tui.input_then_render('n');

    tui.reload()
        .assert_current_line_eq(str!["┊╭┄br [c-branch-1] (no commits)"]);

    let branches = tui.env().invoke_git("branch --list");
    assert!(
        branches.contains("c-branch-1"),
        "expected branch c-branch-1 to remain, got: {branches:?}"
    );

    tui.reload().assert_rendered_term_svg_eq(file![
        "snapshots/discard_branch_cancel_keeps_branch_final.svg"
    ]);
}

#[test]
fn discard_multiple_commits() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("zero-stacks");
    env.setup_metadata(&[]);

    let mut tui = test_tui(env);

    tui.input_then_render('b')
        .assert_current_line_eq(str!["┊╭┄br [c-branch-1] (no commits)"]);

    for msg in ["one", "two", "three"] {
        tui.input_then_render('n');
        tui.input_then_render(KeyCode::Enter);
        tui.input_then_render(msg);
        tui.input_then_render(KeyCode::Enter);
    }

    tui.input_then_render(' ');
    tui.input_then_render(KeyCode::Down);
    tui.input_then_render(' ');

    tui.input_then_render('x')
        .assert_rendered_contains("Discard?");

    tui.input_then_render('y');

    tui.reload()
        .assert_rendered_term_svg_eq(file!["snapshots/discard_multiple_commits_final.svg"]);
}

#[test]
fn mark_and_discard_uncommitted_files() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("zero-stacks");
    env.setup_metadata(&[]);

    env.file("one", "");
    env.file("two", "");
    env.file("three", "");

    let mut tui = test_tui(env);

    tui.input_then_render('j');
    tui.input_then_render(' ');
    tui.input_then_render('j');
    tui.input_then_render(' ');

    tui.input_then_render('x');
    tui.input_then_render('y');

    tui.reload().assert_rendered_term_svg_eq(file![
        "snapshots/mark_and_discard_uncommitted_files_final.svg"
    ]);
}

#[test]
fn discard_individual_committed_files_from_local_file_list() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("zero-stacks");
    env.setup_metadata(&[]);

    env.file("one", "");
    env.file("two", "");
    env.file("three", "");

    let mut tui = test_tui(env);

    tui.input_then_render('c');
    tui.input_then_render('e');
    tui.input_then_render('b');

    tui.input_then_render('f')
        .assert_rendered_term_svg_eq(file![
            "snapshots/discard_individual_committed_files_from_local_file_list_001.svg"
        ]);

    tui.input_then_render('x')
        .assert_rendered_term_svg_eq(file![
            "snapshots/discard_individual_committed_files_from_local_file_list_002.svg"
        ]);
    tui.input_then_render('y')
        .assert_current_line_eq(str![["┊│     5f:or A three"]])
        .assert_rendered_term_svg_eq(file![
            "snapshots/discard_individual_committed_files_from_local_file_list_003.svg"
        ])
        .assert_backstack_eq([BackstackEntry::ShowFileList]);

    tui.input_then_render('x')
        .assert_rendered_term_svg_eq(file![
            "snapshots/discard_individual_committed_files_from_local_file_list_004.svg"
        ]);
    tui.input_then_render('y')
        .assert_current_line_eq(str![["┊│     c0:tw A two"]])
        .assert_rendered_term_svg_eq(file![
            "snapshots/discard_individual_committed_files_from_local_file_list_005.svg"
        ])
        .assert_backstack_eq([BackstackEntry::ShowFileList]);

    tui.input_then_render('x')
        .assert_rendered_term_svg_eq(file![
            "snapshots/discard_individual_committed_files_from_local_file_list_006.svg"
        ]);
    tui.input_then_render('y')
        .assert_current_line_eq(str![["┊●   0b42c46 (no commit message) (no changes)"]])
        .assert_rendered_term_svg_eq(file![
            "snapshots/discard_individual_committed_files_from_local_file_list_007.svg"
        ])
        .assert_backstack_eq([]);
}

#[test]
fn discard_individual_committed_files_from_global_file_list() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("zero-stacks");
    env.setup_metadata(&[]);

    env.file("one", "");
    env.file("two", "");
    env.file("three", "");

    let mut tui = test_tui(env);

    tui.input_then_render('c');
    tui.input_then_render('e');
    tui.input_then_render('b');
    tui.input_then_render((KeyModifiers::SHIFT, 'F'))
        .assert_rendered_term_svg_eq(file![
            "snapshots/discard_individual_committed_files_from_global_file_list_001.svg"
        ]);

    tui.input_then_render('j');

    tui.input_then_render('x')
        .assert_rendered_term_svg_eq(file![
            "snapshots/discard_individual_committed_files_from_global_file_list_002.svg"
        ]);
    tui.input_then_render('y')
        .assert_rendered_term_svg_eq(file![
            "snapshots/discard_individual_committed_files_from_global_file_list_003.svg"
        ]);

    tui.input_then_render('x');
    tui.input_then_render('y')
        .assert_rendered_term_svg_eq(file![
            "snapshots/discard_individual_committed_files_from_global_file_list_004.svg"
        ]);

    tui.input_then_render('x');
    tui.input_then_render('y')
        .assert_rendered_term_svg_eq(file![
            "snapshots/discard_individual_committed_files_from_global_file_list_005.svg"
        ])
        .assert_backstack_eq([]);
}

#[test]
fn discard_marked_committed_files_from_local_file_list() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("zero-stacks");
    env.setup_metadata(&[]);

    env.file("one", "");
    env.file("two", "");
    env.file("three", "");

    let mut tui = test_tui(env);

    tui.input_then_render('c');
    tui.input_then_render('e');
    tui.input_then_render('b');
    tui.input_then_render('f')
        .assert_rendered_term_svg_eq(file![
            "snapshots/discard_marked_committed_files_from_local_file_list_001.svg"
        ]);

    tui.input_then_render(' ');
    tui.input_then_render(' ')
        .assert_backstack_eq([BackstackEntry::Mark, BackstackEntry::ShowFileList])
        .assert_rendered_term_svg_eq(file![
            "snapshots/discard_marked_committed_files_from_local_file_list_002.svg"
        ]);

    tui.input_then_render('x')
        .assert_rendered_term_svg_eq(file![
            "snapshots/discard_marked_committed_files_from_local_file_list_003.svg"
        ]);
    tui.input_then_render('y')
        .assert_current_line_eq(str![["┊│     c0:tw A two"]])
        .assert_backstack_eq([BackstackEntry::ShowFileList])
        .assert_rendered_term_svg_eq(file![
            "snapshots/discard_marked_committed_files_from_local_file_list_004.svg"
        ]);

    tui.input_then_render(' ')
        .assert_backstack_eq([BackstackEntry::Mark, BackstackEntry::ShowFileList]);

    tui.input_then_render('x');
    tui.input_then_render('y')
        .assert_current_line_eq(str![["┊●   0b42c46 (no commit message) (no changes)"]])
        .assert_backstack_eq([])
        .assert_rendered_term_svg_eq(file![
            "snapshots/discard_marked_committed_files_from_local_file_list_005.svg"
        ]);
}

#[test]
fn discard_marked_committed_files_from_global_file_list() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("zero-stacks");
    env.setup_metadata(&[]);

    env.file("one", "");
    env.file("two", "");
    env.file("three", "");

    let mut tui = test_tui(env);

    tui.input_then_render('c');
    tui.input_then_render('e');
    tui.input_then_render('b');
    tui.input_then_render((KeyModifiers::SHIFT, 'F'))
        .assert_rendered_term_svg_eq(file![
            "snapshots/discard_marked_committed_files_from_global_file_list_001.svg"
        ]);

    tui.input_then_render('j');
    tui.input_then_render(' ');
    tui.input_then_render(' ')
        .assert_rendered_term_svg_eq(file![
            "snapshots/discard_marked_committed_files_from_global_file_list_002.svg"
        ]);

    tui.input_then_render('x')
        .assert_rendered_term_svg_eq(file![
            "snapshots/discard_marked_committed_files_from_global_file_list_003.svg"
        ]);
    tui.input_then_render('y')
        .assert_rendered_term_svg_eq(file![
            "snapshots/discard_marked_committed_files_from_global_file_list_004.svg"
        ]);

    tui.input_then_render('k')
        .assert_rendered_term_svg_eq(file![
            "snapshots/discard_marked_committed_files_from_global_file_list_005.svg"
        ]);
}

#[test]
fn global_file_list_stays_open_after_discarding_the_last_file_in_a_commit() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);

    let mut tui = test_tui(env);

    tui.input_then_render((KeyModifiers::SHIFT, 'F'))
        .assert_rendered_term_svg_eq(file![
            "snapshots/global_file_list_stays_open_after_discarding_the_last_file_in_a_commit_001.svg"
        ]);

    // discard the file in the top commit
    tui.input_then_render('j');
    tui.input_then_render('j');
    tui.input_then_render('j')
        .assert_current_line_eq(str![["┊│     94:tm A A"]]);
    tui.input_then_render('x');
    tui.input_then_render('y');

    // after discarding the last file in the commit the global file list should still be open
    tui.input_then_render('g')
        .assert_rendered_term_svg_eq(file![
            "snapshots/global_file_list_stays_open_after_discarding_the_last_file_in_a_commit_002.svg"
        ])
        .assert_backstack_eq([BackstackEntry::ShowFileList]);
}

#[test]
fn global_file_list_stays_open_after_marking_and_discarding_all_files_in_a_commit() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);

    let mut tui = test_tui(env);

    tui.input_then_render((KeyModifiers::SHIFT, 'F'));

    tui.input_then_render('j');
    tui.input_then_render('j');
    tui.input_then_render('j');
    tui.input_then_render(' ');
    tui.input_then_render('x')
        .assert_rendered_term_svg_eq(file![
            "snapshots/global_file_list_stays_open_after_marking_and_discarding_all_files_in_a_commit_001.svg"
        ]);
    tui.input_then_render('y')
        .assert_rendered_term_svg_eq(file![
            "snapshots/global_file_list_stays_open_after_marking_and_discarding_all_files_in_a_commit_002.svg"
        ])
        .assert_backstack_eq([BackstackEntry::ShowFileList]);
}
