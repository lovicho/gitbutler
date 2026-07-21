use anyhow::Result;
use but_api::legacy::forge::{forge_info, forge_provider};
use but_forge::ForgeName;
use but_testsupport::{CommandExt, git_at_dir, open_repo};

#[test]
fn forge_state_follows_target_initialization() -> Result<()> {
    let (repo, tmp) = crate::support::repo_with_feature_branch()?;
    drop(repo);
    git_at_dir(tmp.path())
        .args(["branch", "-D", "feature"])
        .run();
    git_at_dir(tmp.path())
        .args([
            "config",
            "remote.origin.url",
            "git@gitlab.example.com:acme/widgets.git",
        ])
        .run();
    git_at_dir(tmp.path())
        .args([
            "config",
            "remote.fork.url",
            "https://github.com/acme/widgets.git",
        ])
        .run();

    let mut ctx =
        but_ctx::Context::from_repo_for_testing(open_repo(tmp.path())?)?.with_memory_app_cache();
    assert!(
        forge_info(&ctx)?.is_none(),
        "configured remotes are ambiguous until the single-branch project has a target"
    );
    assert!(
        forge_provider(&ctx)?.is_none(),
        "the provider follows the same targetless state"
    );

    let target_ref = gix::refs::FullName::try_from("refs/remotes/origin/main")?;
    but_api::workspace::set_target_ref_and_init_project(&mut ctx, target_ref.as_ref(), None)?;
    assert_eq!(
        forge_provider(&ctx)?,
        Some(ForgeName::GitLab),
        "the target remote determines the forge"
    );

    let existing_ctx =
        but_ctx::Context::from_repo_for_testing(open_repo(tmp.path())?)?.with_memory_app_cache();
    assert_eq!(
        forge_info(&existing_ctx)?.map(|info| info.name),
        Some(ForgeName::GitLab),
        "persisted target metadata remains available after reopening the project"
    );
    Ok(())
}
