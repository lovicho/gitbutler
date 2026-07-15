import { commitOperand, type CommitOperand, type Operand } from "#ui/operands.ts";
import type { Commit, ForgeInfo } from "@gitbutler/but-sdk";

export const shortCommitId = (commitId: string): string => commitId.slice(0, 7);

export const commitTitle = (input: string): string | undefined => {
	const trimmed = input.trim();
	const _title = trimmed.split("\n")[0];
	const title = _title === "" ? undefined : _title;
	return title;
};

export const commitBody = (input: string): string | undefined => {
	const trimmed = input.trim();
	const _body = trimmed.includes("\n") ? trimmed.slice(trimmed.indexOf("\n") + 1).trim() : "";
	const body = _body === "" ? undefined : _body;
	return body;
};

export const commitIsDiverged = (commit: Commit): boolean =>
	commit.state.type === "LocalAndRemote" && commit.state.subject !== commit.id;

export const rewrittenCommitOperand = ({
	commit,
	replacedCommits,
}: {
	commit: CommitOperand;
	replacedCommits: Record<string, string>;
}): CommitOperand | null => {
	const commitId = replacedCommits[commit.commitId];
	if (commitId === undefined) return null;

	return { commitId };
};

export const rewrittenCommitSelection = ({
	selection,
	replacedCommits,
}: {
	selection: Operand | null;
	replacedCommits: Record<string, string>;
}): Operand | null => {
	if (selection?._tag !== "Commit") return selection;

	const commit = rewrittenCommitOperand({
		commit: selection,
		replacedCommits,
	});
	if (!commit) return selection;

	return commitOperand(commit);
};

type ForgeUrlFreshness = "fresh" | "stale";

/**
 * Builds a forge URL for commits present on the remote. May produce stale URLs for rewritten
 * commits that haven't been pushed yet.
 */
export const commitForgeUrl = (
	commit: Commit,
	forge: ForgeInfo,
): { url: string; freshness: ForgeUrlFreshness } | null => {
	if (commit.state.type === "LocalOnly") return null;

	const commitId = commit.state.type === "LocalAndRemote" ? commit.state.subject : commit.id;
	return {
		url: `${forge.baseUrl}${forge.commitUrlPath}${commitId}`,
		freshness: "subject" in commit.state && commit.state.subject !== commit.id ? "stale" : "fresh",
	};
};
