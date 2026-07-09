<script lang="ts">
	import { PR_SERVICE } from "$lib/forge/prService.svelte";
	import { createPollBackoff } from "$lib/forge/shared/pollErrorBackoff.svelte";
	import { inject } from "@gitbutler/core/context";

	type Props = {
		projectId: string;
		number: number;
	};

	const { projectId, number }: Props = $props();
	const prService = inject(PR_SERVICE);

	let elapsedMs = $state<number>(0);
	let isClosed = $state(false);

	// Backs polling off while the PR query is failing (offline, or the shared
	// GitHub token is rate-limited) and restores the schedule on recovery.
	const backoff = createPollBackoff({
		getResult: () => prQuery.result,
		getElapsedMs: () => elapsedMs,
		getShouldStop: () => isClosed,
	});
	const pollingInterval = $derived(backoff.pollingInterval);

	const prQuery = $derived(
		prService.get(projectId, number, { subscriptionOptions: { pollingInterval } }),
	);

	$effect(() => {
		const result = prQuery.result;
		const pr = result?.data;

		if (pr) {
			const lastUpdatedMs = Date.parse(pr.modifiedAt);
			isClosed = !!pr.closedAt;
			elapsedMs = Date.now() - lastUpdatedMs;
		}
	});
</script>
