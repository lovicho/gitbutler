import {
	syncStackAfterReviewCreation,
	unstackPRs,
	updatePrDescriptionTables,
	updateStackPrs,
} from "$lib/forge/shared/prFooter";
import { showWarning } from "$lib/notifications/toasts";
import { describe, expect, test, vi } from "vitest";
import type { PullRequest } from "$lib/forge/interface/types";
import type { PrService } from "$lib/forge/prService.svelte";
import type { Segment } from "@gitbutler/but-sdk";

vi.mock("$lib/notifications/toasts", () => ({ showWarning: vi.fn() }));

function mockPrService(bodies: Record<number, string | null> = {}) {
	const fetch = vi.fn(async (_projectId: string, number: number) => ({
		number,
		body: bodies[number] ?? null,
	}));
	const updateReviewFooters = vi.fn(async () => undefined);
	const service = { fetch, updateReviewFooters } as unknown as PrService;
	return { service, fetch, updateReviewFooters };
}

function segment(name: string, review?: number): Segment {
	return {
		refName: { displayName: name },
		metadata: { review: { pullRequest: review } },
	} as Segment;
}

describe("updatePrDescriptionTables", () => {
	test("translates desktop top-to-base ordering to Rust base-to-top ordering", async () => {
		const { service, fetch, updateReviewFooters } = mockPrService({
			100: "Base description",
			102: "Top description",
		});

		await updatePrDescriptionTables(service, "project", [102, 101, 100], "#");

		expect(fetch.mock.calls.map(([, number]) => number)).toEqual([100, 101, 102]);
		expect(updateReviewFooters).toHaveBeenCalledWith("project", [
			{
				number: 100,
				body: "Base description",
				updateDescription: true,
				unitSymbol: "#",
				targetBranch: null,
			},
			{
				number: 101,
				body: null,
				updateDescription: true,
				unitSymbol: "#",
				targetBranch: null,
			},
			{
				number: 102,
				body: "Top description",
				updateDescription: true,
				unitSymbol: "#",
				targetBranch: null,
			},
		]);
	});

	test("does not invoke Rust for a single review", async () => {
		const { service, fetch, updateReviewFooters } = mockPrService();

		await updatePrDescriptionTables(service, "project", [100]);

		expect(fetch).not.toHaveBeenCalled();
		expect(updateReviewFooters).not.toHaveBeenCalled();
	});

	test("propagates Rust synchronization failures", async () => {
		const { service, updateReviewFooters } = mockPrService();
		updateReviewFooters.mockRejectedValueOnce(new Error("forge update failed"));

		await expect(updatePrDescriptionTables(service, "project", [101, 100])).rejects.toThrow(
			"forge update failed",
		);
	});
});

describe("syncStackAfterReviewCreation", () => {
	test("returns the created review and warns when stack synchronization fails", async () => {
		const { service, updateReviewFooters } = mockPrService();
		const createdReview = { number: 101 } as PullRequest;
		updateReviewFooters.mockRejectedValueOnce(new Error("forge update failed"));
		vi.spyOn(console, "error").mockImplementationOnce(() => undefined);

		const result = await syncStackAfterReviewCreation(
			service,
			"project",
			createdReview,
			[101, 100],
			"#",
		);

		expect(result).toBe(createdReview);
		expect(showWarning).toHaveBeenCalledWith(
			"Pull request created with incomplete stack information",
			"PR #101 was created, but its stack information could not be synchronized. forge update failed",
		);
	});
});

describe("updateStackPrs", () => {
	test("sends reviews base-to-top with chained target branches", async () => {
		const { service, updateReviewFooters } = mockPrService({
			100: "Base",
			101: "Middle",
			102: "Top",
		});

		await updateStackPrs(
			service,
			"project",
			[segment("top", 102), segment("middle", 101), segment("base", 100)],
			"main",
		);

		expect(updateReviewFooters).toHaveBeenCalledWith("project", [
			{
				number: 100,
				body: "Base",
				updateDescription: true,
				unitSymbol: "#",
				targetBranch: "main",
			},
			{
				number: 101,
				body: "Middle",
				updateDescription: true,
				unitSymbol: "#",
				targetBranch: "base",
			},
			{
				number: 102,
				body: "Top",
				updateDescription: true,
				unitSymbol: "#",
				targetBranch: "middle",
			},
		]);
	});

	test("uses an unpublished branch as the next published review target", async () => {
		const { service, updateReviewFooters } = mockPrService();

		await updateStackPrs(
			service,
			"project",
			[segment("top", 102), segment("unpublished"), segment("base", 100)],
			"main",
		);

		expect(updateReviewFooters).toHaveBeenCalledWith("project", [
			{
				number: 100,
				body: null,
				updateDescription: true,
				unitSymbol: "#",
				targetBranch: "main",
			},
			{
				number: 102,
				body: null,
				updateDescription: true,
				unitSymbol: "#",
				targetBranch: "unpublished",
			},
		]);
	});
});

describe("unstackPRs", () => {
	test("submits each former stack member as a one-review cleanup batch", async () => {
		const { service, updateReviewFooters } = mockPrService({ 100: "Base", 101: "Top" });

		await unstackPRs(service, "project", [100, 101], "main");

		expect(updateReviewFooters).toHaveBeenCalledTimes(2);
		expect(updateReviewFooters).toHaveBeenCalledWith("project", [
			{
				number: 100,
				body: "Base",
				updateDescription: true,
				unitSymbol: "",
				targetBranch: "main",
			},
		]);
		expect(updateReviewFooters).toHaveBeenCalledWith("project", [
			{
				number: 101,
				body: "Top",
				updateDescription: true,
				unitSymbol: "",
				targetBranch: "main",
			},
		]);
	});
});
