use snapbox::str;

use crate::utils::{CommandExt as _, Sandbox};

fn expand_env() -> Sandbox {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack");
    env.setup_metadata(&["A"]);
    env
}

#[test]
fn resolves_cli_id_atom() {
    let env = expand_env();

    env.but("_expand A")
        .assert()
        .success()
        .stderr_eq(str![])
        .stdout_eq(str![[r#"
Matches: 1

branch: g0 A

"#]]);

    env.but("_expand zz")
        .assert()
        .success()
        .stderr_eq(str![])
        .stdout_eq(str![[r#"
Matches: 1

uncommitted area

"#]]);
}

#[test]
fn reports_no_matches() {
    let env = expand_env();

    env.but("_expand does-not-exist")
        .assert()
        .success()
        .stderr_eq(str![])
        .stdout_eq(str![[r#"
Matches: 0


"#]]);
}

#[cfg(feature = "legacy")]
#[test]
fn resolves_duplicated_change_ids() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("zero-stacks");
    env.setup_metadata(&[]);

    env.but("_commit2 -m first").assert().success();
    env.but("_commit2 -m second").assert().success();

    env.but("_expand 1#0")
        .assert()
        .success()
        .stdout_eq(str![[r#"
Matches: 1

commit: 1 e8564e1938a8b7e00c0f3cf88d08f0687d6863d3

"#]]);
    env.but("_expand 1#1")
        .assert()
        .success()
        .stdout_eq(str![[r#"
Matches: 1

commit: 1 71c4380695ab83fc7ff085860bb656ab27ed524e

"#]]);
}

#[cfg(feature = "legacy")]
#[test]
fn resolves_distinct_change_id_prefixes() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("zero-stacks");
    env.setup_metadata(&[]);

    set_change_id(&env, "123");
    env.but("_commit2 -m first").assert().success();

    set_change_id(&env, "132");
    env.but("_commit2 -m second").assert().success();

    env.but("_expand 12").assert().success().stdout_eq(str![[r#"
Matches: 1

commit: 123 a8954d4c8daf2bc64ad7a62a33dbbad7a920bdb7

"#]]);
    env.but("_expand 13").assert().success().stdout_eq(str![[r#"
Matches: 1

commit: 132 ea3b1e3ff9f628d463fc5d66a20a9d523fb9a95b

"#]]);
}

#[cfg(feature = "legacy")]
fn set_change_id(env: &Sandbox, change_id: &str) {
    env.invoke_git(&format!(
        "config --local gitbutler.testing.changeId {change_id}"
    ));
}

#[test]
fn supports_json_output() {
    let env = expand_env();

    env.but("--format json _expand zz")
        .allow_json()
        .assert()
        .success()
        .stderr_eq(str![])
        .stdout_eq(str![[r#"
{
  "matches": 1,
  "resources": [
    {
      "type": "uncommitted"
    }
  ]
}

"#]]);
}

#[test]
fn requires_exactly_one_argument() {
    let env = Sandbox::empty();

    env.but("_expand").assert().failure();
    env.but("_expand zz extra").assert().failure();
}
