import { spawn } from "node:child_process";
import http, { type IncomingMessage, type ServerResponse } from "node:http";
import path from "node:path";
import type { Socket } from "node:net";

export type FakeGitHubOptions = {
	headRepoPath?: string;
	forkRepoPath?: string;
	baseRepoPath?: string;
	sourceBranch?: string;
	owner?: string;
	repo?: string;
	repoOwner?: string;
	reviewNumber?: number;
	title?: string;
	isFork?: boolean;
	listed?: boolean;
	failReviewUpdates?: boolean;
	failGitPushes?: boolean;
};

type ResolvedFakeGitHubOptions = {
	headRepoPath: string;
	baseRepoPath: string;
	sourceBranch: string;
	owner: string;
	repo: string;
	repoOwner: string;
	reviewNumber: number;
	title: string;
	isFork: boolean;
};

export type FakeGitHubServer = {
	apiBaseUrl: string;
	repositoryUrl: string;
	setListed: (listed: boolean) => void;
	getReview: (number: number) => FakeGitHubReview | undefined;
	getReviews: () => FakeGitHubReview[];
	getReviewUpdateCount: () => number;
	getReviewUpdateHistory: (number: number) => ReviewMutation[];
	setReviewUpdatesFailing: (failing: boolean) => void;
	setFailingReviewUpdates: (numbers: number[]) => void;
	setGitPushesFailing: (failing: boolean) => void;
	close: () => Promise<void>;
};

export type FakeGitHubReview = ReturnType<typeof pullRequestPayload>;

export type ReviewMutation = {
	title?: string;
	head?: string;
	base?: string;
	body?: string | null;
	draft?: boolean;
};

export async function startFakeGitHubServer({
	headRepoPath,
	forkRepoPath,
	baseRepoPath,
	sourceBranch = "fork-feature",
	owner = "acme",
	repo = "widgets",
	repoOwner = "Contributor User",
	reviewNumber = 42,
	title = "Fork PR",
	isFork = true,
	listed = true,
	failReviewUpdates = false,
	failGitPushes = false,
}: FakeGitHubOptions): Promise<FakeGitHubServer> {
	const reviewRepoPath = headRepoPath ?? forkRepoPath;
	if (!reviewRepoPath) {
		throw new Error("Fake GitHub review needs a headRepoPath or forkRepoPath");
	}

	const options = {
		headRepoPath: reviewRepoPath,
		baseRepoPath: baseRepoPath ?? reviewRepoPath,
		sourceBranch,
		owner,
		repo,
		repoOwner,
		reviewNumber,
		title,
		isFork,
	};
	const reviews = new Map<number, FakeGitHubReview>([[reviewNumber, pullRequestPayload(options)]]);
	let nextReviewNumber = reviewNumber;
	let reviewUpdateCount = 0;
	const reviewUpdateHistory = new Map<number, ReviewMutation[]>();
	let isListed = listed;
	let reviewUpdatesFailing = failReviewUpdates;
	let failingReviewUpdates = new Set<number>();
	let gitPushesFailing = failGitPushes;

	const sockets = new Set<Socket>();
	const server = http.createServer((request, response) => {
		handleRequest(request, response, options, {
			get listed() {
				return isListed;
			},
			get gitPushesFailing() {
				return gitPushesFailing;
			},
			listReviews() {
				return [...reviews.values()].sort((a, b) => a.number - b.number);
			},
			getReview(number) {
				return reviews.get(number);
			},
			createReview(input) {
				const requestedHead = input.head ?? options.sourceBranch;
				const number = nextReviewNumber++;
				const created = pullRequestPayload(
					{
						...options,
						reviewNumber: number,
						title: input.title ?? options.title,
						sourceBranch: unqualifiedHeadRef(requestedHead),
					},
					requestedHead,
					input,
				);
				reviews.set(number, created);
				return created;
			},
			updateReview(number, input) {
				reviewUpdateCount += 1;
				const history = reviewUpdateHistory.get(number) ?? [];
				history.push(structuredClone(input));
				reviewUpdateHistory.set(number, history);
				if (reviewUpdatesFailing || failingReviewUpdates.has(number)) {
					return { status: "failed" as const };
				}
				const current = reviews.get(number);
				if (!current) return { status: "notFound" as const };
				const updated = {
					...current,
					title: input.title ?? current.title,
					body: input.body === undefined ? current.body : input.body,
					draft: input.draft ?? current.draft,
					base: input.base ? { ...current.base, ref: input.base } : current.base,
					updated_at: "2026-06-01T00:01:00Z",
				};
				reviews.set(number, updated);
				return { status: "updated" as const, review: updated };
			},
		}).catch((error: unknown) => {
			response.writeHead(500, { "Content-Type": "application/json" });
			response.end(JSON.stringify({ message: String(error) }));
		});
	});
	server.on("connection", (socket) => {
		sockets.add(socket);
		socket.on("close", () => sockets.delete(socket));
	});

	await new Promise<void>((resolve) => {
		server.listen(0, "127.0.0.1", resolve);
	});

	const address = server.address();
	if (!address || typeof address === "string") {
		throw new Error("Fake GitHub server did not bind to a TCP port");
	}

	const root = `http://127.0.0.1:${address.port}`;
	return {
		apiBaseUrl: `${root}/api/v3`,
		repositoryUrl: `${root}/${owner}/${repo}.git`,
		setListed: (value) => {
			isListed = value;
		},
		getReview: (number) => {
			const review = reviews.get(number);
			return review ? structuredClone(review) : undefined;
		},
		getReviews: () =>
			[...reviews.values()]
				.sort((a, b) => a.number - b.number)
				.map((review) => structuredClone(review)),
		getReviewUpdateCount: () => reviewUpdateCount,
		getReviewUpdateHistory: (number) =>
			(reviewUpdateHistory.get(number) ?? []).map((update) => structuredClone(update)),
		setReviewUpdatesFailing: (failing) => {
			reviewUpdatesFailing = failing;
		},
		setFailingReviewUpdates: (numbers) => {
			failingReviewUpdates = new Set(numbers);
		},
		setGitPushesFailing: (failing) => {
			gitPushesFailing = failing;
		},
		close: async () =>
			await new Promise<void>((resolve, reject) => {
				server.close((error) => (error ? reject(error) : resolve()));
				for (const socket of sockets) socket.destroy();
			}),
	};
}

async function handleRequest(
	request: IncomingMessage,
	response: ServerResponse,
	options: ResolvedFakeGitHubOptions,
	state: {
		readonly listed: boolean;
		readonly gitPushesFailing: boolean;
		listReviews: () => FakeGitHubReview[];
		getReview: (number: number) => FakeGitHubReview | undefined;
		createReview: (input: ReviewMutation) => FakeGitHubReview;
		updateReview: (
			number: number,
			input: ReviewMutation,
		) =>
			| { status: "updated"; review: FakeGitHubReview }
			| { status: "notFound" }
			| { status: "failed" };
	},
) {
	const url = new URL(request.url ?? "/", "http://127.0.0.1");
	const pullPath = `/api/v3/repos/${options.owner}/${options.repo}/pulls`;

	if (request.method === "GET" && url.pathname === "/api/v3/user") {
		return json(response, {
			id: 1,
			login: "e2e-user",
			name: "E2E User",
			email: null,
			avatar_url: null,
			type: "User",
		});
	}

	if (request.method === "GET" && url.pathname === pullPath) {
		return json(response, state.listed ? state.listReviews() : []);
	}

	const reviewNumber = reviewNumberFromPath(url.pathname, pullPath);
	if (request.method === "GET" && reviewNumber !== undefined) {
		const review = state.getReview(reviewNumber);
		return review ? json(response, review) : json(response, { message: "Not Found" }, 404);
	}

	if (request.method === "POST" && url.pathname === pullPath) {
		const input = JSON.parse(await readBody(request)) as ReviewMutation;
		return json(response, state.createReview(input), 201);
	}

	if (request.method === "PATCH" && reviewNumber !== undefined) {
		const input = JSON.parse(await readBody(request)) as ReviewMutation;
		const result = state.updateReview(reviewNumber, input);
		if (result.status === "failed") {
			return json(response, { message: "Configured review update failure" }, 500);
		}
		return result.status === "updated"
			? json(response, result.review)
			: json(response, { message: "Not Found" }, 404);
	}

	const repositoryPath = `/${options.owner}/${options.repo}.git`;
	if (url.pathname === repositoryPath || url.pathname.startsWith(`${repositoryPath}/`)) {
		if (
			state.gitPushesFailing &&
			request.method === "POST" &&
			url.pathname === `${repositoryPath}/git-receive-pack`
		) {
			request.resume();
			response.writeHead(500, { Connection: "close", "Content-Type": "text/plain" });
			response.end("Configured Git push failure");
			return;
		}
		return await serveGitRequest(request, response, url, repositoryPath, options.baseRepoPath);
	}

	response.writeHead(404, { "Content-Type": "application/json" });
	response.end(
		JSON.stringify({ message: `No fake GitHub route for ${request.method} ${url.pathname}` }),
	);
}

function pullRequestPayload(
	{
		headRepoPath,
		sourceBranch,
		owner,
		repo,
		repoOwner,
		reviewNumber,
		title,
		isFork,
	}: ResolvedFakeGitHubOptions,
	headLabel = `${repoOwner}:${sourceBranch}`,
	input: ReviewMutation = {},
) {
	return {
		html_url: `http://127.0.0.1/${owner}/${repo}/pull/${reviewNumber}`,
		number: reviewNumber,
		title,
		body: input.body ?? null,
		user: null,
		labels: [],
		draft: input.draft ?? false,
		merge_commit_sha: null,
		head: {
			label: headLabel,
			ref: sourceBranch,
			sha: "0000000000000000000000000000000000000000",
			repo: {
				ssh_url: headRepoPath,
				clone_url: headRepoPath,
				owner: {
					id: 2,
					login: repoOwner,
					name: repoOwner,
					email: null,
					avatar_url: null,
					type: "User",
				},
				fork: isFork,
			},
		},
		base: {
			ref: input.base ?? "master",
			sha: "0000000000000000000000000000000000000000",
			repo: {
				ssh_url: `git@example.com:${owner}/${repo}.git`,
				clone_url: `https://example.com/${owner}/${repo}.git`,
				owner: {
					id: 3,
					login: owner,
					name: owner,
					email: null,
					avatar_url: null,
					type: "Organization",
				},
				fork: false,
			},
		},
		created_at: "2026-06-01T00:00:00Z",
		updated_at: "2026-06-01T00:00:00Z",
		merged_at: null,
		closed_at: null,
		requested_reviewers: [],
	};
}

function reviewNumberFromPath(pathname: string, pullPath: string): number | undefined {
	if (!pathname.startsWith(`${pullPath}/`)) return undefined;
	const value = Number(pathname.slice(pullPath.length + 1));
	return Number.isSafeInteger(value) ? value : undefined;
}

function unqualifiedHeadRef(head: string): string {
	return head.slice(head.lastIndexOf(":") + 1);
}

async function serveGitRequest(
	request: IncomingMessage,
	response: ServerResponse,
	url: URL,
	repositoryPath: string,
	localRepositoryPath: string,
) {
	const repositoryParent = path.dirname(localRepositoryPath);
	const repositoryName = path.basename(localRepositoryPath);
	const pathSuffix = url.pathname.slice(repositoryPath.length);
	const child = spawn("git", ["http-backend"], {
		env: {
			...process.env,
			GIT_HTTP_EXPORT_ALL: "1",
			GIT_PROJECT_ROOT: repositoryParent,
			PATH_INFO: `/${repositoryName}${pathSuffix}`,
			QUERY_STRING: url.search.slice(1),
			REQUEST_METHOD: request.method ?? "GET",
			CONTENT_TYPE: request.headers["content-type"] ?? "",
			CONTENT_LENGTH: request.headers["content-length"] ?? "",
			HTTP_GIT_PROTOCOL: headerValue(request.headers["git-protocol"]),
			REMOTE_ADDR: request.socket.remoteAddress ?? "127.0.0.1",
		},
	});

	request.pipe(child.stdin);
	const stdout: Buffer[] = [];
	const stderr: Buffer[] = [];
	child.stdout.on("data", (chunk: Buffer) => stdout.push(chunk));
	child.stderr.on("data", (chunk: Buffer) => stderr.push(chunk));

	const exitCode = await new Promise<number | null>((resolve, reject) => {
		child.on("error", reject);
		child.on("close", resolve);
	});
	if (exitCode !== 0) {
		throw new Error(`git http-backend failed: ${Buffer.concat(stderr).toString("utf8")}`);
	}

	const output = Buffer.concat(stdout);
	const headerEnd = output.indexOf("\r\n\r\n");
	if (headerEnd < 0) throw new Error("git http-backend returned malformed CGI output");
	const headers = output.subarray(0, headerEnd).toString("utf8").split("\r\n");
	let status = 200;
	for (const header of headers) {
		const separator = header.indexOf(":");
		if (separator < 0) continue;
		const name = header.slice(0, separator);
		const value = header.slice(separator + 1).trim();
		if (name.toLowerCase() === "status") status = Number.parseInt(value, 10);
		else response.setHeader(name, value);
	}
	response.setHeader("Connection", "close");
	response.writeHead(status);
	response.end(output.subarray(headerEnd + 4));
}

function headerValue(value: string | string[] | undefined): string {
	return Array.isArray(value) ? (value[0] ?? "") : (value ?? "");
}

async function readBody(request: IncomingMessage): Promise<string> {
	const chunks: Buffer[] = [];
	for await (const chunk of request) {
		chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk));
	}
	return Buffer.concat(chunks).toString("utf8");
}

function json(response: ServerResponse, subject: unknown, status = 200) {
	response.writeHead(status, { Connection: "close", "Content-Type": "application/json" });
	response.end(JSON.stringify(subject));
}
