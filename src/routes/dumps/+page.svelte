<script lang="ts">
	import { onMount } from 'svelte';
	import {
		downloadDumps,
		getAllDumpInfo,
		getLocalDumpVersions,
		importDumps,
		saveLocalDumpVersions,
		type DumpVersions
	} from '$lib/rust-bindings/dumps';
	import { checkedForDumpUpdate } from '$lib/stores';
	import { LoaderCircle } from 'lucide-svelte';
	import { goto } from '$app/navigation';
	import { BaseDirectory, exists, mkdir, remove } from '@tauri-apps/plugin-fs';
	import { tempDir } from '@tauri-apps/api/path';
	import { once } from '@tauri-apps/api/event';

	let importFailed = false;
	let importing = false;

	onMount(async () => {
		try {
			if (!(await exists('game-chronicle', { baseDir: BaseDirectory.Temp }))) {
				await mkdir('game-chronicle', { baseDir: BaseDirectory.Temp });
			}
			const localDumpVersions = await getLocalDumpVersions();
			const allDumpsInfo = await getAllDumpInfo();
			const urlsToDownload = [];
			const dumpVersions: DumpVersions = { games: '', websites: '', platforms: '', covers: '' };
			for (const dumpInfo of allDumpsInfo) {
				const localDumpVersion = localDumpVersions[dumpInfo.name];
				if (localDumpVersion !== dumpInfo.version) {
					urlsToDownload.push(dumpInfo.url);
				}
				dumpVersions[dumpInfo.name] = dumpInfo.version;
			}
			$checkedForDumpUpdate = true;
			if (urlsToDownload.length === 0) {
				goto('/');
				return;
			}
			importing = true;
			const directory = (await tempDir()).concat('/game-chronicle');
			await downloadDumps(allDumpsInfo, directory);
			await importDumps(directory);
			await once('import_finished', async () => {
				await saveLocalDumpVersions(dumpVersions);
				await remove(directory, { baseDir: BaseDirectory.Temp, recursive: true });
				goto('/');
			});
		} catch (e) {
			console.error(e);
			importFailed = true;
		}
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
