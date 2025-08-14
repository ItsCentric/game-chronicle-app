<script lang="ts">
	import { onMount } from 'svelte';
	import { importIgdbDumps } from '$lib/rust-bindings/dumps';
	import { checkedForDumpUpdate } from '$lib/stores';
	import { LoaderCircle } from 'lucide-svelte';
	import { goto } from '$app/navigation';
	import { listen } from '@tauri-apps/api/event';

	let importFailed = false;
	let importing = false;

	type ImportProgressPayload = {
		step: 'Download' | 'Import';
		status: 'Started' | 'Completed';
	};

	onMount(() => {
		const unlisten = listen<ImportProgressPayload>('import_progress', async (event) => {
			const { payload } = event;
			if (payload.step === 'Import' && payload.status === 'Started') importing = true;
			else if (payload.step === 'Import' && payload.status === 'Completed') {
				$checkedForDumpUpdate = true;
				await goto('/');
			}
		});
		importIgdbDumps().catch((e) => {
			console.error(JSON.stringify(e));
			importFailed = true;
		});

		return () => unlisten.then((f) => f());
	});
</script>

<main class="flex flex-col gap-2 justify-center items-center h-full">
	{#if !importFailed}
		{#if !importing}
			<h1 class="text-xl">Checking for new titles...</h1>
		{:else}
			<h1 class="text-xl">Importing new titles...</h1>
		{/if}
		<LoaderCircle size={32} class="animate-spin w-16" />
	{:else}
		<h1 class="text-xl">Failed to import new titles</h1>
		<button
			class="btn"
			on:click={() => {
				$checkedForDumpUpdate = true;
				goto('/');
			}}>Go back</button
		>
	{/if}
</main>
