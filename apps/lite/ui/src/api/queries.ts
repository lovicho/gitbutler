import type {
	AbsorptionPlanParams,
	BranchDetailsParams,
	BranchDiffParams,
	CommitDetailsWithLineStatsParams,
	GetReviewParams,
	ListBranchesParams,
	ListCiChecksParams,
	TreeChangeDiffParams,
} from "#electron/ipc.ts";
import { aggregateCIChecks } from "#ui/ci.ts";
import { queryOptions } from "@tanstack/react-query";

export enum QueryKey {
	BranchDetails = "branchDetails",
	BranchDiff = "branchDiff",
	ChangesInWorktree = "changesInWorktree",
	CIChecks = "ciChecks",
	CommitDetailsWithLineStats = "commitDetailsWithLineStats",
	ForgeInfo = "forgeInfo",
	HeadInfo = "headInfo",
	Review = "review",
	ReviewMergeStatus = "reviewMergeStatus",
	Branches = "branches",
	Editors = "editors",
	Projects = "projects",
	TreeChangeDiffs = "treeChangeDiffs",
	AbsorptionPlan = "absorptionPlan",
	DryRun = "dryRun",
}

export const branchDetailsQueryOptions = ({ projectId, ...params }: BranchDetailsParams) =>
	queryOptions({
		queryKey: [QueryKey.BranchDetails, projectId, params],
		queryFn: () => window.lite.branchDetails({ projectId, ...params }),
	});

export const branchDiffQueryOptions = ({ projectId, ...params }: BranchDiffParams) =>
	queryOptions({
		queryKey: [QueryKey.BranchDiff, projectId, params],
		queryFn: () => window.lite.branchDiff({ projectId, ...params }),
	});

export const changesInWorktreeQueryOptions = (projectId: string) =>
	queryOptions({
		queryKey: [QueryKey.ChangesInWorktree, projectId],
		queryFn: () => window.lite.changesInWorktree(projectId),
	});

export const commitDetailsWithLineStatsQueryOptions = ({
	projectId,
	...params
}: CommitDetailsWithLineStatsParams) =>
	queryOptions({
		queryKey: [QueryKey.CommitDetailsWithLineStats, projectId, params],
		queryFn: () => window.lite.commitDetailsWithLineStats({ projectId, ...params }),
	});

export const forgeInfoOptions = (projectId: string) =>
	queryOptions({
		queryKey: [QueryKey.ForgeInfo, projectId],
		queryFn: () => window.lite.forgeInfo(projectId),
	});

export const headInfoQueryOptions = (projectId: string) =>
	queryOptions({
		queryKey: [QueryKey.HeadInfo, projectId],
		queryFn: () => window.lite.headInfo(projectId),
	});

export const getReviewQueryOptions = ({ projectId, reviewId }: GetReviewParams) =>
	queryOptions({
		queryKey: [QueryKey.Review, projectId, reviewId],
		queryFn: () => window.lite.getReview({ projectId, reviewId }),
	});

export const getReviewMergeStatusQueryOptions = ({ projectId, reviewId }: GetReviewParams) =>
	queryOptions({
		queryKey: [QueryKey.ReviewMergeStatus, projectId, reviewId],
		queryFn: () => window.lite.getReviewMergeStatus({ projectId, reviewId }),
	});

/** @public */
export const listBranchesQueryOptions = ({ projectId, ...params }: ListBranchesParams) =>
	queryOptions({
		queryKey: [QueryKey.Branches, projectId, params],
		queryFn: () => window.lite.listBranches(projectId, params.filter),
	});

export const listProjectsQueryOptions = queryOptions({
	queryKey: [QueryKey.Projects],
	queryFn: () => window.lite.listProjectsStateless(),
});

export const listEditorsQueryOptions = queryOptions({
	queryKey: [QueryKey.Editors],
	queryFn: () => window.lite.listEditors(),
});

// There is no watcher event that could invalidate this query.
export const listCIChecksQueryOptions = ({
	projectId,
	reference,
	polling,
}: Omit<ListCiChecksParams, "cacheConfig"> & {
	polling: "passive" | "priority";
}) =>
	queryOptions({
		queryKey: [QueryKey.CIChecks, projectId, reference],
		queryFn: async () => {
			// Aggregated data is needed in queryFn to adjust refetching behaviour. Aggregating here, for
			// use as mentioned and also at call sites, is more efficient.
			//
			// listCiChecks will reject with a message citing HTTP 422 once the branch is merged.
			try {
				const data = await window.lite.listCiChecks({
					projectId,
					reference,
					cacheConfig: "noCache",
				});
				return { data, aggregate: aggregateCIChecks(data) };
			} catch {
				return { data: [], aggregate: null };
			}
		},
		// Refetch periodically, being mindful of rate limiting. Similarly tweak stale time for
		// prioritised queries so that fresh data is likely fetched when the user would see/expect it
		// e.g. window refocus.
		refetchInterval: ({ state: { data: checks } }): number => {
			const prio = polling === "priority";

			switch (checks?.aggregate?.status) {
				case "in_progress":
					return prio ? 5_000 : 15_000;
				case "action_required":
					return prio ? 10_000 : 45_000;
				case "success":
				case "cancelled":
				case "failure":
				case "unknown":
				case undefined:
					return prio ? 20_000 : 120_000;
			}
		},
		staleTime: ({ state: { data: checks } }): number => {
			// Our global default.
			if (polling === "passive") return Number.POSITIVE_INFINITY;

			switch (checks?.aggregate?.status) {
				case "in_progress":
					return 5_000;
				case "action_required":
					return 10_000;
				case "success":
				case "cancelled":
				case "failure":
				case "unknown":
				case undefined:
					return 30_000;
			}
		},
	});

export const treeChangeDiffsQueryOptions = ({ projectId, change }: TreeChangeDiffParams) =>
	queryOptions({
		queryKey: [QueryKey.TreeChangeDiffs, projectId, change],
		queryFn: () => window.lite.treeChangeDiffs({ projectId, change }),
	});

export const absorptionPlanQueryOptions = ({ projectId, target }: AbsorptionPlanParams) =>
	queryOptions({
		queryKey: [QueryKey.AbsorptionPlan, projectId, target],
		queryFn: () => window.lite.absorptionPlan({ projectId, target }),
	});
