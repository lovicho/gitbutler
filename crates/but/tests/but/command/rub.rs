use snapbox::str;

use crate::{
    command::util::{
        branch_commit_cli_ids, commit_two_files_as_two_hunks_each,
        status_json_with_files as status_json,
    },
    utils::Sandbox,
};

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
