import type { HeadInfoIndex } from "#ui/api/ref-info.ts";
import {
	branchOperand,
	commitOperand,
	hunkOperand,
	operandEquals,
	operandIdentityKey,
	type BranchOperand,
	type CommitOperand,
	type HunkOperand,
	type Operand,
} from "#ui/operands.ts";
import type { Placement } from "#ui/operations/operation.ts";
import {
	absorbOutlineMode,
	defaultOutlineMode,
	isValidOutlineModeForSelection,
	keyboardTransferMode,
	pointerTransferMode,
	renameBranchOutlineMode,
	rewordCommitOutlineMode,
	transferOutlineMode,
	type OutlineMode,
	type TransferMode,
} from "#ui/outline/mode.ts";
import { navigationIndexIncludes, type NavigationIndex } from "#ui/workspace/navigation-index.ts";
import type { SelectionScope } from "#ui/selection-scopes.ts";
import { createSelector } from "@reduxjs/toolkit";
import type { AbsorptionTarget } from "@gitbutler/but-sdk";
import { Match } from "effect";

export type SelectionState = {
	uncommittedFiles: string | null;
	outline: Operand | null;
	files: string | null;
	diff: HunkOperand | null;
};

type DetailsSelectionScope = Extract<SelectionScope, "uncommitted-files" | "outline">;

type WorkspaceState = {
	checkedCommitIds: Record<string, true>;
	detailsSelectionScope: DetailsSelectionScope | null;
	highlightedCommitIds: Array<string>;
	mode: OutlineMode;
	selection: SelectionState;
};

const createInitialSelectionState = (): SelectionState => ({
	uncommittedFiles: null,
	outline: null,
	files: null,
	diff: null,
});

const createInitialWorkspaceState = (): WorkspaceState => ({
	checkedCommitIds: {},
	detailsSelectionScope: null,
	highlightedCommitIds: [],
	mode: defaultOutlineMode,
	selection: createInitialSelectionState(),
});

export type ProjectState = {
	filesVisible: boolean;
	workspace: WorkspaceState;
};

export const createInitialProjectState = (): ProjectState => ({
	filesVisible: false,
	workspace: createInitialWorkspaceState(),
});

const resolveNavigationIndexSelection = <T>(
	navigationIndex: NavigationIndex<T>,
	selection: T | null,
	getKey: (item: T) => string,
): T | null =>
	selection !== null && navigationIndexIncludes(navigationIndex, selection, getKey)
		? selection
		: (navigationIndex.items[0] ?? null);

const hunkOperandIdentityKey = (operand: HunkOperand): string =>
	operandIdentityKey(hunkOperand(operand));

export const projectReducers = {
	setDetailsSelectionScope: (state: ProjectState, { scope }: { scope: DetailsSelectionScope }) => {
		state.workspace.detailsSelectionScope = scope;
	},
	selectUncommittedFiles: (state: ProjectState, { selection }: { selection: string | null }) => {
		const workspaceState = state.workspace;
		if (workspaceState.selection.uncommittedFiles === selection) return;

		workspaceState.selection.uncommittedFiles = selection;
	},
	selectOutline: (state: ProjectState, { selection }: { selection: Operand | null }) => {
		const workspaceState = state.workspace;
		if (
			selection &&
			workspaceState.selection.outline &&
			operandEquals(workspaceState.selection.outline, selection)
		)
			return;

		workspaceState.selection.outline = selection;
		workspaceState.selection.files = null;
		workspaceState.selection.diff = null;

		if (!selection || !isValidOutlineModeForSelection({ mode: workspaceState.mode, selection }))
			workspaceState.mode = defaultOutlineMode;
	},
	selectFiles: (state: ProjectState, { selection }: { selection: string | null }) => {
		const workspaceState = state.workspace;
		if (workspaceState.selection.files === selection) return;

		workspaceState.selection.files = selection;
	},
	selectDiff: (state: ProjectState, { selection }: { selection: HunkOperand | null }) => {
		const workspaceState = state.workspace;
		if (
			selection &&
			workspaceState.selection.diff &&
			operandEquals(hunkOperand(workspaceState.selection.diff), hunkOperand(selection))
		)
			return;

		workspaceState.selection.diff = selection;
	},
	startRewordCommit: (state: ProjectState, { commit }: { commit: CommitOperand }) => {
		const workspaceState = state.workspace;
		const selection = commitOperand(commit);
		if (
			!workspaceState.selection.outline ||
			!operandEquals(workspaceState.selection.outline, selection)
		) {
			workspaceState.selection.outline = selection;
			workspaceState.selection.files = null;
			workspaceState.selection.diff = null;
			if (!isValidOutlineModeForSelection({ mode: workspaceState.mode, selection }))
				workspaceState.mode = defaultOutlineMode;
		}

		workspaceState.mode = rewordCommitOutlineMode({ operand: commit });
	},
	startRenameBranch: (state: ProjectState, { branch }: { branch: BranchOperand }) => {
		const workspaceState = state.workspace;
		const selection = branchOperand(branch);
		if (
			!workspaceState.selection.outline ||
			!operandEquals(workspaceState.selection.outline, selection)
		) {
			workspaceState.selection.outline = selection;
			workspaceState.selection.files = null;
			workspaceState.selection.diff = null;
			if (!isValidOutlineModeForSelection({ mode: workspaceState.mode, selection }))
				workspaceState.mode = defaultOutlineMode;
		}

		workspaceState.mode = renameBranchOutlineMode({ operand: branch });
	},
	updateRewrittenBranchReferences: (
		state: ProjectState,
		{ oldBranch, newBranch }: { oldBranch: BranchOperand; newBranch: BranchOperand },
	) => {
		const workspaceState = state.workspace;
		const oldBranchOperand = branchOperand(oldBranch);
		const newBranchOperand = branchOperand(newBranch);

		if (
			workspaceState.selection.outline?._tag === "Branch" &&
			operandEquals(workspaceState.selection.outline, oldBranchOperand)
		)
			workspaceState.selection.outline = newBranchOperand;

		if (
			workspaceState.mode._tag === "RenameBranch" &&
			operandEquals(branchOperand(workspaceState.mode.operand), oldBranchOperand)
		)
			workspaceState.mode = renameBranchOutlineMode({ operand: newBranch });
	},
	enterTransferMode: (state: ProjectState, { mode }: { mode: TransferMode }) => {
		state.workspace.mode = transferOutlineMode(mode);
	},
	enterKeyboardTransferMode: (
		state: ProjectState,
		{ sources, placement }: { sources: Array<Operand>; placement?: Placement },
	) => {
		const workspaceState = state.workspace;
		workspaceState.mode = transferOutlineMode(
			keyboardTransferMode({
				sources,
				placement: placement ?? "into",
				restoreSelection: {
					uncommittedFiles: workspaceState.selection.uncommittedFiles,
					outline: workspaceState.selection.outline,
					files: workspaceState.selection.files,
					diff: workspaceState.selection.diff,
				},
			}),
		);
	},
	enterAbsorbMode: (
		state: ProjectState,
		{ source, sourceTarget }: { source: Operand; sourceTarget: AbsorptionTarget },
	) => {
		const workspaceState = state.workspace;
		workspaceState.mode = absorbOutlineMode({
			source,
			restoreSelection: {
				uncommittedFiles: workspaceState.selection.uncommittedFiles,
				outline: workspaceState.selection.outline,
				files: workspaceState.selection.files,
				diff: workspaceState.selection.diff,
			},
			sourceTarget,
		});
	},
	updatePointerTransfer: (
		state: ProjectState,
		{ target, placement }: { target: Operand | null; placement: Placement | null },
	) => {
		const workspaceState = state.workspace;
		Match.value(workspaceState.mode).pipe(
			Match.when({ _tag: "Transfer", value: { _tag: "Pointer" } }, ({ value: mode }) => {
				const sameTarget =
					target === null
						? mode.target === null
						: mode.target !== null && operandEquals(mode.target, target);
				if (sameTarget && mode.placement === placement) return;

				workspaceState.mode = transferOutlineMode(
					pointerTransferMode({
						sources: mode.sources,
						target,
						placement,
					}),
				);
			}),
			Match.orElse(() => {}),
		);
	},
	updateTransferPlacement: (state: ProjectState, { placement }: { placement: Placement }) => {
		const workspaceState = state.workspace;
		Match.value(workspaceState.mode).pipe(
			Match.when({ _tag: "Transfer", value: { _tag: "Keyboard" } }, ({ value: mode }) => {
				workspaceState.mode = transferOutlineMode(
					keyboardTransferMode({
						sources: mode.sources,
						placement,
						restoreSelection: mode.restoreSelection,
					}),
				);
			}),
			Match.orElse(() => {}),
		);
	},
	exitMode: (state: ProjectState) => {
		state.workspace.mode = defaultOutlineMode;
	},
	cancelMode: (state: ProjectState) => {
		const workspaceState = state.workspace;
		const restoreSelection = Match.value(workspaceState.mode).pipe(
			Match.tags({
				Absorb: (mode) => mode.restoreSelection,
				Transfer: (mode) => (mode.value._tag === "Keyboard" ? mode.value.restoreSelection : null),
			}),
			Match.orElse(() => null),
		);
		workspaceState.mode = defaultOutlineMode;

		if (!restoreSelection) return;

		workspaceState.selection = restoreSelection;
	},
	setHighlightedCommitIds: (
		state: ProjectState,
		{ commitIds }: { commitIds: Array<string> | null },
	) => {
		state.workspace.highlightedCommitIds = commitIds ?? [];
	},
	checkCommit: (
		state: ProjectState,
		{ commitId, checked }: { commitId: string; checked: boolean },
	) => {
		const checkedCommitIds = state.workspace.checkedCommitIds;
		if (checked) checkedCommitIds[commitId] = true;
		else delete checkedCommitIds[commitId];
	},
	checkCommits: (
		state: ProjectState,
		{ commitIds, checked }: { commitIds: Array<string>; checked: boolean },
	) => {
		const checkedCommitIds = state.workspace.checkedCommitIds;
		for (const commitId of commitIds) {
			if (checked) checkedCommitIds[commitId] = true;
			else delete checkedCommitIds[commitId];
		}
	},
	setCheckedCommits: (state: ProjectState, { commitIds }: { commitIds: Array<string> }) => {
		state.workspace.checkedCommitIds = commitIds.reduce(
			(acc, commitId) => {
				acc[commitId] = true;
				return acc;
			},
			{} as Record<string, true>,
		);
	},
	clearCheckedCommits: (state: ProjectState) => {
		state.workspace.checkedCommitIds = {};
	},
	updateRewrittenCommitReferences: (
		state: ProjectState,
		{ replacedCommits }: { replacedCommits: Record<string, string> },
	) => {
		const workspaceState = state.workspace;
		const selection = workspaceState.selection.outline;
		if (selection?._tag === "Commit") {
			const newId = replacedCommits[selection.commitId];
			if (newId !== undefined)
				workspaceState.selection.outline = commitOperand({ commitId: newId });
		}

		for (const oldId of Object.keys(workspaceState.checkedCommitIds)) {
			const newId = replacedCommits[oldId];
			if (newId !== undefined) {
				delete workspaceState.checkedCommitIds[oldId];
				workspaceState.checkedCommitIds[newId] = true;
			}
		}

		if (workspaceState.mode._tag === "RewordCommit") {
			const newId = replacedCommits[workspaceState.mode.operand.commitId];
			if (newId !== undefined)
				workspaceState.mode = rewordCommitOutlineMode({ operand: { commitId: newId } });
		}
	},
	toggleFiles: (state: ProjectState) => {
		state.filesVisible = !state.filesVisible;
	},
};

const selectCheckedCommits = createSelector(
	(state: ProjectState) => state.workspace.checkedCommitIds,
	(_state: ProjectState, headInfoIndex: HeadInfoIndex) => headInfoIndex,
	(checkedCommitIds, headInfoIndex) =>
		new Set(
			Object.keys(checkedCommitIds).filter(
				(commitId) => headInfoIndex.commitContextById(commitId) !== undefined,
			),
		),
);

const selectCheckedCommitOperands = createSelector(selectCheckedCommits, (checkedCommitIds) =>
	Array.from(checkedCommitIds).map((commitId) => commitOperand({ commitId })),
);

export const projectSelectors = {
	selectFilesVisible: (state: ProjectState) => state.filesVisible,
	selectCanShowFiles: (state: ProjectState) =>
		state.workspace.detailsSelectionScope !== "uncommitted-files",
	selectDetailsSelectionScope: (state: ProjectState) => state.workspace.detailsSelectionScope,
	selectSelectionUncommittedFiles: (
		state: ProjectState,
		navigationIndex: NavigationIndex<string>,
	) =>
		resolveNavigationIndexSelection(
			navigationIndex,
			state.workspace.selection.uncommittedFiles,
			(path) => path,
		),
	selectIsSelectedOutline: (
		state: ProjectState,
		navigationIndex: NavigationIndex<Operand>,
		operand: Operand,
	) => {
		const selection = resolveNavigationIndexSelection(
			navigationIndex,
			state.workspace.selection.outline,
			operandIdentityKey,
		);
		return selection !== null && operandEquals(selection, operand);
	},
	selectSelectionOutline: (state: ProjectState, navigationIndex: NavigationIndex<Operand>) =>
		resolveNavigationIndexSelection(
			navigationIndex,
			state.workspace.selection.outline,
			operandIdentityKey,
		),
	selectSelectionFiles: (state: ProjectState, navigationIndex: NavigationIndex<string>) =>
		resolveNavigationIndexSelection(
			navigationIndex,
			state.workspace.selection.files,
			(item) => item,
		),
	selectSelectionDiff: (state: ProjectState, navigationIndex: NavigationIndex<HunkOperand>) =>
		resolveNavigationIndexSelection(
			navigationIndex,
			state.workspace.selection.diff,
			hunkOperandIdentityKey,
		),
	selectOutlineModeState: (state: ProjectState) => state.workspace.mode,
	selectHighlightedCommitIds: (state: ProjectState) => state.workspace.highlightedCommitIds,
	selectCommitChecked: (state: ProjectState, commitId: string) =>
		state.workspace.checkedCommitIds[commitId] === true,
	selectCheckedCommits,
	selectCheckedCommitOperands,
	selectCheckedCommitCount: (state: ProjectState, headInfoIndex: HeadInfoIndex) =>
		selectCheckedCommits(state, headInfoIndex).size,
	selectHasCheckedCommits: (state: ProjectState, headInfoIndex: HeadInfoIndex) =>
		selectCheckedCommits(state, headInfoIndex).size > 0,
};
