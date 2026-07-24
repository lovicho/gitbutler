use snapbox::str;

use crate::{
    command::util::{branch_commit_cli_ids, status_json_with_files as status_json},
    utils::Sandbox,
};

fn uncommitted_contains_file(status: &serde_json::Value, file_path: &str) -> bool {
    status["uncommittedChanges"]
        .as_array()
        .unwrap()
        .iter()
        .any(|change| change["filePath"].as_str().unwrap() == file_path)
}

fn branch_commits_contain_file(
    status: &serde_json::Value,
    branch_name: &str,
    file_path: &str,
) -> bool {
    status["stacks"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|stack| stack["branches"].as_array().unwrap().iter())
        .filter(|branch| branch["name"].as_str().unwrap() == branch_name)
        .flat_map(|branch| branch["commits"].as_array().unwrap().iter())
        .flat_map(|commit| commit["changes"].as_array().unwrap().iter())
        .any(|change| change["filePath"].as_str().unwrap() == file_path)
}

#[test]
fn amend_rejects_dependency_changes() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("zero-stacks");
    env.setup_metadata(&[]);

    // Commit `first` to branch foo and an unrelated file to branch bar.
    env.file("first", "Some text");
    env.but("commit -m 'add first' -b foo").assert().success();
    env.file("second", "Other text");
    env.but("commit -m 'add second' -b bar").assert().success();

    // Change `first` (which depends on foo) and try to amend it into bar's
    // commit. The squash internals reject the operation atomically.
    env.file("first", "changes");
    let status = status_json(&env)?;
    let bar_commit_cli_id = branch_commit_cli_ids(&status, "bar")[0].clone();
    env.but(format!("amend first --target {bar_commit_cli_id}"))
        .assert()
        .failure()
        .stdout_eq(str![""])
        .stderr_eq(str![[r#"
Error: Couldn't squash all changes

"#]]);

    let after = status_json(&env)?;
    assert!(
        uncommitted_contains_file(&after, "first"),
        "a rejected amend must leave its source uncommitted"
    );
    assert!(
        !branch_commits_contain_file(&after, "bar", "first"),
        "a rejected amend must not modify the target branch"
    );

    Ok(())
}

#[test]
fn amend_accepts_multiple_uncommitted_changes() {
    assert_multiple_amend(|target_cli_id| {
        format!("amend one.txt two.txt --target {target_cli_id}")
    })
    .unwrap();
}

#[test]
fn amend_accepts_branch_target() {
    assert_multiple_amend(|_target_cli_id| "amend one.txt two.txt --target A".to_string()).unwrap();
}

fn assert_multiple_amend(args: impl FnOnce(&str) -> String) -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);

    env.file("one.txt", "one\n");
    env.file("two.txt", "two\n");
    env.file("three.txt", "three\n");

    let before = status_json(&env)?;
    let target_cli_id = branch_commit_cli_ids(&before, "A")[0].clone();

    env.but(args(&target_cli_id))
        .assert()
        .success()
        .stdout_eq(str![[r#"
Amended [..] to create [..]

"#]])
        .stderr_eq(str![""]);

    let after = status_json(&env)?;
    assert!(
        !uncommitted_contains_file(&after, "one.txt"),
        "first amended file should no longer be uncommitted"
    );
    assert!(
        !uncommitted_contains_file(&after, "two.txt"),
        "second amended file should no longer be uncommitted"
    );
    assert!(
        uncommitted_contains_file(&after, "three.txt"),
        "unmentioned file should remain uncommitted"
    );
    assert!(
        branch_commits_contain_file(&after, "A", "one.txt"),
        "first file should be amended into a commit on branch A"
    );
    assert!(
        branch_commits_contain_file(&after, "A", "two.txt"),
        "second file should be amended into a commit on branch A"
    );

    Ok(())
}
