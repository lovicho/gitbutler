import { assertGitConfigValue } from "../src/branch.ts";
import { mergeStatus, mockForge, repoInfo } from "../src/forge.ts";
import { applyUpstream, getButlerPort, openWorkspace, type GitButler } from "../src/setup.ts";
import { test } from "../src/test.ts";
import {
	clickByTestId,
	dragAndDropByLocator,
	stack,
	textEditorFillByTestId,
	waitForTestId,
} from "../src/util.ts";
import { expect, type Page } from "@playwright/test";
import { readFileSync, writeFileSync } from "node:fs";
import type { FakeGitHubReview, FakeGitHubServer } from "../src/fakeGithub.ts";

const FOOTER_TOP = "<!-- GitButler Footer Boundary Top -->";
const FOOTER_BOTTOM = "<!-- GitButler Footer Boundary Bottom -->";
const POLICY_KEY = "gitbutler.reviewStackingDescription";

test("review stack descriptions follow the per-project policy", async ({
	page,
	gitbutler,
	fakeGithub,
}) => {
	const server = await fakeGithub({
		headRepoPath: gitbutler.pathInWorkdir("remote-project"),
		isFork: false,
		listed: false,
	});
	await gitbutler.runScript("project-with-stacks.sh", [server.repositoryUrl]);
	await storeFakeGitHubEnterprisePat(page, server);
	await applyUpstream(gitbutler, "branch1", "branch2", "branch3", "branch4");
	await mockForge(page, {
		get_review_merge_status: mergeStatus(),
		get_repo_info: repoInfo(),
		list_ci_checks: [],
	});
	await openWorkspace(page);
	await combineBranchesIntoStack(page);

	await publishReview(page, "branch1", "Description for branch1");
	server.setListed(true);
	await publishReview(page, "branch2", "Description for branch2");

	await expectReviews(server, 2, (reviews) => {
		expectBottomFooter(reviews[0], "Description for branch1", "part 1 of 2");
		expectBottomFooter(reviews[1], "Description for branch2", "part 2 of 2");
		expectStackOrder(reviews, [43, 42]);
	});

	await setDescriptionPolicy(page, gitbutler, "top");
	await publishReview(page, "branch3", "Description for branch3");

	await expectReviews(server, 3, (reviews) => {
		for (const [index, review] of reviews.entries()) {
			expect(review.body).toMatch(new RegExp(`^${escapeRegExp(FOOTER_TOP)}`));
			expect(review.body).toContain(`part ${index + 1} of 3`);
			expect(review.body).toMatch(new RegExp(`Description for branch${index + 1}$`));
		}
		expectStackOrder(reviews, [44, 43, 42]);
	});

	await setDescriptionPolicy(page, gitbutler, "disabled");
	await publishReview(page, "branch4", "Description for branch4");

	await expectReviews(server, 4, (reviews) => {
		for (const [index, review] of reviews.entries()) {
			expect(review.body).toBe(`Description for branch${index + 1}`);
			expect(review.body).not.toContain(FOOTER_TOP);
			expect(review.body).not.toContain(FOOTER_BOTTOM);
		}
		expect(reviews.map((review) => review.base.ref)).toEqual([
			"master",
			"branch1",
			"branch2",
			"branch3",
		]);
	});
});

test("pushing one of multiple stacks keeps review descriptions stack-local", async ({
	page,
	gitbutler,
	fakeGithub,
}) => {
	const server = await fakeGithub({
		headRepoPath: gitbutler.pathInWorkdir("remote-project"),
		isFork: false,
		listed: false,
	});
	await gitbutler.runScript("project-with-stacks.sh", [server.repositoryUrl]);
	await storeFakeGitHubEnterprisePat(page, server);
	mirrorFakeCredentialForCli(gitbutler);
	await applyUpstream(gitbutler, "branch1", "branch2", "branch3", "branch4");
	await mockForge(page, {
		get_review_merge_status: mergeStatus(),
		get_repo_info: repoInfo(),
		list_ci_checks: [],
	});
	await openWorkspace(page);

	const branchHeaders = page.getByTestId("branch-header");
	for (const [branch, parent, remainingStacks] of [
		["branch2", "branch1", 3],
		["branch4", "branch3", 2],
	] as const) {
		await dragAndDropByLocator(
			page,
			branchHeaders.filter({ hasText: branch }),
			branchHeaders.filter({ hasText: parent }),
			{ force: true, position: { x: 120, y: -10 } },
		);
		await expect(stack(page)).toHaveCount(remainingStacks);
	}

	await publishReview(page, "branch1", "Description for branch1");
	server.setListed(true);
	await publishReview(page, "branch2", "Description for branch2");
	await publishReview(page, "branch3", "Description for branch3");
	await publishReview(page, "branch4", "Description for branch4");

	await expectReviews(server, 4, (reviews) => {
		expectStackMembership(reviews[0], [42, 43], [44, 45]);
		expectStackMembership(reviews[1], [42, 43], [44, 45]);
		expectStackMembership(reviews[2], [44, 45], [42, 43]);
		expectStackMembership(reviews[3], [44, 45], [42, 43]);
	});
	const reviewUpdatesBeforePush = server.getReviewUpdateCount();

	writeFileSync(
		gitbutler.pathInWorkdir("local-clone/d_file"),
		"branch4 change after publishing reviews\n",
		{ flag: "a" },
	);
	const branch4Stack = stack(page, "branch4");
	await branch4Stack.getByTestId("start-commit-button").click();
	await page.getByTestId("commit-drawer-title-input").fill("branch4: post-review change");
	await page.getByTestId("commit-drawer-action-button").click();
	await gitbutler.runScript("push-branch.sh", ["branch4"]);
	await expect.poll(() => server.getReviewUpdateCount()).toBeGreaterThan(reviewUpdatesBeforePush);

	const reviews = server.getReviews();
	expectStackMembership(reviews[0], [42, 43], [44, 45]);
	expectStackMembership(reviews[1], [42, 43], [44, 45]);
	expectStackMembership(reviews[2], [44, 45], [42, 43]);
	expectStackMembership(reviews[3], [44, 45], [42, 43]);
});

async function combineBranchesIntoStack(page: Page) {
	const branchHeaders = page.getByTestId("branch-header");
	await expect(stack(page)).toHaveCount(4);

	for (const [branch, parent, remainingStacks] of [
		["branch2", "branch1", 3],
		["branch3", "branch2", 2],
		["branch4", "branch3", 1],
	] as const) {
		await dragAndDropByLocator(
			page,
			branchHeaders.filter({ hasText: branch }),
			branchHeaders.filter({ hasText: parent }),
			{ force: true, position: { x: 120, y: -10 } },
		);
		await expect(stack(page)).toHaveCount(remainingStacks);
	}
}

async function publishReview(page: Page, branch: string, description: string) {
	const header = page.getByTestId("branch-header").filter({ hasText: branch });
	const headerWrapper = header.locator("..");
	await headerWrapper.getByTestId("create-review-button").click();
	await waitForTestId(page, "create-review-box");
	await textEditorFillByTestId(page, "create-review-box-description-input", description);
	await clickByTestId(page, "create-review-box-create-button");
	await expect(headerWrapper.getByTestId("create-review-button")).toHaveCount(0);
}

async function setDescriptionPolicy(page: Page, gitbutler: GitButler, policy: "top" | "disabled") {
	await clickByTestId(page, "chrome-sidebar-project-settings-button");
	await waitForTestId(page, "project-settings-modal");
	const select = page.getByTestId("review-stacking-description-select");
	await expect(select).toBeVisible();
	await select.scrollIntoViewIfNeeded();
	await select.click();
	await page
		.getByTestId(`review-stacking-description-option-${policy}`)
		.getByRole("button")
		.click();
	await assertGitConfigValue(POLICY_KEY, policy, gitbutler.pathInWorkdir("local-clone"));
	await page.keyboard.press("Escape");
	await expect(page.getByTestId("project-settings-modal")).toHaveCount(0);
}

async function expectReviews(
	server: FakeGitHubServer,
	count: number,
	assertions: (reviews: FakeGitHubReview[]) => void,
) {
	await expect
		.poll(
			() => {
				const reviews = server.getReviews();
				if (reviews.length !== count) return false;
				try {
					assertions(reviews);
					return true;
				} catch {
					return false;
				}
			},
			{
				message: `Expected ${count} fake GitHub reviews with synchronized descriptions`,
			},
		)
		.toBe(true);
}

function expectBottomFooter(review: FakeGitHubReview, description: string, part: string) {
	expect(review.body).toMatch(new RegExp(`^${description}`));
	expect(review.body).toContain(part);
	expect(review.body).toMatch(new RegExp(`${escapeRegExp(FOOTER_BOTTOM)}$`));
}

function expectStackOrder(reviews: FakeGitHubReview[], topToBase: number[]) {
	for (const review of reviews) {
		const body = review.body ?? "";
		const positions = topToBase.map((number) => body.indexOf(`#${number}`));
		expect(positions.every((position) => position >= 0)).toBe(true);
		expect(positions).toEqual([...positions].sort((a, b) => a - b));
	}
}

function expectStackMembership(
	review: FakeGitHubReview,
	expectedReviewNumbers: number[],
	unrelatedReviewNumbers: number[],
) {
	const body = review.body ?? "";
	for (const number of expectedReviewNumbers) {
		expect(body, `review #${review.number} should include stack peer #${number}`).toContain(
			`#${number}`,
		);
	}
	for (const number of unrelatedReviewNumbers) {
		expect(
			body,
			`review #${review.number} should not include unrelated review #${number}`,
		).not.toContain(`#${number}`);
	}
}

async function storeFakeGitHubEnterprisePat(page: Page, server: FakeGitHubServer) {
	const response = await page.request.post(
		`http://localhost:${getButlerPort()}/store_github_enterprise_pat`,
		{
			data: { host: server.apiBaseUrl, accessToken: "fake-token" },
		},
	);
	expect(response.ok()).toBe(true);
}

function mirrorFakeCredentialForCli(gitbutler: GitButler) {
	const credentialPath = gitbutler.pathInWorkdir("../config/git-credentials");
	const serverCredential = readFileSync(credentialPath, "utf8");
	const cliCredential = serverCredential.replace("development-", "com.gitbutler.app.dev-");
	writeFileSync(credentialPath, `${serverCredential.trimEnd()}\n${cliCredential}`);
}

function escapeRegExp(value: string): string {
	return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
