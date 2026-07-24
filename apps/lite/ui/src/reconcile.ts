/**
 * @file Known rewritten commit IDs and branch names are handled separately before reaching this
 * module.
 */

import { useQueries, useQuery } from "@tanstack/react-query";
import {
	branchDiffQueryOptions,
	changesInWorktreeQueryOptions,
	commitDetailsWithLineStatsQueryOptions,
	headInfoQueryOptions,
} from "./api/queries.ts";
import { useParams } from "@tanstack/react-router";
import { getHeadInfoIndex, type HeadInfoIndex } from "./api/ref-info.ts";
import { useEffectEvent, useLayoutEffect, useRef } from "react";
import { useAppDispatch, useAppSelector } from "./store.ts";
import { projectSlice } from "./projects/state.ts";
import { commitOperand } from "./operands.ts";
import { decodeBytes } from "./api/bytes.ts";

/**
 * Reconcile state between Redux and React Query. This hook should be called very high up in the
 * tree so that synchronous dispatches in layout effects don't waste too much work. This hook
 * remains subscribed to any queries that are relevant to the current state.
 */
export const useStateReconciler = (): void => {
	const { id: projectId } = useParams({ from: "/project/$id/workspace" });

	const dispatch = useAppDispatch();

	const outlineSelection = useAppSelector((state) =>
		projectSlice.selectors.selectPrimaryOutlineSelection(state, projectId),
	);
	const outlineTab = useAppSelector((state) =>
		projectSlice.selectors.selectOutlineTab(state, projectId),
	);
	const reconcileSelectedCommit = useEffectEvent(
		(headInfoIndex: HeadInfoIndex, prevHeadInfoIndex: HeadInfoIndex | null) => {
			// The Branches tab selects commits on unapplied branches, which are
			// absent from headInfo (workspace stacks); its selection is validated
			// against the branches navigation index instead.
			if (outlineTab === "branches") return;
			if (outlineSelection?._tag !== "Commit") return;

			const curr = headInfoIndex.commitContextById(outlineSelection.commitId);
			if (curr) return;

			const prev = prevHeadInfoIndex?.commitContextById(outlineSelection.commitId);
			// Change IDs are not necessarily globally unique, but typically will be. In any case this is
			// a best-effort fallback.
			const commitId = prev
				? headInfoIndex.commitContextById(prev.commit.changeId)?.commit.id
				: null;

			dispatch(
				projectSlice.actions.selectOutline({
					projectId,
					selection: commitId != null ? commitOperand({ commitId }) : null,
				}),
			);
		},
	);

	const checkedOperands = useAppSelector((state) =>
		projectSlice.selectors.selectCheckedOperands(state, projectId),
	);

	const checkedCommits = checkedOperands.filter((operand) => operand._tag === "Commit");
	const reconcileCheckedCommits = useEffectEvent((headInfoIndex: HeadInfoIndex) => {
		const invalidated = checkedCommits.filter(
			(commit) => !headInfoIndex.commitContextById(commit.commitId),
		);

		if (invalidated.length > 0) {
			dispatch(
				projectSlice.actions.checkOperands({ projectId, operands: invalidated, checked: false }),
			);
		}
	});

	const checkedFiles = checkedOperands.filter((operand) => operand._tag === "File");

	const checkedUncommittedFiles = checkedFiles.flatMap(({ parent, ...file }) =>
		parent._tag === "UncommittedChanges" ? [{ ...file, parent }] : [],
	);
	const reconcileCheckedUncommittedFiles = useEffectEvent((worktreeChangePaths: Set<string>) => {
		const invalidated = checkedUncommittedFiles.filter(
			(file) => !worktreeChangePaths.has(file.path),
		);

		if (invalidated.length > 0) {
			dispatch(
				projectSlice.actions.checkOperands({ projectId, operands: invalidated, checked: false }),
			);
		}
	});

	const checkedCommitFiles = checkedFiles.flatMap(({ parent, ...file }) =>
		parent._tag === "Commit" ? [{ ...file, parent }] : [],
	);
	const reconcileCheckedCommitFiles = useEffectEvent(
		(headInfoIndex: HeadInfoIndex, checkedCommitFilesByCommitId: Map<string, Set<string>>) => {
			const invalidated = checkedCommitFiles.filter(
				(file) =>
					!headInfoIndex.commitContextById(file.parent.commitId) ||
					checkedCommitFilesByCommitId.get(file.parent.commitId)?.has(file.path) === false,
			);

			if (invalidated.length > 0) {
				dispatch(
					projectSlice.actions.checkOperands({ projectId, operands: invalidated, checked: false }),
				);
			}
		},
	);

	const checkedBranchFiles = checkedFiles.flatMap(({ parent, ...file }) =>
		parent._tag === "Branch" ? [{ ...file, parent }] : [],
	);
	const reconcileCheckedBranchFiles = useEffectEvent(
		(headInfoIndex: HeadInfoIndex, checkedBranchFilesByBranchName: Map<string, Set<string>>) => {
			const invalidated = checkedBranchFiles.filter(
				(file) =>
					!headInfoIndex.branchContextByRefBytes(file.parent.branchRef) ||
					checkedBranchFilesByBranchName.get(decodeBytes(file.parent.branchRef))?.has(file.path) ===
						false,
			);

			if (invalidated.length > 0) {
				dispatch(
					projectSlice.actions.checkOperands({ projectId, operands: invalidated, checked: false }),
				);
			}
		},
	);

	const { data: headInfoIndex } = useQuery({
		...headInfoQueryOptions(projectId),
		select: getHeadInfoIndex,
	});
	const prevHeadInfoIndexRef = useRef<HeadInfoIndex>(null);
	useLayoutEffect(() => {
		if (!headInfoIndex) return;

		reconcileSelectedCommit(headInfoIndex, prevHeadInfoIndexRef.current);
		reconcileCheckedCommits(headInfoIndex);

		prevHeadInfoIndexRef.current = headInfoIndex;
	}, [headInfoIndex]);

	const { data: worktreeChangePaths } = useQuery({
		...changesInWorktreeQueryOptions(projectId),
		select: (data) => new Set(data.changes.map((change) => change.path)),
		enabled: checkedUncommittedFiles.length > 0,
	});
	useLayoutEffect(() => {
		if (!worktreeChangePaths) return;

		reconcileCheckedUncommittedFiles(worktreeChangePaths);
	}, [worktreeChangePaths]);

	const checkedCommitFileCommitIds = new Set(
		checkedCommitFiles.map((file) => file.parent.commitId),
	);
	const checkedCommitFilesByCommitId = useQueries({
		queries: Array.from(checkedCommitFileCommitIds, (commitId) =>
			commitDetailsWithLineStatsQueryOptions({ projectId, commitId }),
		),
		combine: (results) =>
			new Map(
				results.flatMap((result) =>
					result.data
						? [[result.data.commit.id, new Set(result.data.changes.map((change) => change.path))]]
						: [],
				),
			),
	});
	useLayoutEffect(() => {
		if (!headInfoIndex) return;

		reconcileCheckedCommitFiles(headInfoIndex, checkedCommitFilesByCommitId);
	}, [headInfoIndex, checkedCommitFilesByCommitId]);

	const checkedBranchFileBranchNames = Array.from(
		new Set(checkedBranchFiles.map((file) => decodeBytes(file.parent.branchRef))),
	);
	const checkedBranchFilesByBranchName = useQueries({
		queries: checkedBranchFileBranchNames.map((branch) =>
			branchDiffQueryOptions({ projectId, branch }),
		),
		combine: (results) =>
			new Map(
				results.flatMap((result, idx) => {
					const key = checkedBranchFileBranchNames[idx];
					return key !== undefined && result.data
						? [[key, new Set(result.data.changes.map((change) => change.path))]]
						: [];
				}),
			),
	});
	useLayoutEffect(() => {
		if (!headInfoIndex) return;

		reconcileCheckedBranchFiles(headInfoIndex, checkedBranchFilesByBranchName);
	}, [headInfoIndex, checkedBranchFilesByBranchName]);
};
