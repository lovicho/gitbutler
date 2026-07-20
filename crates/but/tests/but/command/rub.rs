use snapbox::str;

use crate::{
    command::util::{
        branch_commit_cli_ids, commit_two_files_as_two_hunks_each,
        status_json_with_files as status_json,
    },
    utils::Sandbox,
};

fn stack_assigned_contains_file(
    status: &serde_json::Value,
    branch_name: &str,
    file_path: &str,
) -> bool {
    status["stacks"].as_array().unwrap().iter().any(|stack| {
        let has_branch = stack["branches"]
            .as_array()
            .unwrap()
            .iter()
            .any(|branch| branch["name"].as_str().unwrap() == branch_name);
        has_branch
            && stack["assignedChanges"]
                .as_array()
                .unwrap()
                .iter()
                .any(|change| change["filePath"].as_str().unwrap() == file_path)
    })
}

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
fn rub_matrix_uncommitted_hunk_to_stack_smoke() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);

    env.file("uncommitted-to-stack.txt", "content\n");

    env.but("rub uncommitted-to-stack.txt A@{stack}")
        .assert()
        .success()
        .stdout_eq(str![[r#"
Staged the only hunk in uncommitted-to-stack.txt in the uncommitted area → stack [..].

"#]])
        .stderr_eq(str![""]);

    let after = status_json(&env)?;
    assert!(
        stack_assigned_contains_file(&after, "A", "uncommitted-to-stack.txt"),
        "file should be assigned to A stack"
    );

    Ok(())
}

#[test]
fn rub_matrix_uncommitted_area_to_branch_smoke() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);

    env.file("zz-to-branch.txt", "content\n");

    env.but("rub zz A")
        .assert()
        .success()
        .stdout_eq(str![[r#"
Staged all unstaged changes to [A].

"#]])
        .stderr_eq(str![""]);

    let after = status_json(&env)?;
    assert!(
        !uncommitted_contains_file(&after, "zz-to-branch.txt"),
        "file should no longer be uncommitted"
    );
    assert!(
        stack_assigned_contains_file(&after, "A", "zz-to-branch.txt"),
        "file should be assigned to branch A stack"
    );

    Ok(())
}

#[test]
fn rub_matrix_uncommitted_area_to_stack_smoke() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);

    env.file("zz-to-stack.txt", "content\n");

    env.but("rub zz A@{stack}")
        .assert()
        .success()
        .stdout_eq(str![[r#"
Staged all unstaged changes to [A].

"#]])
        .stderr_eq(str![""]);

    let after = status_json(&env)?;
    assert!(
        !uncommitted_contains_file(&after, "zz-to-stack.txt"),
        "file should no longer be uncommitted"
    );
    assert!(
        stack_assigned_contains_file(&after, "A", "zz-to-stack.txt"),
        "file should be assigned to A stack"
    );

    Ok(())
}

#[test]
fn rub_matrix_commit_to_stack_smoke() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);

    commit_two_files_as_two_hunks_each(&env, "A", "a.txt", "b.txt", "first commit");

    let before = status_json(&env)?;
    let source_cli_id = branch_commit_cli_ids(&before, "A")[0].clone();

    env.but(format!("rub {source_cli_id} B@{{stack}}"))
        .assert()
        .success()
        .stdout_eq(str![[r#"
Uncommitted [..] to [B]

"#]])
        .stderr_eq(str![""]);

    let after = status_json(&env)?;
    let commit_cli_ids_after = branch_commit_cli_ids(&after, "A");
    assert!(
        !commit_cli_ids_after.contains(&source_cli_id),
        "source commit should no longer be present in branch A after uncommit to stack"
    );
    assert!(
        stack_assigned_contains_file(&after, "B", "a.txt")
            && stack_assigned_contains_file(&after, "B", "b.txt"),
        "source commit files should be assigned to branch B stack"
    );

    Ok(())
}

#[test]
fn rub_matrix_branch_to_uncommitted_smoke() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);

    env.file("branch-to-zz.txt", "content\n");
    env.but("rub branch-to-zz.txt A")
        .assert()
        .success()
        .stdout_eq(str![[r#"
Staged the only hunk in branch-to-zz.txt in the uncommitted area → [A].

"#]])
        .stderr_eq(str![""]);

    env.but("rub A zz")
        .assert()
        .success()
        .stdout_eq(str![[r#"
Unstaged all [A] changes.

"#]])
        .stderr_eq(str![""]);

    let after = status_json(&env)?;
    assert!(
        uncommitted_contains_file(&after, "branch-to-zz.txt"),
        "file should move back to uncommitted"
    );

    Ok(())
}

#[test]
fn rub_matrix_branch_to_stack_smoke() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);

    env.file("branch-to-stack.txt", "content\n");
    env.but("rub branch-to-stack.txt A")
        .assert()
        .success()
        .stdout_eq(str![[r#"
Staged the only hunk in branch-to-stack.txt in the uncommitted area → [A].

"#]])
        .stderr_eq(str![""]);

    env.but("rub A B@{stack}")
        .assert()
        .success()
        .stdout_eq(str![[r#"
Staged all [A] changes to [B].

"#]])
        .stderr_eq(str![""]);

    let after = status_json(&env)?;
    assert!(
        stack_assigned_contains_file(&after, "B", "branch-to-stack.txt"),
        "file should be reassigned to B stack"
    );

    Ok(())
}

#[test]
fn rub_matrix_branch_to_commit_smoke() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);

    env.file("branch-to-commit.txt", "content\n");
    env.but("rub branch-to-commit.txt A")
        .assert()
        .success()
        .stdout_eq(str![[r#"
Staged the only hunk in branch-to-commit.txt in the uncommitted area → [A].

"#]])
        .stderr_eq(str![""]);

    let before = status_json(&env)?;
    let target_cli_id = branch_commit_cli_ids(&before, "A")[0].clone();

    env.but(format!("rub A {target_cli_id}"))
        .assert()
        .success()
        .stdout_eq(str![[r#"
Amended assigned files [A] → [..]

"#]])
        .stderr_eq(str![""]);

    let after = status_json(&env)?;
    assert!(
        branch_commits_contain_file(&after, "A", "branch-to-commit.txt"),
        "file should be amended into a commit on branch A"
    );

    Ok(())
}

#[test]
fn rub_matrix_branch_to_branch_smoke() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);

    env.file("branch-to-branch.txt", "content\n");
    env.but("rub branch-to-branch.txt A")
        .assert()
        .success()
        .stdout_eq(str![[r#"
Staged the only hunk in branch-to-branch.txt in the uncommitted area → [A].

"#]])
        .stderr_eq(str![""]);

    env.but("rub A B")
        .assert()
        .success()
        .stdout_eq(str![[r#"
Staged all [A] changes to [B].

"#]])
        .stderr_eq(str![""]);

    let after = status_json(&env)?;
    assert!(
        stack_assigned_contains_file(&after, "B", "branch-to-branch.txt"),
        "file should be reassigned to branch B"
    );

    Ok(())
}

#[test]
fn rub_matrix_stack_to_uncommitted_smoke() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);

    env.file("stack-to-zz.txt", "content\n");
    env.but("rub stack-to-zz.txt A")
        .assert()
        .success()
        .stdout_eq(str![[r#"
Staged the only hunk in stack-to-zz.txt in the uncommitted area → [A].

"#]])
        .stderr_eq(str![""]);

    env.but("rub A@{stack} zz")
        .assert()
        .success()
        .stdout_eq(str![[r#"
Unstaged all [A] changes.

"#]])
        .stderr_eq(str![""]);

    let after = status_json(&env)?;
    assert!(
        uncommitted_contains_file(&after, "stack-to-zz.txt"),
        "file should move back to uncommitted"
    );

    Ok(())
}

#[test]
fn rub_matrix_stack_to_stack_smoke() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);

    env.file("stack-to-stack.txt", "content\n");
    env.but("rub stack-to-stack.txt A")
        .assert()
        .success()
        .stdout_eq(str![[r#"
Staged the only hunk in stack-to-stack.txt in the uncommitted area → [A].

"#]])
        .stderr_eq(str![""]);

    env.but("rub A@{stack} B@{stack}")
        .assert()
        .success()
        .stdout_eq(str![[r#"
Staged all [A] changes to [B].

"#]])
        .stderr_eq(str![""]);

    let after = status_json(&env)?;
    assert!(
        stack_assigned_contains_file(&after, "B", "stack-to-stack.txt"),
        "file should be reassigned to B stack"
    );

    Ok(())
}

#[test]
fn rub_matrix_stack_to_branch_smoke() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);

    env.file("stack-to-branch.txt", "content\n");
    env.but("rub stack-to-branch.txt A")
        .assert()
        .success()
        .stdout_eq(str![[r#"
Staged the only hunk in stack-to-branch.txt in the uncommitted area → [A].

"#]])
        .stderr_eq(str![""]);

    env.but("rub A@{stack} B")
        .assert()
        .success()
        .stdout_eq(str![[r#"
Staged all [A] changes to [B].

"#]])
        .stderr_eq(str![""]);

    let after = status_json(&env)?;
    assert!(
        stack_assigned_contains_file(&after, "B", "stack-to-branch.txt"),
        "file should be reassigned to B branch"
    );

    Ok(())
}

#[test]
fn rub_matrix_stack_to_commit_smoke() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);

    env.file("stack-to-commit.txt", "content\n");
    env.but("rub stack-to-commit.txt A")
        .assert()
        .success()
        .stdout_eq(str![[r#"
Staged the only hunk in stack-to-commit.txt in the uncommitted area → [A].

"#]])
        .stderr_eq(str![""]);

    let before = status_json(&env)?;
    let target_cli_id = branch_commit_cli_ids(&before, "A")[0].clone();

    env.but(format!("rub A@{{stack}} {target_cli_id}"))
        .assert()
        .success()
        .stdout_eq(str![[r#"
Amended files assigned to [A] → [..]

"#]])
        .stderr_eq(str![""]);

    let after = status_json(&env)?;
    assert!(
        !stack_assigned_contains_file(&after, "A", "stack-to-commit.txt"),
        "file should no longer be assigned to stack A"
    );
    assert!(
        branch_commits_contain_file(&after, "A", "stack-to-commit.txt"),
        "file should be amended into a commit on branch A"
    );

    Ok(())
}

#[test]
fn rub_matrix_committed_file_to_branch_smoke() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);

    commit_two_files_as_two_hunks_each(&env, "A", "a.txt", "b.txt", "first commit");

    let before = status_json(&env)?;
    let source_cli_id = branch_commit_cli_ids(&before, "A")[0].clone();

    env.but(format!("rub {source_cli_id}:a.txt B"))
        .assert()
        .success()
        .stdout_eq(str![[r#"
Uncommitted changes

"#]])
        .stderr_eq(str![""]);

    let after = status_json(&env)?;
    assert!(
        stack_assigned_contains_file(&after, "B", "a.txt"),
        "file extracted from commit should be assigned to B"
    );

    Ok(())
}

#[test]
fn rub_matrix_invalid_pairs_smoke() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);

    commit_two_files_as_two_hunks_each(&env, "A", "a.txt", "b.txt", "invalid matrix setup");
    env.file("invalid-a.txt", "content\n");
    env.file("invalid-b.txt", "content\n");

    let status = status_json(&env)?;
    let commit_cli_id = branch_commit_cli_ids(&status, "A")[0].clone();

    env.but("rub invalid-a.txt invalid-b.txt")
        .assert()
        .failure()
        .stderr_eq(str![[r#"
Rubbed the wrong way. Operation doesn't make sense.[..]

"#]]);

    env.but("rub A invalid-a.txt")
        .assert()
        .failure()
        .stderr_eq(str![[r#"
Rubbed the wrong way. Operation doesn't make sense.[..]

"#]]);

    env.but("rub zz zz").assert().failure().stderr_eq(str![[r#"
Rubbed the wrong way. Operation doesn't make sense.[..]

"#]]);

    env.but(format!("rub {commit_cli_id}:a.txt A@{{stack}}"))
        .assert()
        .failure()
        .stderr_eq(str![[r#"
Rubbed the wrong way. Operation doesn't make sense.[..]

"#]]);

    Ok(())
}
