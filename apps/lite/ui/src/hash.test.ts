import { hash, combineHashes } from "./hash.ts";
import { describe, expect, it } from "vitest";
import fc from "fast-check";

describe("hash", () => {
	it("is stable", () => {
		expect(hash("foo")).toMatchInlineSnapshot(`193410979`);
		expect(hash("bar")).toMatchInlineSnapshot(`193415156`);
	});

	it("is pure", () => {
		fc.assert(
			fc.property(fc.string(), (x) => {
				expect(hash(x)).toBe(hash(x));
			}),
		);
	});
});

describe("combineHashes", () => {
	it("is stable", () => {
		expect(combineHashes(123, 456)).toMatchInlineSnapshot(`6335`);
		expect(combineHashes(1995, 10)).toMatchInlineSnapshot(`105741`);
	});

	it("is pure", () => {
		fc.assert(
			fc.property(fc.integer(), fc.integer(), (x, y) => {
				expect(combineHashes(x, y)).toBe(combineHashes(x, y));
			}),
		);
	});
});
