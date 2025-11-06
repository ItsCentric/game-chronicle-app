<script lang="ts">
	import { onMount } from 'svelte';
	import { importIgdbDumps } from '$lib/rust-bindings/dumps';
	import { goto } from '$app/navigation';
	import { listen } from '@tauri-apps/api/event';
	import { Progress } from '$lib/components/ui/progress';
	import { load } from '@tauri-apps/plugin-store';

	let importFailed = false;
	let importing = false;
	let progress = 0;
	let total = 0;

	type ImportProgressPayload = {
		step: 'Download' | 'Import';
		status: 'Started' | 'Progress' | 'Completed';
		progress?: number;
		total?: number;
	};

	async function handleStarted(payload: ImportProgressPayload) {
		let path: string;
		if (process.env.NODE_ENV === 'development') {
			path = 'dev/persistent.json';
		} else {
			path = 'persistent.json';
		}
		const store = await load(path);
		store.set('lastDumpUpdate', Date.now());
		if (payload.step === 'Import') importing = true;
		progress = 0;
		total = payload.total || 0;
	}

	function handleProgress(payload: ImportProgressPayload) {
		if (payload.step === 'Import' && (payload.progress ?? 0) % 9000 === 0) {
			progress = payload.progress ?? 0;
		} else if (payload.step === 'Download') {
			progress += payload.progress ?? 0;
		}
		if (payload.total) total = payload.total;
	}

	async function handleCompleted(payload: ImportProgressPayload) {
		if (payload.step === 'Import') {
			await goto('/');
		}
	}

	onMount(() => {
		const unlisten = listen<ImportProgressPayload>('import_progress', async (event) => {
			console.log('Received event:', event);
			const { payload } = event;
			switch (payload.status) {
				case 'Started':
					await handleStarted(payload);
					break;
				case 'Progress':
					handleProgress(payload);
					break;
				case 'Completed':
					await handleCompleted(payload);
					break;
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
			<h1 class="text-xl">Downloading data</h1>
		{:else}
			<h1 class="text-xl">Importing new titles...</h1>
		{/if}
		<Progress value={progress} max={total} class="max-w-xl" />
	{:else}
		<h1 class="text-xl">Failed to import new titles</h1>
		<button class="btn" on:click={() => goto('/')}>Go back</button>
	{/if}
</main>
