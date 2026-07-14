use crate::utils::Sandbox;

#[test]
fn can_show_by_duplicated_change_id() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("zero-stacks");
    env.setup_metadata(&[]);

    env.but("_commit2 -m first").assert().success();
    env.but("_commit2 -m second").assert().success();

    env.but("status")
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
╭┄zz [uncommitted] (no changes)
┊
┊╭┄br [a-branch-1]
┊●   1#0 e8564e1 second (no changes)
┊●   1#1 71c4380 first (no changes)
├╯
┊
┴ 0dc3733 (common base) 2000-01-02 add M

Hint: run `but help` for all commands

"#]]);

    env.but("show 1#0")
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Commit:    e8564e1938a8b7e00c0f3cf88d08f0687d6863d3
Change-ID: 1
Author:    author <author@example.com>
Date:      2000-01-02 00:00:00 +0000 (26y ago)
Committer: committer <committer@example.com>

second


"#]]);
    env.but("show 1#1")
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Commit:    71c4380695ab83fc7ff085860bb656ab27ed524e
Change-ID: 1
Author:    author <author@example.com>
Date:      2000-01-02 00:00:00 +0000 (26y ago)
Committer: committer <committer@example.com>

first


"#]]);
}

#[test]
fn can_show_by_distinct_change_id() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("zero-stacks");
    env.setup_metadata(&[]);

    set_change_id(&env, "123");
    env.but("_commit2 -m first").assert().success();

    set_change_id(&env, "132");
    env.but("_commit2 -m second").assert().success();

    env.but("status")
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
╭┄zz [uncommitted] (no changes)
┊
┊╭┄br [a-branch-1]
┊●   132 ea3b1e3 second (no changes)
┊●   123 a8954d4 first (no changes)
├╯
┊
┴ 0dc3733 (common base) 2000-01-02 add M

Hint: run `but help` for all commands

"#]]);

    env.but("show 12")
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Commit:    a8954d4c8daf2bc64ad7a62a33dbbad7a920bdb7
Change-ID: 123
Author:    author <author@example.com>
Date:      2000-01-02 00:00:00 +0000 (26y ago)
Committer: committer <committer@example.com>

first


"#]]);
    env.but("show 13")
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
Commit:    ea3b1e3ff9f628d463fc5d66a20a9d523fb9a95b
Change-ID: 132
Author:    author <author@example.com>
Date:      2000-01-02 00:00:00 +0000 (26y ago)
Committer: committer <committer@example.com>

second


"#]]);
}

fn set_change_id(env: &Sandbox, change_id: &str) {
    env.invoke_git(&format!(
        "config --local gitbutler.testing.changeId {change_id}"
    ));
}
