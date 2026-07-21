use crate::utils::Sandbox;

fn setup_multi_hunk_uncommitted_changes(path: &str) -> Sandbox {
    let env = Sandbox::init_scenario_with_target_and_default_settings("zero-stacks");
    env.setup_metadata(&["A"]);

    let original_content = "this\nis\nsome\ncontent\nto\ndiff\nwith\nadded\nlines\n";
    env.file(path, original_content);
    env.but("commit -m 'Add file'")
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Created new independent branch 'a-branch-1'
✓ Created commit 1 on branch a-branch-1

"#]]);

    env.file(path, format!("new first\n{original_content}new last"));

    env
}

#[test]
fn open_uncommitted_file_with_() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("zero-stacks");
    env.setup_metadata(&["A"]);

    env.file("new-file.txt", "content");

    env.but("status")
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
╭┄zz [uncommitted]
┊   xk A new-file.txt
┊
┴ 0dc3733 (common base) 2000-01-02 add M

Hint: run `but branch new` to create a new branch to work on

"#]]);

    env.but("_open xk -p echo")
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
filepath='/[..]/new-file.txt'

"#]]);
}

#[test]
fn open_uncommitted_hunk() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("zero-stacks");
    env.setup_metadata(&["A"]);

    let original_content = "this\nis\nsome\ncontent\nto\ndiff\nwith\nadded\nlines\n";
    env.file("file-with-additions.txt", original_content);
    env.file("file-with-deletions.txt", original_content);
    env.file("file-with-mixed.txt", original_content);
    env.but("commit -m 'Add files'").assert().success();

    env.file(
        "file-with-additions.txt",
        format!("new first\n{original_content}new last"),
    );
    env.file(
        "file-with-deletions.txt",
        "is\nsome\ncontent\nto\ndiff\nwith\nadded\n",
    );
    env.file(
        "file-with-mixed.txt",
        "this\nIS\nsome\ncontent\nto\ndiff\nwith\nADDED\n",
    );

    env.but("diff")
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
────────────────────────────╮
rn:7 file-with-additions.txt│
────────────────────────────╯
     1│+new first
   1 2│ this
   2 3│ is
   3 4│ some
────────────────────────────╮
rn:4 file-with-additions.txt│
────────────────────────────╯
    7  8│ with
    8  9│ added
    9 10│ lines
      11│+new last
────────────────────────────╮
rw:b file-with-deletions.txt│
────────────────────────────╯
   1  │-this
   2 1│ is
   3 2│ some
   4 3│ content
────────────────────────────╮
rw:6 file-with-deletions.txt│
────────────────────────────╯
    6  5│ diff
    7  6│ with
    8  7│ added
    9   │-lines
────────────────────────╮
lp:6 file-with-mixed.txt│
────────────────────────╯
    1  1│ this
    2   │-is
       2│+IS
    3  3│ some
    4  4│ content
    5  5│ to
    6  6│ diff
    7  7│ with
    8   │-added
    9   │-lines
       8│+ADDED

"#]]);

    env.but("_open rn:7 -p echo ")
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
filepath='/[..]/file-with-additions.txt' line_number='1'

"#]]);
    env.but("_open rn:4 -p echo ")
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
filepath='/[..]/file-with-additions.txt' line_number='11'

"#]]);
    env.but("_open rw:b -p echo ")
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
filepath='/[..]/file-with-deletions.txt' line_number='1'

"#]]);
    env.but("_open rw:6 -p echo ")
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
filepath='/[..]/file-with-deletions.txt' line_number='7'

"#]]);
    env.but("_open lp:6 -p echo ")
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
filepath='/[..]/file-with-mixed.txt' line_number='2'

"#]]);
}

#[test]
fn open_uncommitted_hunk_in_file_that_contains_spaces_and_shell_metacharacters() {
    let env = setup_multi_hunk_uncommitted_changes(
        "file with some $meta; cat A > new-file.txt; spaces/in it.txt",
    );

    env.but("status").assert().success().stdout_eq(snapbox::str![[r#"
╭┄zz [uncommitted]
┊   pr M file with some $meta; cat A > new-file.txt; spaces/in it.txt
┊
┊╭┄br [a-branch-1]
┊●   1 Add file
├╯
┊
┴ 0dc3733 (common base) 2000-01-02 add M

Hint: run `but diff` to see uncommitted changes and `but commit <branch> -m "message" --changes <id>` to commit them

"#]]);

    env.but("_open pr:4 -p echo")
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
filepath='/[..]/file with some $meta; cat A > new-file.txt; spaces/in it.txt' line_number='11'

"#]]);
}

#[test]
fn cannot_open_non_existing_cli_id() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("zero-stacks");

    env.but("_open notexist -p echo")
        .assert()
        .failure()
        .stderr_eq(snapbox::str![[r#"
Error: Could not find uncommitted change: 'notexist'

Hint: Run `but status` for applicable targets.

"#]]);
}

#[test]
fn cannot_open_committed_changes() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack");

    env.but("status -f")
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
╭┄zz [uncommitted] (no changes)
┊
┊╭┄g0 [A]
┊●   tpm add A
┊│     tpm:t A A
├╯
┊
┴ 0dc3733 (common base) 2000-01-02 add M

Hint: run `but help` for all commands

"#]]);

    env.but("_open A -p echo")
        .assert()
        .failure()
        .stderr_eq(snapbox::str![[r#"
Error: Expected uncommitted file or hunk, got a branch

"#]]);

    env.but("_open tpm -p echo")
        .assert()
        .failure()
        .stderr_eq(snapbox::str![[r#"
Error: Expected uncommitted file or hunk, got a commit

"#]]);

    env.but("_open tpm:t -p echo")
        .assert()
        .failure()
        .stderr_eq(snapbox::str![[r#"
Error: Expected uncommitted file or hunk, got a committed file

"#]]);
}
