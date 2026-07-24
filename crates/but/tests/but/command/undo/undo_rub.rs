use crate::{command::undo::run_mutate_undo_roundtrip_test, utils::Sandbox};

// RubOperation::UnassignUncommitted
#[test]
#[ignore = "undo currently does not restore hunk assignment metadata for rub operations that only move changes between uncommitted, branch, and stack buckets. https://linear.app/gitbutler/issue/GB-1435/cannot-undo-rub-operations-that-deal-with-uncommitted-changes"]
fn undo_unassign_uncommitted() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);
    env.file("unassign-uncommitted.txt", "content\n");
    env.but("rub unassign-uncommitted.txt A").assert().success();

    run_mutate_undo_roundtrip_test(&env, |env| {
        env.but("rub A@{stack}:unassign-uncommitted.txt zz")
            .assert()
            .success()
            .stdout_eq("Unstaged the only hunk in unassign-uncommitted.txt in a stack\n")
            .stderr_eq("");
    });
}

// RubOperation::UncommittedToBranch
#[test]
#[ignore = "undo currently does not restore hunk assignment metadata for rub operations that only move changes between uncommitted, branch, and stack buckets. https://linear.app/gitbutler/issue/GB-1435/cannot-undo-rub-operations-that-deal-with-uncommitted-changes"]
fn undo_uncommitted_hunk_to_branch() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);
    env.file("uncommitted-to-branch.txt", "content\n");

    run_mutate_undo_roundtrip_test(&env, |env| {
        env.but("rub uncommitted-to-branch.txt A")
            .assert()
            .success()
            .stdout_eq(
                "Staged the only hunk in uncommitted-to-branch.txt in the uncommitted area → [A].\n",
            )
            .stderr_eq("");
    });
}

// RubOperation::UncommittedToStack
#[test]
#[ignore = "undo currently does not restore hunk assignment metadata for rub operations that only move changes between uncommitted, branch, and stack buckets. https://linear.app/gitbutler/issue/GB-1435/cannot-undo-rub-operations-that-deal-with-uncommitted-changes"]
fn undo_uncommitted_hunk_to_stack() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);
    env.file("uncommitted-to-stack.txt", "content\n");

    run_mutate_undo_roundtrip_test(&env, |env| {
        env.but("rub uncommitted-to-stack.txt A@{stack}")
            .assert()
            .success()
            .stdout_eq("Staged the only hunk in uncommitted-to-stack.txt in the uncommitted area → stack [..].\n")
            .stderr_eq("");
    });
}

// RubOperation::StackToUncommittedArea
#[test]
#[ignore = "undo currently does not restore hunk assignment metadata for rub operations that only move changes between uncommitted, branch, and stack buckets. https://linear.app/gitbutler/issue/GB-1435/cannot-undo-rub-operations-that-deal-with-uncommitted-changes"]
fn undo_stack_to_uncommitted_area() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);
    env.file("stack-to-uncommitted.txt", "content\n");
    env.but("rub stack-to-uncommitted.txt A").assert().success();

    run_mutate_undo_roundtrip_test(&env, |env| {
        env.but("rub A@{stack} zz")
            .assert()
            .success()
            .stdout_eq("Unstaged all [A] changes.\n")
            .stderr_eq("");
    });
}

// RubOperation::StackToStack
#[test]
#[ignore = "undo currently does not restore hunk assignment metadata for rub operations that only move changes between uncommitted, branch, and stack buckets. https://linear.app/gitbutler/issue/GB-1435/cannot-undo-rub-operations-that-deal-with-uncommitted-changes"]
fn undo_stack_to_stack() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);
    env.file("stack-to-stack.txt", "content\n");
    env.but("rub stack-to-stack.txt A").assert().success();

    run_mutate_undo_roundtrip_test(&env, |env| {
        env.but("rub A@{stack} B@{stack}")
            .assert()
            .success()
            .stdout_eq("Staged all [A] changes to [B].\n")
            .stderr_eq("");
    });
}

// RubOperation::StackToBranch
#[test]
#[ignore = "undo currently does not restore hunk assignment metadata for rub operations that only move changes between uncommitted, branch, and stack buckets. https://linear.app/gitbutler/issue/GB-1435/cannot-undo-rub-operations-that-deal-with-uncommitted-changes"]
fn undo_stack_to_branch() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);
    env.file("stack-to-branch.txt", "content\n");
    env.but("rub stack-to-branch.txt A").assert().success();

    run_mutate_undo_roundtrip_test(&env, |env| {
        env.but("rub A@{stack} B")
            .assert()
            .success()
            .stdout_eq("Staged all [A] changes to [B].\n")
            .stderr_eq("");
    });
}

// RubOperation::UncommittedAreaToBranch
#[test]
#[ignore = "undo currently does not restore hunk assignment metadata for rub operations that only move changes between uncommitted, branch, and stack buckets. https://linear.app/gitbutler/issue/GB-1435/cannot-undo-rub-operations-that-deal-with-uncommitted-changes"]
fn undo_uncommitted_area_to_branch() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);
    env.file("uncommitted-to-branch.txt", "content\n");

    run_mutate_undo_roundtrip_test(&env, |env| {
        env.but("rub zz A")
            .assert()
            .success()
            .stdout_eq("Staged all unstaged changes to [A].\n")
            .stderr_eq("");
    });
}

// RubOperation::UncommittedAreaToStack
#[test]
#[ignore = "undo currently does not restore hunk assignment metadata for rub operations that only move changes between uncommitted, branch, and stack buckets. https://linear.app/gitbutler/issue/GB-1435/cannot-undo-rub-operations-that-deal-with-uncommitted-changes"]
fn undo_uncommitted_area_to_stack() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);
    env.file("uncommitted-to-stack.txt", "content\n");

    run_mutate_undo_roundtrip_test(&env, |env| {
        env.but("rub zz A@{stack}")
            .assert()
            .success()
            .stdout_eq("Staged all unstaged changes to [A].\n")
            .stderr_eq("");
    });
}

// RubOperation::BranchToUncommittedArea
#[test]
#[ignore = "undo currently does not restore hunk assignment metadata for rub operations that only move changes between uncommitted, branch, and stack buckets. https://linear.app/gitbutler/issue/GB-1435/cannot-undo-rub-operations-that-deal-with-uncommitted-changes"]
fn undo_branch_to_uncommitted_area() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);
    env.file("branch-to-uncommitted.txt", "content\n");
    env.but("rub branch-to-uncommitted.txt A")
        .assert()
        .success();

    run_mutate_undo_roundtrip_test(&env, |env| {
        env.but("rub A zz")
            .assert()
            .success()
            .stdout_eq("Unstaged all [A] changes.\n")
            .stderr_eq("");
    });
}

// RubOperation::BranchToStack
#[test]
#[ignore = "undo currently does not restore hunk assignment metadata for rub operations that only move changes between uncommitted, branch, and stack buckets. https://linear.app/gitbutler/issue/GB-1435/cannot-undo-rub-operations-that-deal-with-uncommitted-changes"]
fn undo_branch_to_stack() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);
    env.file("branch-to-stack.txt", "content\n");
    env.but("rub branch-to-stack.txt A").assert().success();

    run_mutate_undo_roundtrip_test(&env, |env| {
        env.but("rub A B@{stack}")
            .assert()
            .success()
            .stdout_eq("Staged all [A] changes to [B].\n")
            .stderr_eq("");
    });
}

// RubOperation::BranchToBranch
#[test]
#[ignore = "undo currently does not restore hunk assignment metadata for rub operations that only move changes between uncommitted, branch, and stack buckets. https://linear.app/gitbutler/issue/GB-1435/cannot-undo-rub-operations-that-deal-with-uncommitted-changes"]
fn undo_branch_to_branch() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);
    env.file("branch-to-branch.txt", "content\n");
    env.but("rub branch-to-branch.txt A").assert().success();

    run_mutate_undo_roundtrip_test(&env, |env| {
        env.but("rub A B")
            .assert()
            .success()
            .stdout_eq("Staged all [A] changes to [B].\n")
            .stderr_eq("");
    });
}
