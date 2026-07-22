import {
	generateFooter,
	STACKING_FOOTER_BOUNDARY_BOTTOM,
	STACKING_FOOTER_BOUNDARY_TOP,
	updateBody,
} from "$lib/forge/shared/prFooter";
import { describe, expect, test } from "vitest";

// The desktop receives stack segments child -> parent, i.e. [top, ..., base]
// (104 is the tip, 100 the base). The footer must render top-first with the base
// numbered 1, and "part X of N" must match each PR's position badge.
describe("generateFooter", () => {
	const topToBase = [104, 103, 102, 101, 100];

	function listLines(footer: string) {
		return footer.split("\n").filter((l) => l.startsWith("- "));
	}

	test("lists the stack top-first and numbers the base 1", () => {
		const footer = generateFooter(100, topToBase, "#");
		const lines = listLines(footer);

		expect(lines[0]).toContain("#104");
		expect(lines.at(-1)).toContain("#100");
		expect(footer).toContain("<kbd>&nbsp;1&nbsp;</kbd> #100");
		expect(footer).toContain("<kbd>&nbsp;5&nbsp;</kbd> #104");
	});

	test("'part X of N' matches the current PR's position badge", () => {
		expect(generateFooter(100, topToBase, "#")).toContain("part 1 of 5 in a stack");
		expect(generateFooter(104, topToBase, "#")).toContain("part 5 of 5 in a stack");
		expect(generateFooter(102, topToBase, "#")).toContain("part 3 of 5 in a stack");
	});

	test("marks only the current PR with the indicator", () => {
		const footer = generateFooter(102, topToBase, "#");
		const lines = listLines(footer);

		expect(lines.find((l) => l.includes("#102"))).toContain("👈");
		expect(lines.find((l) => l.includes("#101"))).not.toContain("👈");
	});
});

describe("updateBody", () => {
	test("adds no footer when the stack has a single PR", () => {
		expect(updateBody("My description", 123, [123], "#")).toBe("My description");
	});

	test("removes an existing footer when the stack shrinks to a single PR", () => {
		const body = `My description\n\n${generateFooter(123, [124, 123], "#")}`;
		expect(body).toContain("in a stack");
		expect(updateBody(body, 123, [123], "#")).toBe("My description");
	});

	test("preserves content after the footer when removing it", () => {
		const body = `Head\n\n${STACKING_FOOTER_BOUNDARY_TOP}\nfooter\n${STACKING_FOOTER_BOUNDARY_BOTTOM}\n\nTail`;
		expect(updateBody(body, 123, [123], "#")).toBe("Head\n\nTail");
	});

	test("adds a footer when the stack has multiple PRs", () => {
		const result = updateBody("My description", 123, [124, 123], "#");
		expect(result).toContain("My description");
		expect(result).toContain("part 1 of 2 in a stack");
	});

	test("leaves a footer-less body untouched instead of rewriting it", () => {
		const body = "\nMy description with trailing space \n";
		expect(updateBody(body, 123, [123], "#")).toBe(body);
	});

	test("does not drop content when only the top boundary is present", () => {
		const body = `Intro\n\n${STACKING_FOOTER_BOUNDARY_TOP}\ndangling text`;
		expect(updateBody(body, 123, [123], "#")).toBe(body);
	});
});
