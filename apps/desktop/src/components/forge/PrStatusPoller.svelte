<script lang="ts">
	import { PR_SERVICE } from "$lib/forge/prService.svelte";
	import { getPollingInterval } from "$lib/forge/shared/progressivePolling";
	import { inject } from "@gitbutler/core/context";

	type Props = {
		projectId: string;
		number: number;
	};

	const { projectId, number }: Props = $props();
	const prService = inject(PR_SERVICE);

	let elapsedMs = $state<number>(0);
	let isClosed = $state(false);
	let hasPollError = $state(false);

	let pollingInterval = $derived(getPollingInterval(elapsedMs, isClosed, hasPollError));

	const prQuery = $derived(
		prService.get(projectId, number, { subscriptionOptions: { pollingInterval } }),
	);

	$effect(() => {
		const result = prQuery.result;
		const pr = result?.data;

		// Back off while the PR query is failing (offline, or the shared GitHub
		// token is rate-limited) so we don't amplify the failure; a successful
		// fetch clears it. Kept as state, not a $derived off prQuery, to avoid a
		// reactive cycle with pollingInterval.
		hasPollError = result?.isError ?? false;

		if (pr) {
			const lastUpdatedMs = Date.parse(pr.modifiedAt);
			isClosed = !!pr.closedAt;
			elapsedMs = Date.now() - lastUpdatedMs;
		}
	});
</script>
