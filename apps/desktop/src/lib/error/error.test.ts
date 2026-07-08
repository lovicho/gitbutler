import { emitQueryError } from "$lib/error/error";
import { describe, expect, test, vi } from "vitest";
import type { PostHogWrapper } from "$lib/telemetry/posthog";

function fakePosthog() {
	const capture = vi.fn();
	return { posthog: { capture } as unknown as PostHogWrapper, capture };
}

/**
 * `query:error` is the "something is broken" telemetry stream. Errors the
 * classifier marks `silent` (offline network blips, known noise) are
 * expected states and must stay out of it, while real failures keep
 * flowing through with their severity attached.
 */
describe("emitQueryError", () => {
	test("silent severity is not captured", () => {
		const { posthog, capture } = fakePosthog();
		emitQueryError(
			posthog,
			{ name: "API error", message: "Unable to connect to GitHub.", code: "NetworkError" },
			{ command: "list_ci_checks_silent_test", severity: "silent" },
		);
		expect(capture).not.toHaveBeenCalled();
	});

	test("error severity is captured with severity attached", () => {
		const { posthog, capture } = fakePosthog();
		emitQueryError(
			posthog,
			{ name: "API error", message: "boom", code: "Unknown" },
			{ command: "list_ci_checks_error_test", severity: "error" },
		);
		expect(capture).toHaveBeenCalledWith("query:error", {
			error_title: "API error",
			error_message: "boom",
			error_code: "Unknown",
			command: "list_ci_checks_error_test",
			actionName: undefined,
			severity: "error",
		});
	});

	test("errors without a severity still capture (callers outside the classifier)", () => {
		const { posthog, capture } = fakePosthog();
		emitQueryError(
			posthog,
			{ name: "API error", message: "boom" },
			{ command: "no_severity_test" },
		);
		expect(capture).toHaveBeenCalledOnce();
	});

	test("SilentError name is suppressed regardless of severity", () => {
		const { posthog, capture } = fakePosthog();
		emitQueryError(
			posthog,
			{ name: "SilentError", message: "handled elsewhere" },
			{ command: "silent_error_test", severity: "error" },
		);
		expect(capture).not.toHaveBeenCalled();
	});
});
