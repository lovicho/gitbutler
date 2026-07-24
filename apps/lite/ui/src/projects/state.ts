import {
	createInitialProjectState,
	projectReducers,
	projectSelectors,
	type ProjectState,
} from "#ui/projects/project.ts";
import { createSlice, type PayloadAction } from "@reduxjs/toolkit";

type ProjectSliceState = {
	byProjectId: Record<string, ProjectState>;
};

const initialState: ProjectSliceState = {
	byProjectId: {},
};

const ensureProjectState = (state: ProjectSliceState, projectId: string): ProjectState => {
	const existingState = state.byProjectId[projectId];
	if (existingState) return existingState;

	const projectState = createInitialProjectState();
	state.byProjectId[projectId] = projectState;
	return projectState;
};

const initialProjectState: ProjectState = createInitialProjectState();
const selectProjectState = (state: ProjectSliceState, projectId: string): ProjectState =>
	state.byProjectId[projectId] ?? initialProjectState;

const withProject =
	<T>(reducer: (state: ProjectState, payload: T) => void) =>
	(state: ProjectSliceState, action: PayloadAction<T & { projectId: string }>) => {
		reducer(ensureProjectState(state, action.payload.projectId), action.payload);
	};

const fromProject =
	<T extends Array<unknown>, R>(selector: (state: ProjectState, ...args: T) => R) =>
	(state: ProjectSliceState, projectId: string, ...args: T): R =>
		selector(selectProjectState(state, projectId), ...args);

export const projectSlice = createSlice({
	name: "project",
	initialState,
	reducers: {
		setDetailsSelectionScope: withProject(projectReducers.setDetailsSelectionScope),
		selectUncommittedFiles: withProject(projectReducers.selectUncommittedFiles),
		selectOutline: withProject(projectReducers.selectOutline),
		selectFiles: withProject(projectReducers.selectFiles),
		selectDiff: withProject(projectReducers.selectDiff),
		startRewordCommit: withProject(projectReducers.startRewordCommit),
		startRenameBranch: withProject(projectReducers.startRenameBranch),
		updateRewrittenBranchReferences: withProject(projectReducers.updateRewrittenBranchReferences),
		enterTransferMode: withProject(projectReducers.enterTransferMode),
		enterKeyboardTransferMode: withProject(projectReducers.enterKeyboardTransferMode),
		enterAbsorbMode: withProject(projectReducers.enterAbsorbMode),
		updatePointerTransfer: withProject(projectReducers.updatePointerTransfer),
		updateTransferPlacement: withProject(projectReducers.updateTransferPlacement),
		exitMode: withProject(projectReducers.exitMode),
		cancelMode: withProject(projectReducers.cancelMode),
		setHighlightedCommitIds: withProject(projectReducers.setHighlightedCommitIds),
		checkOperand: withProject(projectReducers.checkOperand),
		checkOperands: withProject(projectReducers.checkOperands),
		clearCheckedOperands: withProject(projectReducers.clearCheckedOperands),
		updateRewrittenCommitReferences: withProject(projectReducers.updateRewrittenCommitReferences),
		toggleFiles: withProject(projectReducers.toggleFiles),
		setOutlineTab: withProject(projectReducers.setOutlineTab),
		toggleBranchUnfolded: withProject(projectReducers.toggleBranchUnfolded),
		setBranchSearch: withProject(projectReducers.setBranchSearch),
		toggleBranchFilter: withProject(projectReducers.toggleBranchFilter),
	},
	selectors: {
		selectFilesVisible: fromProject(projectSelectors.selectFilesVisible),
		selectOutlineTab: fromProject(projectSelectors.selectOutlineTab),
		selectBranchFilters: fromProject(projectSelectors.selectBranchFilters),
		selectBranchSearch: fromProject(projectSelectors.selectBranchSearch),
		selectUnfoldedBranches: fromProject(projectSelectors.selectUnfoldedBranches),
		selectBranchUnfolded: fromProject(projectSelectors.selectBranchUnfolded),
		selectCanShowFiles: fromProject(projectSelectors.selectCanShowFiles),
		selectDetailsSelectionScope: fromProject(projectSelectors.selectDetailsSelectionScope),
		selectSelectionUncommittedFiles: fromProject(projectSelectors.selectSelectionUncommittedFiles),
		selectIsSelectedOutline: fromProject(projectSelectors.selectIsSelectedOutline),
		selectPrimaryOutlineSelection: fromProject(projectSelectors.selectPrimaryOutlineSelection),
		selectSelectionOutline: fromProject(projectSelectors.selectSelectionOutline),
		selectSelectionFiles: fromProject(projectSelectors.selectSelectionFiles),
		selectSelectionDiff: fromProject(projectSelectors.selectSelectionDiff),
		selectOutlineModeState: fromProject(projectSelectors.selectOutlineModeState),
		selectHighlightedCommitIds: fromProject(projectSelectors.selectHighlightedCommitIds),
		selectOperandChecked: fromProject(projectSelectors.selectOperandChecked),
		selectCheckedOperands: fromProject(projectSelectors.selectCheckedOperands),
		selectCheckedOperandKeys: fromProject(projectSelectors.selectCheckedOperandKeys),
		selectCheckedCommitIds: fromProject(projectSelectors.selectCheckedCommitIds),
		selectCheckedOperandCount: fromProject(projectSelectors.selectCheckedOperandCount),
		selectHasCheckedOperands: fromProject(projectSelectors.selectHasCheckedOperands),
	},
});
