import { getDependencyCommitIds, getHunkDependencyDiffsByPath } from "#ui/hunk.ts";
import type { TreeChange, WorktreeChanges } from "@gitbutler/but-sdk";

type ChangeFileRowItem = {
	change: TreeChange;
	dependencyCommitIds: Array<string>;
	path: string;
};

export const changeFileRowItem = ({
	change,
	dependencyCommitIds,
	path,
}: ChangeFileRowItem): FileRowItem => ({
	_tag: "Change",
	change,
	dependencyCommitIds,
	path,
});

type ConflictFileRowItem = {
	path: string;
};

export const conflictFileRowItem = ({ path }: ConflictFileRowItem): FileRowItem => ({
	_tag: "Conflict",
	path,
});

export const getChangesFileRowItems = (worktreeChanges: WorktreeChanges): Array<FileRowItem> => {
	const hunkDependencyDiffsByPath = getHunkDependencyDiffsByPath(
		worktreeChanges.dependencies?.diffs ?? [],
	);

	return worktreeChanges.changes.map((change) => {
		const hunkDependencyDiffs = hunkDependencyDiffsByPath.get(change.path);
		const dependencyCommitIds = hunkDependencyDiffs
			? getDependencyCommitIds({ hunkDependencyDiffs })
			: [];

		return changeFileRowItem({
			change,
			dependencyCommitIds,
			path: change.path,
		});
	});
};

export type FileRowItem =
	| ({ _tag: "Change" } & ChangeFileRowItem)
	| ({ _tag: "Conflict" } & ConflictFileRowItem);
