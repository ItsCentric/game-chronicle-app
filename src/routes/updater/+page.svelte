<script lang="ts">
	import { tweened } from 'svelte/motion';
	import type { PageData } from './$types';
	import { cubicOut } from 'svelte/easing';
	import Button from '$lib/components/ui/button/button.svelte';
	import { relaunch } from '@tauri-apps/plugin-process';

	export let data: PageData;
	const downloadProgress = tweened(0, { duration: 2500, easing: cubicOut });
	let updateSize = 0;
	data.update?.downloadAndInstall((downloadEvent) => {
		switch (downloadEvent.event) {
			case 'Started':
				updateSize = downloadEvent.data.contentLength ?? 0;
				break;
			case 'Progress':
				downloadProgress.set(downloadEvent.data.chunkLength / updateSize);
				break;
			case 'Finished':
				downloadProgress.set(1);
				break;
		}
	});
</script>

<main class="w-full relative h-full flex justify-center items-center">
	<div class="flex flex-col gap-2">
		<h1 class="font-heading text-2xl font-bold">Game Chronicle</h1>
		<div class="h-8">
			<span class="h-full float-left bg-green-400" style={`width: ${$downloadProgress * 100}%`} />
		</div>
		<p class="text-sm self-start">Downloading version {data.update?.version}</p>
		{#if $downloadProgress === 1}
			<Button on:click={async () => await relaunch()}>Restart</Button>
		{/if}
	</div>
	<p class="absolute right-2 bottom-2 text-sm">{data.update?.currentVersion}</p>
</main>
