use crate::{
    command::{
        undo::run_mutate_undo_roundtrip_test,
        util::{branch_commit_cli_ids, commit_two_files_as_two_hunks_each, status_json},
    },
    utils::Sandbox,
};

// move2: commit to branch
#[test]
fn undo_move_commit_to_branch() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("two-stacks");
    env.setup_metadata(&["A", "B"]);
    commit_two_files_as_two_hunks_each(
        &env,
        "A",
        "commit-to-branch-a.txt",
        "commit-to-branch-b.txt",
        "first",
    );
    let source_cli_id = branch_commit_cli_ids(&status_json(&env).unwrap(), "A")[0].clone();

    run_mutate_undo_roundtrip_test(&env, |env| {
        env.but(format!("_move2 {source_cli_id} --branch B"))
            .assert()
            .success();
    });
}
