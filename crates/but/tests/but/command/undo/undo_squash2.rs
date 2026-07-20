use crate::{
    command::{
        undo::run_mutate_undo_roundtrip_test,
        util::{
            branch_commit_cli_id_for_file, branch_commit_cli_ids,
            commit_two_files_as_two_hunks_each, status_json, status_json_with_files,
        },
    },
    utils::Sandbox,
};

// squash2: SquashCommits
#[test]
fn undo_squash_commits() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack-two-commits");
    env.setup_metadata(&["A"]);

    run_mutate_undo_roundtrip_test(&env, |env| {
        env.but("_squash2 y -t z -u").assert().success();
    });
}

// squash2: UncommittedToCommit
#[test]
fn undo_uncommitted_hunk_to_commit() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);
    env.file("uncommitted-to-commit.txt", "content\n");
    let target_cli_id = branch_commit_cli_ids(&status_json(&env).unwrap(), "A")[0].clone();

    run_mutate_undo_roundtrip_test(&env, |env| {
        env.but(format!(
            "_squash2 uncommitted-to-commit.txt -t {target_cli_id} -u"
        ))
        .assert()
        .success();
    });
}

// squash2: UncommittedAreaToCommit
#[test]
fn undo_uncommitted_area_to_commit() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);
    env.file("uncommitted-to-commit.txt", "content\n");
    let target_cli_id = branch_commit_cli_ids(&status_json(&env).unwrap(), "A")[0].clone();

    run_mutate_undo_roundtrip_test(&env, |env| {
        env.but(format!("_squash2 zz -t {target_cli_id} -u"))
            .assert()
            .success();
    });
}

// squash2: CommitToUncommittedArea
#[test]
fn undo_commit_to_uncommitted_area() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);
    commit_two_files_as_two_hunks_each(
        &env,
        "A",
        "commit-to-zz-a.txt",
        "commit-to-zz-b.txt",
        "first",
    );
    let source_cli_id = branch_commit_cli_ids(&status_json(&env).unwrap(), "A")[0].clone();

    run_mutate_undo_roundtrip_test(&env, |env| {
        env.but(format!("_squash2 {source_cli_id} -t zz"))
            .assert()
            .success();
    });
}

// squash2: CommittedFileToCommit
#[test]
fn undo_committed_file_to_commit() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack");
    env.setup_metadata(&["A"]);
    commit_two_files_as_two_hunks_each(&env, "A", "source-a.txt", "source-b.txt", "source");
    commit_two_files_as_two_hunks_each(&env, "A", "target-a.txt", "target-b.txt", "target");
    let status = status_json_with_files(&env).unwrap();
    let source_cli_id = branch_commit_cli_id_for_file(&status, "A", "source-a.txt").unwrap();
    let target_cli_id = branch_commit_cli_id_for_file(&status, "A", "target-a.txt").unwrap();

    run_mutate_undo_roundtrip_test(&env, |env| {
        env.but(format!(
            "_squash2 {source_cli_id}:source-a.txt -t {target_cli_id} -u"
        ))
        .assert()
        .success();
    });
}

// squash2: CommittedFileToUncommittedArea
#[test]
fn undo_committed_file_to_uncommitted_area() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);
    commit_two_files_as_two_hunks_each(&env, "A", "file-to-zz-a.txt", "file-to-zz-b.txt", "first");
    let source_cli_id = branch_commit_cli_ids(&status_json(&env).unwrap(), "A")[0].clone();

    run_mutate_undo_roundtrip_test(&env, |env| {
        env.but(format!("_squash2 {source_cli_id}:file-to-zz-a.txt -t zz"))
            .assert()
            .success();
    });
}
