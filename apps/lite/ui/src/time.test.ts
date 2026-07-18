import { describe, expect, it, vi } from "vitest";

// Node v22 is missing this API, causing the test suite to throw on importing the module.
vi.hoisted(() => {
	Object.defineProperty(Intl, "DurationFormat", {
		value: class DurationFormat {},
	});
});

import { formatRelativeTimeWith } from "./time.ts";

describe("formatRelativeTime", () => {
	const now = 1_800_000_000_000;
	const formatRelativeTime = formatRelativeTimeWith(
		new Intl.RelativeTimeFormat("en", { numeric: "always", style: "long" }),
	);

	it("formats seconds", () => {
		expect(formatRelativeTime(now - 2_000, now)).toMatchInlineSnapshot(`"2 seconds ago"`);
	});

	it("formats minutes", () => {
		expect(formatRelativeTime(now - 2 * 60_000, now)).toMatchInlineSnapshot(`"2 minutes ago"`);
	});

	it("formats hours", () => {
		expect(formatRelativeTime(now - 2 * 60 * 60_000, now)).toMatchInlineSnapshot(`"2 hours ago"`);
	});

	it("formats days", () => {
		expect(formatRelativeTime(now - 2 * 24 * 60 * 60_000, now)).toMatchInlineSnapshot(
			`"2 days ago"`,
		);
	});

	it("formats months", () => {
		expect(formatRelativeTime(now - 2 * 30 * 24 * 60 * 60_000, now)).toMatchInlineSnapshot(
			`"2 months ago"`,
		);
	});

	it("formats years", () => {
		expect(formatRelativeTime(now - 2 * 365 * 24 * 60 * 60_000, now)).toMatchInlineSnapshot(
			`"2 years ago"`,
		);
	});
});
