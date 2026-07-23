import { showWarning } from "$lib/notifications/toasts";
import { isDefined } from "@gitbutler/ui/utils/typeguards";
import type { PullRequest } from "$lib/forge/interface/types";
import type { PrService } from "$lib/forge/prService.svelte";
import type { Segment } from "@gitbutler/but-sdk";

export const STACKING_FOOTER_BOUNDARY_TOP = "<!-- GitButler Footer Boundary Top -->";
export const STACKING_FOOTER_BOUNDARY_BOTTOM = "<!-- GitButler Footer Boundary Bottom -->";

export const BUT_REVIEW_FOOTER_BOUNDARY_TOP = "<!-- GitButler Review Footer Boundary Top -->";
export const BUT_REVIEW_FOOTER_BOUNDARY_BOTTOM = "<!-- GitButler Review Footer Boundary Bottom -->";

export function unixifyNewlines(target: string): string {
	return target.split(/\r?\n/).join("\n");
}

/**
 * Updates a pull request description with a table pointing to other pull
 * requests in the same stack.
 */
export async function updatePrDescriptionTables(
	prService: PrService,
	projectId: string,
	prNumbers: number[],
	unitSymbol = "#",
) {
	if (prService && prNumbers.length > 1) {
		// Review creation tracks segments top-to-base; the Rust API accepts base-to-top.
		const baseToTopPrNumbers = [...prNumbers].reverse();
		const prs = await Promise.all(
			baseToTopPrNumbers.map(async (id) => await prService.fetch(projectId, id)),
		);
		await prService.updateReviewFooters(
			projectId,
			prs.filter(isDefined).map((pr) => ({
				number: pr.number,
				body: pr.body ?? null,
				updateDescription: true,
				unitSymbol,
				targetBranch: null,
			})),
		);
	}
}

/**
 * Synchronizes stack descriptions after a review has been created. The review creation is already
 * durable at this point, so synchronization failures are reported as partial success.
 */
export async function syncStackAfterReviewCreation(
	prService: PrService,
	projectId: string,
	createdReview: PullRequest,
	prNumbers: number[],
	unitSymbol = "#",
): Promise<PullRequest> {
	try {
		await updatePrDescriptionTables(prService, projectId, prNumbers, unitSymbol);
	} catch (error) {
		console.error(error);
		const message = error instanceof Error ? error.message : String(error);
		showWarning(
			"Pull request created with incomplete stack information",
			`PR ${unitSymbol}${createdReview.number} was created, but its stack information could not be synchronized. ${message}`,
		);
	}

	return createdReview;
}

type PrUpdate = {
	prNumber: number;
	targetBase: string;
	body: string | null;
};

export async function updateStackPrs(
	prService: PrService,
	projectId: string,
	branchDetails: Segment[],
	baseBranchName: string,
	unitSymbol = "#",
) {
	if (branchDetails.length <= 1) return;
	const updates: PrUpdate[] = [];
	let prevBranch: string | undefined = undefined;

	for (let i = branchDetails.length - 1; i >= 0; i--) {
		const details = branchDetails[i];
		if (!details) continue;
		const branchName = details.refName?.displayName;
		if (!branchName) continue;
		const prNumber = details.metadata?.review.pullRequest;
		if (!isDefined(prNumber)) {
			prevBranch = branchName;
			continue;
		}
		const pr = await prService.fetch(projectId, prNumber);

		if (!isDefined(pr)) {
			prevBranch = branchName;
			continue;
		}

		updates.push({
			prNumber,
			body: pr.body ?? null,
			targetBase: prevBranch ?? baseBranchName,
		});
		prevBranch = branchName;
	}

	if (updates.length > 0) {
		await prService.updateReviewFooters(
			projectId,
			updates.map(({ prNumber, targetBase, body }) => ({
				number: prNumber,
				body,
				updateDescription: true,
				unitSymbol,
				targetBranch: targetBase,
			})),
		);
	}
}

/**
 * Remove the PR description footer from the given PR numbers.
 */
export async function unstackPRs(
	prService: PrService,
	projectId: string,
	prNumbers: number[],
	baseBranchName: string,
) {
	if (prService && prNumbers.length > 0) {
		const prs = await Promise.all(
			prNumbers.map(async (id) => await prService.fetch(projectId, id)),
		);
		await Promise.all(
			prs.filter(isDefined).map(async (pr) => {
				await prService.updateReviewFooters(projectId, [
					{
						number: pr.number,
						body: pr.body ?? null,
						updateDescription: true,
						unitSymbol: "",
						targetBranch: baseBranchName,
					},
				]);
			}),
		);
	}
}
