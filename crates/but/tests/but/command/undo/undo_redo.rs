use gitbutler_oplog::entry::OperationKind;

use crate::utils::Sandbox;

#[track_caller]
fn reword(env: &Sandbox, commit_before: &str, new_message: &str) -> (std::process::Output, String) {
    #[derive(serde::Deserialize)]
    struct RewordOutput {
        new_commit_id: String,
    }

    let reword_output = env
        .but("reword")
        .args([commit_before, "-m", new_message, "--format", "json"])
        .assert()
        .success();

    let reword_output =
        serde_json::from_slice::<RewordOutput>(&reword_output.get_output().stdout).unwrap();

    (
        env.but("status").output().unwrap(),
        reword_output.new_commit_id,
    )
}

#[track_caller]
fn undo(
    env: &Sandbox,
    operation_reverted_to: OperationKind,
    snapshot_restored_to: &str,
    expected_status: &std::process::Output,
) {
    env.but("undo").assert().success().stdout_eq(format!(
        r#"Undoing operation...
  Reverting to: {} (2000-01-02 00:00:00)
✓ Undo completed successfully! Restored to snapshot: {snapshot_restored_to}
"#,
        operation_reverted_to.title()
    ));

    env.but("status")
        .assert()
        .success()
        .stdout_eq(expected_status.stdout.clone())
        .stderr_eq(expected_status.stderr.clone());
}

#[track_caller]
fn redo(
    env: &Sandbox,
    operation_reverted_to: OperationKind,
    snapshot_restored_to: &str,
    expected_status: &std::process::Output,
) {
    env.but("redo").assert().success().stdout_eq(format!(
        r#"Redoing operation...
  Reverting to: {} (2000-01-02 00:00:00)
✓ Redo completed successfully! Restored to snapshot: {snapshot_restored_to}
"#,
        operation_reverted_to.title()
    ));

    env.but("status")
        .assert()
        .success()
        .stdout_eq(expected_status.stdout.clone())
        .stderr_eq(expected_status.stderr.clone());
}

#[track_caller]
fn restore(env: &Sandbox, operation_to_restore_to: &str, expected_status: &std::process::Output) {
    env.but("oplog")
        .args(["restore", operation_to_restore_to])
        .assert()
        .success()
        .stdout_eq(
            r#"
✓ Restore completed successfully!

Workspace has been restored to the selected snapshot.
"#,
        );
    env.but("status")
        .assert()
        .success()
        .stdout_eq(expected_status.stdout.clone())
        .stderr_eq(expected_status.stderr.clone());
}

#[test]
fn can_undo_repeatedly() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack-two-commits");
    env.setup_metadata(&["A"]);

    let (status_one, new_commit) = reword(&env, "9ac4652", "one");
    let (status_two, new_commit) = reword(&env, &new_commit, "two");
    let (status_three, new_commit) = reword(&env, &new_commit, "three");
    reword(&env, &new_commit, "four");

    env.but("oplog")
        .args(["list"])
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Operations History
──────────────────────────────────────────────────
5129db9 2000-01-02 00:00:00 [REWORD] Updated commit message
da67dd1 2000-01-02 00:00:00 [REWORD] Updated commit message
e23e4fa 2000-01-02 00:00:00 [REWORD] Updated commit message
7665ea7 2000-01-02 00:00:00 [REWORD] Updated commit message

"#]]);

    undo(
        &env,
        OperationKind::UpdateCommitMessage,
        "5129db9",
        &status_three,
    );

    env.but("oplog")
        .args(["list"])
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Operations History
──────────────────────────────────────────────────
94d85c5 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (5129db9)
5129db9 2000-01-02 00:00:00 [REWORD] Updated commit message
da67dd1 2000-01-02 00:00:00 [REWORD] Updated commit message
e23e4fa 2000-01-02 00:00:00 [REWORD] Updated commit message
7665ea7 2000-01-02 00:00:00 [REWORD] Updated commit message

"#]]);

    undo(
        &env,
        OperationKind::UpdateCommitMessage,
        "da67dd1",
        &status_two,
    );

    env.but("oplog")
        .args(["list"])
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Operations History
──────────────────────────────────────────────────
5e5fe67 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (da67dd1)
94d85c5 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (5129db9)
5129db9 2000-01-02 00:00:00 [REWORD] Updated commit message
da67dd1 2000-01-02 00:00:00 [REWORD] Updated commit message
e23e4fa 2000-01-02 00:00:00 [REWORD] Updated commit message
7665ea7 2000-01-02 00:00:00 [REWORD] Updated commit message

"#]]);

    undo(
        &env,
        OperationKind::UpdateCommitMessage,
        "e23e4fa",
        &status_one,
    );

    env.but("oplog")
        .args(["list"])
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Operations History
──────────────────────────────────────────────────
0d7b714 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (e23e4fa)
5e5fe67 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (da67dd1)
94d85c5 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (5129db9)
5129db9 2000-01-02 00:00:00 [REWORD] Updated commit message
da67dd1 2000-01-02 00:00:00 [REWORD] Updated commit message
e23e4fa 2000-01-02 00:00:00 [REWORD] Updated commit message
7665ea7 2000-01-02 00:00:00 [REWORD] Updated commit message

"#]]);
}

#[test]
fn can_undo_explicit_restore() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack-two-commits");
    env.setup_metadata(&["A"]);

    let (_, new_commit) = reword(&env, "9ac4652", "one");
    let (status_two, new_commit) = reword(&env, &new_commit, "two");
    let (_, new_commit) = reword(&env, &new_commit, "three");
    let (status_four, _) = reword(&env, &new_commit, "four");

    env.but("oplog")
        .args(["list"])
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Operations History
──────────────────────────────────────────────────
5129db9 2000-01-02 00:00:00 [REWORD] Updated commit message
da67dd1 2000-01-02 00:00:00 [REWORD] Updated commit message
e23e4fa 2000-01-02 00:00:00 [REWORD] Updated commit message
7665ea7 2000-01-02 00:00:00 [REWORD] Updated commit message

"#]]);

    restore(&env, "da67dd1", &status_two);

    env.but("oplog")
        .args(["list"])
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Operations History
──────────────────────────────────────────────────
8b4f09e 2000-01-02 00:00:00 [RESTORE] Restored from snapshot: Updated commit message (da67dd1)
5129db9 2000-01-02 00:00:00 [REWORD] Updated commit message
da67dd1 2000-01-02 00:00:00 [REWORD] Updated commit message
e23e4fa 2000-01-02 00:00:00 [REWORD] Updated commit message
7665ea7 2000-01-02 00:00:00 [REWORD] Updated commit message

"#]]);

    undo(
        &env,
        OperationKind::RestoreFromSnapshot,
        "8b4f09e",
        &status_four,
    );

    env.but("oplog")
        .args(["list"])
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Operations History
──────────────────────────────────────────────────
2c3576c 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (da67dd1)
8b4f09e 2000-01-02 00:00:00 [RESTORE] Restored from snapshot: Updated commit message (da67dd1)
5129db9 2000-01-02 00:00:00 [REWORD] Updated commit message
da67dd1 2000-01-02 00:00:00 [REWORD] Updated commit message
e23e4fa 2000-01-02 00:00:00 [REWORD] Updated commit message
7665ea7 2000-01-02 00:00:00 [REWORD] Updated commit message

"#]]);
}

#[test]
fn can_undo_perform_operation_then_undo_again() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack-two-commits");
    env.setup_metadata(&["A"]);

    let (_, new_commit) = reword(&env, "9ac4652", "one");
    let (status_two, new_commit) = reword(&env, &new_commit, "two");
    let (status_three, new_commit) = reword(&env, &new_commit, "three");
    reword(&env, &new_commit, "four");

    env.but("oplog")
        .args(["list"])
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Operations History
──────────────────────────────────────────────────
5129db9 2000-01-02 00:00:00 [REWORD] Updated commit message
da67dd1 2000-01-02 00:00:00 [REWORD] Updated commit message
e23e4fa 2000-01-02 00:00:00 [REWORD] Updated commit message
7665ea7 2000-01-02 00:00:00 [REWORD] Updated commit message

"#]]);

    undo(
        &env,
        OperationKind::UpdateCommitMessage,
        "5129db9",
        &status_three,
    );

    reword(&env, &new_commit, "three-new");

    env.but("oplog")
        .args(["list"])
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Operations History
──────────────────────────────────────────────────
22e0417 2000-01-02 00:00:00 [REWORD] Updated commit message
94d85c5 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (5129db9)
5129db9 2000-01-02 00:00:00 [REWORD] Updated commit message
da67dd1 2000-01-02 00:00:00 [REWORD] Updated commit message
e23e4fa 2000-01-02 00:00:00 [REWORD] Updated commit message
7665ea7 2000-01-02 00:00:00 [REWORD] Updated commit message

"#]]);

    undo(
        &env,
        OperationKind::UpdateCommitMessage,
        "22e0417",
        &status_three,
    );

    env.but("oplog")
        .args(["list"])
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Operations History
──────────────────────────────────────────────────
aac925c 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (22e0417)
22e0417 2000-01-02 00:00:00 [REWORD] Updated commit message
94d85c5 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (5129db9)
5129db9 2000-01-02 00:00:00 [REWORD] Updated commit message
da67dd1 2000-01-02 00:00:00 [REWORD] Updated commit message
e23e4fa 2000-01-02 00:00:00 [REWORD] Updated commit message
7665ea7 2000-01-02 00:00:00 [REWORD] Updated commit message

"#]]);

    undo(
        &env,
        OperationKind::UpdateCommitMessage,
        "da67dd1",
        &status_two,
    );
}

#[test]
fn undoing_past_end_of_oplog() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack-two-commits");
    env.setup_metadata(&["A"]);

    let status_zero = env.but("status").output().unwrap();
    let (status_one, new_commit) = reword(&env, "9ac4652", "one");
    reword(&env, &new_commit, "two");

    env.but("oplog")
        .args(["list"])
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Operations History
──────────────────────────────────────────────────
e23e4fa 2000-01-02 00:00:00 [REWORD] Updated commit message
7665ea7 2000-01-02 00:00:00 [REWORD] Updated commit message

"#]]);

    undo(
        &env,
        OperationKind::UpdateCommitMessage,
        "e23e4fa",
        &status_one,
    );

    env.but("oplog")
        .args(["list"])
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Operations History
──────────────────────────────────────────────────
aecc993 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (e23e4fa)
e23e4fa 2000-01-02 00:00:00 [REWORD] Updated commit message
7665ea7 2000-01-02 00:00:00 [REWORD] Updated commit message

"#]]);

    undo(
        &env,
        OperationKind::UpdateCommitMessage,
        "7665ea7",
        &status_zero,
    );

    env.but("oplog")
        .args(["list"])
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Operations History
──────────────────────────────────────────────────
f47dc34 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (7665ea7)
aecc993 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (e23e4fa)
e23e4fa 2000-01-02 00:00:00 [REWORD] Updated commit message
7665ea7 2000-01-02 00:00:00 [REWORD] Updated commit message

"#]]);

    env.but("undo").assert().success().stdout_eq(
        r#"No previous operations to undo.
"#,
    );
}

#[test]
fn can_redo() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack-two-commits");
    env.setup_metadata(&["A"]);

    let (_, new_commit) = reword(&env, "9ac4652", "one");
    let (_, new_commit) = reword(&env, &new_commit, "two");
    let (status_three, new_commit) = reword(&env, &new_commit, "three");
    let (status_four, _) = reword(&env, &new_commit, "four");

    env.but("oplog")
        .args(["list"])
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Operations History
──────────────────────────────────────────────────
5129db9 2000-01-02 00:00:00 [REWORD] Updated commit message
da67dd1 2000-01-02 00:00:00 [REWORD] Updated commit message
e23e4fa 2000-01-02 00:00:00 [REWORD] Updated commit message
7665ea7 2000-01-02 00:00:00 [REWORD] Updated commit message

"#]]);

    undo(
        &env,
        OperationKind::UpdateCommitMessage,
        "5129db9",
        &status_three,
    );

    env.but("oplog")
        .args(["list"])
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Operations History
──────────────────────────────────────────────────
94d85c5 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (5129db9)
5129db9 2000-01-02 00:00:00 [REWORD] Updated commit message
da67dd1 2000-01-02 00:00:00 [REWORD] Updated commit message
e23e4fa 2000-01-02 00:00:00 [REWORD] Updated commit message
7665ea7 2000-01-02 00:00:00 [REWORD] Updated commit message

"#]]);

    redo(
        &env,
        OperationKind::RestoreFromSnapshotViaUndo,
        "94d85c5",
        &status_four,
    );

    env.but("oplog")
        .args(["list"])
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Operations History
──────────────────────────────────────────────────
54c81c6 2000-01-02 00:00:00 [REDO] Restored from snapshot: Updated commit message (5129db9)
94d85c5 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (5129db9)
5129db9 2000-01-02 00:00:00 [REWORD] Updated commit message
da67dd1 2000-01-02 00:00:00 [REWORD] Updated commit message
e23e4fa 2000-01-02 00:00:00 [REWORD] Updated commit message
7665ea7 2000-01-02 00:00:00 [REWORD] Updated commit message

"#]]);

    env.but("redo").assert().success().stdout_eq(
        r#"No previous undo to redo.
"#,
    );
}

#[test]
fn can_mix_undo_and_redo() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack-two-commits");
    env.setup_metadata(&["A"]);

    let (status_one, new_commit) = reword(&env, "9ac4652", "one");
    let (status_two, new_commit) = reword(&env, &new_commit, "two");
    let (status_three, new_commit) = reword(&env, &new_commit, "three");
    let (status_four, _) = reword(&env, &new_commit, "four");

    env.but("oplog")
        .args(["list"])
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Operations History
──────────────────────────────────────────────────
5129db9 2000-01-02 00:00:00 [REWORD] Updated commit message
da67dd1 2000-01-02 00:00:00 [REWORD] Updated commit message
e23e4fa 2000-01-02 00:00:00 [REWORD] Updated commit message
7665ea7 2000-01-02 00:00:00 [REWORD] Updated commit message

"#]]);

    undo(
        &env,
        OperationKind::UpdateCommitMessage,
        "5129db9",
        &status_three,
    );
    undo(
        &env,
        OperationKind::UpdateCommitMessage,
        "da67dd1",
        &status_two,
    );

    env.but("oplog")
        .args(["list"])
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Operations History
──────────────────────────────────────────────────
5e5fe67 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (da67dd1)
94d85c5 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (5129db9)
5129db9 2000-01-02 00:00:00 [REWORD] Updated commit message
da67dd1 2000-01-02 00:00:00 [REWORD] Updated commit message
e23e4fa 2000-01-02 00:00:00 [REWORD] Updated commit message
7665ea7 2000-01-02 00:00:00 [REWORD] Updated commit message

"#]]);

    redo(
        &env,
        OperationKind::RestoreFromSnapshotViaUndo,
        "5e5fe67",
        &status_three,
    );

    env.but("oplog")
        .args(["list"])
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Operations History
──────────────────────────────────────────────────
82f0c86 2000-01-02 00:00:00 [REDO] Restored from snapshot: Updated commit message (da67dd1)
5e5fe67 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (da67dd1)
94d85c5 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (5129db9)
5129db9 2000-01-02 00:00:00 [REWORD] Updated commit message
da67dd1 2000-01-02 00:00:00 [REWORD] Updated commit message
e23e4fa 2000-01-02 00:00:00 [REWORD] Updated commit message
7665ea7 2000-01-02 00:00:00 [REWORD] Updated commit message

"#]]);

    undo(
        &env,
        OperationKind::RestoreFromSnapshotViaRedo,
        "82f0c86",
        &status_two,
    );

    env.but("oplog")
        .args(["list"])
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Operations History
──────────────────────────────────────────────────
5e83f80 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (da67dd1)
82f0c86 2000-01-02 00:00:00 [REDO] Restored from snapshot: Updated commit message (da67dd1)
5e5fe67 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (da67dd1)
94d85c5 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (5129db9)
5129db9 2000-01-02 00:00:00 [REWORD] Updated commit message
da67dd1 2000-01-02 00:00:00 [REWORD] Updated commit message
e23e4fa 2000-01-02 00:00:00 [REWORD] Updated commit message
7665ea7 2000-01-02 00:00:00 [REWORD] Updated commit message

"#]]);

    undo(
        &env,
        OperationKind::UpdateCommitMessage,
        "e23e4fa",
        &status_one,
    );

    env.but("oplog")
        .args(["list"])
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Operations History
──────────────────────────────────────────────────
d778c06 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (e23e4fa)
5e83f80 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (da67dd1)
82f0c86 2000-01-02 00:00:00 [REDO] Restored from snapshot: Updated commit message (da67dd1)
5e5fe67 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (da67dd1)
94d85c5 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (5129db9)
5129db9 2000-01-02 00:00:00 [REWORD] Updated commit message
da67dd1 2000-01-02 00:00:00 [REWORD] Updated commit message
e23e4fa 2000-01-02 00:00:00 [REWORD] Updated commit message
7665ea7 2000-01-02 00:00:00 [REWORD] Updated commit message

"#]]);

    redo(
        &env,
        OperationKind::RestoreFromSnapshotViaUndo,
        "d778c06",
        &status_two,
    );

    env.but("oplog")
        .args(["list"])
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Operations History
──────────────────────────────────────────────────
2acbef2 2000-01-02 00:00:00 [REDO] Restored from snapshot: Updated commit message (e23e4fa)
d778c06 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (e23e4fa)
5e83f80 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (da67dd1)
82f0c86 2000-01-02 00:00:00 [REDO] Restored from snapshot: Updated commit message (da67dd1)
5e5fe67 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (da67dd1)
94d85c5 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (5129db9)
5129db9 2000-01-02 00:00:00 [REWORD] Updated commit message
da67dd1 2000-01-02 00:00:00 [REWORD] Updated commit message
e23e4fa 2000-01-02 00:00:00 [REWORD] Updated commit message
7665ea7 2000-01-02 00:00:00 [REWORD] Updated commit message

"#]]);

    redo(
        &env,
        OperationKind::RestoreFromSnapshotViaUndo,
        "5e83f80",
        &status_three,
    );

    env.but("oplog")
        .args(["list"])
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Operations History
──────────────────────────────────────────────────
41af71d 2000-01-02 00:00:00 [REDO] Restored from snapshot: Updated commit message (da67dd1)
2acbef2 2000-01-02 00:00:00 [REDO] Restored from snapshot: Updated commit message (e23e4fa)
d778c06 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (e23e4fa)
5e83f80 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (da67dd1)
82f0c86 2000-01-02 00:00:00 [REDO] Restored from snapshot: Updated commit message (da67dd1)
5e5fe67 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (da67dd1)
94d85c5 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (5129db9)
5129db9 2000-01-02 00:00:00 [REWORD] Updated commit message
da67dd1 2000-01-02 00:00:00 [REWORD] Updated commit message
e23e4fa 2000-01-02 00:00:00 [REWORD] Updated commit message
7665ea7 2000-01-02 00:00:00 [REWORD] Updated commit message

"#]]);

    redo(
        &env,
        OperationKind::RestoreFromSnapshotViaUndo,
        "94d85c5",
        &status_four,
    );

    env.but("oplog")
        .args(["list"])
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Operations History
──────────────────────────────────────────────────
71a02c9 2000-01-02 00:00:00 [REDO] Restored from snapshot: Updated commit message (5129db9)
41af71d 2000-01-02 00:00:00 [REDO] Restored from snapshot: Updated commit message (da67dd1)
2acbef2 2000-01-02 00:00:00 [REDO] Restored from snapshot: Updated commit message (e23e4fa)
d778c06 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (e23e4fa)
5e83f80 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (da67dd1)
82f0c86 2000-01-02 00:00:00 [REDO] Restored from snapshot: Updated commit message (da67dd1)
5e5fe67 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (da67dd1)
94d85c5 2000-01-02 00:00:00 [UNDO] Restored from snapshot: Updated commit message (5129db9)
5129db9 2000-01-02 00:00:00 [REWORD] Updated commit message
da67dd1 2000-01-02 00:00:00 [REWORD] Updated commit message
e23e4fa 2000-01-02 00:00:00 [REWORD] Updated commit message
7665ea7 2000-01-02 00:00:00 [REWORD] Updated commit message

"#]]);
}

#[test]
fn cannot_redo_without_undoing_first() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack-two-commits");
    env.setup_metadata(&["A"]);

    reword(&env, "9ac4652", "one");

    env.but("redo").assert().success().stdout_eq(
        r#"No previous undo to redo.
"#,
    );
}
