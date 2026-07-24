use std::collections::{BTreeMap, HashSet, btree_map::Entry};

use bstr::BString;
use nonempty::NonEmpty;

use crate::id::{WorktreeHunk, id_usage::UintId};

/// Information about uncommitted files.
pub(crate) struct UncommittedInfo {
    /// Uncommitted hunks partitioned by filename.
    pub(crate) partitioned_hunks: Vec<NonEmpty<WorktreeHunk>>,
    pub(crate) uncommitted_short_filenames: HashSet<BString>,
}

impl UncommittedInfo {
    /// Partitions hunk assignments by filename.
    pub(crate) fn from_hunk_assignments(
        hunk_assignments: Vec<WorktreeHunk>,
    ) -> anyhow::Result<Self> {
        let mut uncommitted_hunks: BTreeMap<BString, NonEmpty<_>> = BTreeMap::new();
        let mut uncommitted_short_filenames = HashSet::new();
        for assignment in hunk_assignments {
            if assignment.path_bytes.len() <= UintId::LENGTH_LIMIT
                && !uncommitted_short_filenames.contains(&assignment.path_bytes)
            {
                uncommitted_short_filenames.insert(assignment.path_bytes.clone());
            }
            match uncommitted_hunks.entry(assignment.path_bytes.clone()) {
                Entry::Vacant(vacant_entry) => {
                    vacant_entry.insert(NonEmpty::new(assignment));
                }
                Entry::Occupied(mut occupied_entry) => {
                    occupied_entry.get_mut().push(assignment);
                }
            };
        }

        Ok(Self {
            partitioned_hunks: uncommitted_hunks.into_values().collect(),
            uncommitted_short_filenames,
        })
    }
}
