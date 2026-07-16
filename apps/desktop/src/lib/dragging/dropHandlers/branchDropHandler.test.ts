import {
	acceptsSameStackBranchDrop,
	BranchDropData,
} from "$lib/dragging/dropHandlers/branchDropHandler";
import { describe, expect, test } from "vitest";

function branchDropData(branchName: string, numberOfCommits: number): BranchDropData {
	return new BranchDropData("stack-1", branchName, false, 2, numberOfCommits, undefined, []);
}

describe("acceptsSameStackBranchDrop", () => {
	test("accepts empty branch reorders", () => {
		const data = branchDropData("source", 0);

		expect(acceptsSameStackBranchDrop(data, "target")).toBe(true);
	});

	test("accepts non-empty branch reorders", () => {
		const data = branchDropData("source", 1);

		expect(acceptsSameStackBranchDrop(data, "target")).toBe(true);
	});

	test("rejects self moves", () => {
		const data = branchDropData("source", 1);

		expect(acceptsSameStackBranchDrop(data, "source")).toBe(false);
	});
});
