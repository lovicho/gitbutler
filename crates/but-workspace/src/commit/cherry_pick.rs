//! Cherry-pick commits into a workspace graph.

use anyhow::bail;
use but_core::{RefMetadata, commit::Headers};
use but_rebase::commit::DateMode;
use but_rebase::graph_rebase::{
    Editor, Selector, Step, SuccessfulRebase, ToCommitSelector, ToSelector as _,
    mutate::{InsertSide, RelativeTo},
};

/// Cherry-pick commits above or below a commit, or below a reference, in the
/// workspace graph.
///
/// Sources are deduplicated and ordered parent-first. Child commits, and the
/// target commit, if applicable, are rebased atop the cherry-picked commits.
pub fn cherry_pick_commits<'ws, 'meta, M: RefMetadata>(
    mut editor: Editor<'ws, 'meta, M>,
    source_commits: impl IntoIterator<Item: ToCommitSelector>,
    relative_to: RelativeTo,
    side: InsertSide,
) -> anyhow::Result<(SuccessfulRebase<'ws, 'meta, M>, Vec<Selector>)> {
    let mut source_commits = source_commits.into_iter().peekable();
    if source_commits.peek().is_none() {
        bail!("No commits were provided to cherry-pick")
    }
    if matches!(
        (&relative_to, side),
        (RelativeTo::Reference(_), InsertSide::Above)
    ) {
        bail!("Cannot cherry-pick above a reference")
    }

    let target = relative_to.to_selector(&editor)?;
    let ordered_selectors = editor.order_commit_selectors_by_parentage(source_commits)?;

    let mut inserted_selectors = Vec::with_capacity(ordered_selectors.len());
    let mut previous_selector = None;
    for source_selector in ordered_selectors {
        // Give the copy its own change ID, retaining all other metadata.
        let (_, mut template) = editor.find_selectable_commit(source_selector)?;
        let mut headers = Headers::try_from_commit(&template.inner).unwrap_or_default();
        headers.change_id = Headers::from_config(&editor.repo().config_snapshot()).change_id;
        headers.set_in_commit(&mut template.inner);
        let template_id = editor.new_commit(template, DateMode::CommitterUpdateAuthorKeep)?;

        let (anchor, insert_side) = match previous_selector {
            Some(selector) => (selector, InsertSide::Above),
            None => (target, side),
        };
        let selector = editor.insert(anchor, Step::new_untracked_pick(template_id), insert_side)?;
        inserted_selectors.push(selector);
        previous_selector = Some(selector);
    }

    Ok((editor.rebase()?, inserted_selectors))
}
