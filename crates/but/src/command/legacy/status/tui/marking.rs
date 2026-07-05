use itertools::Either;
use nonempty::NonEmpty;
use strum::VariantArray;

use crate::{
    CliId,
    id::{ShortId, UncommittedHunkOrFile},
};

#[derive(Default, Debug, Clone, PartialEq)]
pub enum Marks {
    #[default]
    Empty,
    Hunks(NonEmpty<UncommittedHunkOrFile>),
    Commits(NonEmpty<MarkedCommit>),
}

impl Marks {
    pub fn toggle(&mut self, markable: Markable) -> anyhow::Result<()> {
        if self.contains(markable.as_ref()) {
            self.remove(markable.as_ref());
        } else {
            self.insert(markable)?;
        }
        Ok(())
    }

    pub fn insert(&mut self, markable: Markable) -> anyhow::Result<()> {
        match markable {
            Markable::Uncommitted(hunk) => {
                if let Self::Empty = self {
                    *self = Self::Hunks(NonEmpty::new(hunk));
                } else if let Self::Hunks(hunks) = self {
                    hunks.push(hunk);
                } else {
                    anyhow::bail!("cannot mix mark sources")
                }
            }
            Markable::Commit(commit) => {
                if let Self::Empty = self {
                    *self = Self::Commits(NonEmpty::new(MarkedCommit {
                        commit_id: commit.commit_id,
                        id: commit.id.clone(),
                    }));
                } else if let Self::Commits(commits) = self {
                    commits.push(MarkedCommit {
                        commit_id: commit.commit_id,
                        id: commit.id.clone(),
                    });
                } else {
                    anyhow::bail!("cannot mix mark sources")
                }
            }
        }
        Ok(())
    }

    pub fn remove(&mut self, markable: MarkableRef<'_>) {
        let is_empty = match self {
            Marks::Empty => false,
            Marks::Hunks(hunks) => {
                if let MarkableRef::Uncommitted(hunk) = markable {
                    remove_from_non_empty(hunks, |marked| marked == hunk)
                } else {
                    false
                }
            }
            Marks::Commits(commits) => {
                if let MarkableRef::Commit(commit) = markable {
                    remove_from_non_empty(commits, |marked| marked.commit_id == commit.commit_id)
                } else {
                    false
                }
            }
        };

        if is_empty {
            *self = Self::Empty;
        }
    }

    pub fn clear(&mut self) {
        *self = Self::Empty;
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    pub fn len(&self) -> usize {
        match self {
            Marks::Empty => 0,
            Marks::Hunks(hunks) => hunks.len(),
            Marks::Commits(commits) => commits.len(),
        }
    }

    pub fn contains(&self, markable: MarkableRef<'_>) -> bool {
        self.iter().any(|mark| mark == markable)
    }

    pub fn contains_cli_id(&self, cli_id: &CliId) -> bool {
        self.iter().any(|mark| mark.matches_cli_id(cli_id))
    }

    pub fn iter(&self) -> impl Iterator<Item = MarkableRef<'_>> {
        match self {
            Marks::Empty => Either::Left(Either::Left(std::iter::empty())),
            Marks::Hunks(hunks) => {
                Either::Left(Either::Right(hunks.iter().map(MarkableRef::Uncommitted)))
            }
            Marks::Commits(commits) => {
                Either::Right(commits.iter().map(|MarkedCommit { commit_id, id }| {
                    MarkableRef::Commit(MarkedCommitRef {
                        commit_id: *commit_id,
                        id,
                    })
                }))
            }
        }
    }

    pub fn marked_commits(&self) -> bool {
        match self {
            Marks::Empty | Marks::Hunks(..) => false,
            Marks::Commits(..) => true,
        }
    }

    pub fn marked_uncommitted(&self) -> bool {
        match self {
            Marks::Commits(..) | Marks::Empty => false,
            Marks::Hunks(..) => true,
        }
    }
}

fn remove_from_non_empty<T>(items: &mut NonEmpty<T>, predicate: impl Fn(&T) -> bool) -> bool {
    let Some(index) = items.iter().position(predicate) else {
        return false;
    };

    if index == 0 {
        if items.tail.is_empty() {
            true
        } else {
            items.head = items.tail.remove(0);
            false
        }
    } else {
        items.tail.remove(index - 1);
        false
    }
}

#[derive(Debug, Clone, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::VariantArray))]
pub enum Markable {
    Uncommitted(UncommittedHunkOrFile),
    Commit(MarkedCommit),
}

impl Markable {
    pub fn try_from_cli_id(cli_id: CliId) -> Option<Self> {
        for variant in MarkableDiscriminants::VARIANTS {
            match variant {
                MarkableDiscriminants::Uncommitted => {
                    if let CliId::UncommittedHunkOrFile(uncommitted) = &cli_id {
                        if uncommitted
                            .hunk_assignments
                            .iter()
                            .any(|hunk| hunk.stack_id.is_some())
                        {
                            return None;
                        }
                        return Some(Self::Uncommitted(uncommitted.clone()));
                    }
                }
                MarkableDiscriminants::Commit => {
                    if let CliId::Commit { commit_id, id } = &cli_id {
                        return Some(Self::Commit(MarkedCommit {
                            commit_id: *commit_id,
                            id: id.clone(),
                        }));
                    }
                }
            }
        }

        None
    }

    pub fn into_cli_id(self) -> CliId {
        match self {
            Markable::Uncommitted(uncommitted_cli_id) => {
                CliId::UncommittedHunkOrFile(uncommitted_cli_id)
            }
            Markable::Commit(MarkedCommit { commit_id, id }) => CliId::Commit { commit_id, id },
        }
    }

    pub fn as_ref(&self) -> MarkableRef<'_> {
        match self {
            Markable::Uncommitted(hunk) => MarkableRef::Uncommitted(hunk),
            Markable::Commit(MarkedCommit { commit_id, id }) => {
                MarkableRef::Commit(MarkedCommitRef {
                    commit_id: *commit_id,
                    id,
                })
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarkedCommit {
    pub commit_id: gix::ObjectId,
    pub id: ShortId,
}

#[derive(Debug, Clone, Copy, PartialEq, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::VariantArray))]
pub enum MarkableRef<'a> {
    Uncommitted(&'a UncommittedHunkOrFile),
    Commit(MarkedCommitRef<'a>),
}

impl<'a> MarkableRef<'a> {
    pub fn try_from_cli_id(cli_id: &'a CliId) -> Option<Self> {
        for variant in MarkableRefDiscriminants::VARIANTS {
            match variant {
                MarkableRefDiscriminants::Uncommitted => {
                    if let CliId::UncommittedHunkOrFile(uncommitted) = cli_id {
                        if uncommitted
                            .hunk_assignments
                            .iter()
                            .any(|hunk| hunk.stack_id.is_some())
                        {
                            return None;
                        }
                        return Some(Self::Uncommitted(uncommitted));
                    }
                }
                MarkableRefDiscriminants::Commit => {
                    if let CliId::Commit { commit_id, id } = cli_id {
                        return Some(Self::Commit(MarkedCommitRef {
                            commit_id: *commit_id,
                            id,
                        }));
                    }
                }
            }
        }

        None
    }

    pub fn matches_cli_id(&self, cli_id: &CliId) -> bool {
        MarkableRef::try_from_cli_id(cli_id).is_some_and(|id| self == &id)
    }

    pub fn to_owned(self) -> Markable {
        match self {
            MarkableRef::Uncommitted(hunk) => Markable::Uncommitted(hunk.clone()),
            MarkableRef::Commit(MarkedCommitRef { commit_id, id }) => {
                Markable::Commit(MarkedCommit {
                    commit_id,
                    id: id.to_owned(),
                })
            }
        }
    }
}

impl PartialEq<MarkableRef<'_>> for Markable {
    fn eq(&self, other: &MarkableRef<'_>) -> bool {
        <MarkableRef<'_> as PartialEq>::eq(&self.as_ref(), other)
    }
}

impl PartialEq<Markable> for MarkableRef<'_> {
    fn eq(&self, other: &Markable) -> bool {
        <MarkableRef<'_> as PartialEq>::eq(&other.as_ref(), self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MarkedCommitRef<'a> {
    pub commit_id: gix::ObjectId,
    pub id: &'a str,
}
