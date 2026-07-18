use crate::{command::undo::run_mutate_undo_roundtrip_test, utils::Sandbox};

#[test]
fn can_undo_but_uncommit_commit_add() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack");
    env.setup_metadata_at_target(&["A"], "origin/main");
    let path = "new-file.txt";
    env.file(path, "content");

    env.but("commit -m 'Add file'").assert().success();

    run_mutate_undo_roundtrip_test(&env, |env| {
        env.but("uncommit 1").assert().success();
    });
}

#[test]
fn can_undo_but_uncommit_commit_modify() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack");
    env.setup_metadata_at_target(&["A"], "origin/main");
    let path = "new-file.txt";
    env.file(path, "content");

    env.but("commit -m 'Add file'").assert().success();

    let path = "new-file.txt";
    env.file(path, "changed content");

    env.but("commit -m 'Change file'").assert().success();

    run_mutate_undo_roundtrip_test(&env, |env| {
        env.but("uncommit 1#0").assert().success();
    });
}

#[test]
fn can_undo_but_uncommit_commit_delete() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack");
    env.setup_metadata_at_target(&["A"], "origin/main");
    let path = "new-file.txt";
    env.file(path, "content");

    env.but("commit -m 'Add file'").assert().success();

    env.remove_file(path);

    env.but("commit -m 'Remove file'").assert().success();

    run_mutate_undo_roundtrip_test(&env, |env| {
        env.but("uncommit 1#0").assert().success();
    });
}

// Regression test for GB-1772: discarding a commit while an uncommitted change touches the same
// file left `undo` unable to restore. Restore checks out the snapshot's workdir tree, which
// already contains the uncommitted change, but `safe_checkout_from_head` was re-applying the
// still-present uncommitted change on top of it, so the change collided with itself.
#[test]
fn can_undo_discard_commit_with_overlapping_uncommitted_change() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack");
    env.setup_metadata_at_target(&["A"], "origin/main");
    let path = "new-file.txt";

    env.file(path, "line 1\n");
    env.but("commit -m 'Add file'").assert().success();

    env.file(path, "line 1\nline 2\n");
    env.but("commit -m 'Update file'").assert().success();

    // Uncommitted change to the same file that the discarded commit also touches.
    env.file(path, "line 1\nline 2\nline 3\n");

    run_mutate_undo_roundtrip_test(&env, |env| {
        env.but("uncommit 1#0 --discard").assert().success();
    });
}

#[test]
fn can_undo_but_uncommit_file_add() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack");
    env.setup_metadata_at_target(&["A"], "origin/main");
    let path = "new-file.txt";
    env.file(path, "content");

    env.but("commit -m 'Add file'").assert().success();

    run_mutate_undo_roundtrip_test(&env, |env| {
        env.but("uncommit 1:xk").assert().success();
    });
}

#[test]
fn can_undo_but_uncommit_file_modify() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack");
    env.setup_metadata_at_target(&["A"], "origin/main");
    let path = "new-file.txt";
    env.file(path, "content");

    env.but("commit -m 'Add file'").assert().success();

    env.file(path, "new content");
    env.but("commit -m 'Modify file'").assert().success();

    run_mutate_undo_roundtrip_test(&env, |env| {
        env.but("uncommit 1#0:xk").assert().success();
    });
}

#[test]
fn can_undo_but_uncommit_file_delete() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack");
    env.setup_metadata_at_target(&["A"], "origin/main");
    let path = "new-file.txt";
    env.file(path, "content");

    env.but("commit -m 'Add file'").assert().success();

    env.remove_file(path);
    env.but("commit -m 'Remove file'").assert().success();

    run_mutate_undo_roundtrip_test(&env, |env| {
        env.but("uncommit 1#0:xk").assert().success();
    });
}
