<script lang="ts">
	import { settingsSchema } from '$lib/schemas';
	import { OpenDirectoryDialog } from '$lib/wailsjs/go/main/App';
	import { SaveUserSettings, GetUserSettings } from '$lib/wailsjs/go/main/Database';
	import { WindowReloadApp } from '$lib/wailsjs/runtime/runtime';
	import { validator } from '@felte/validator-zod';
	import { createForm } from 'felte';
	import { ArrowLeft, PencilIcon, Trash } from 'lucide-svelte';
	import { onMount } from 'svelte';
	import type { z } from 'zod';

	var reloadApplicationModal: HTMLDialogElement | null;
	const { form, isValid, isDirty, setData, data, setInitialValues, setIsDirty } = createForm<
		z.infer<typeof settingsSchema>
	>({
		initialValues: { processMonitoringEnabled: false, executablePaths: [] },
		onSubmit: async (values) => {
			var newSettings = {
				processMonitoringEnabled: values.processMonitoringEnabled,
				executablePaths: values.executablePaths.join(';')
			};
			await SaveUserSettings(newSettings);
			reloadApplicationModal?.showModal();
		},
		extend: validator({ schema: settingsSchema })
	});

	async function newDirectoryDialog() {
		var result = await OpenDirectoryDialog();
		if (result.selectedDirectory) {
			addPath(result.selectedDirectory);
		}
	}
	async function editDirectoryDialog(pathToEdit: string) {
		var result = await OpenDirectoryDialog();
		if (result.selectedDirectory) {
			removePath(pathToEdit);
			addPath(result.selectedDirectory);
		}
	}
	onMount(async () => {
		var userPreferencesRes = await GetUserSettings();
		if (userPreferencesRes.preferences) {
			var executablePathsString = userPreferencesRes.preferences.executablePaths;
			var executablePathsArray = executablePathsString.split(';');
			executablePathsArray = executablePathsArray.filter((path) => path !== '');
			setInitialValues({
				processMonitoringEnabled: userPreferencesRes.preferences.processMonitoringEnabled,
				executablePaths: executablePathsArray
			});
			setData('executablePaths', executablePathsArray);
			setData('processMonitoringEnabled', userPreferencesRes.preferences.processMonitoringEnabled);
		}
	});
	function removePath(path: string) {
		var newExecutablePathsArray = $data.executablePaths.filter((p) => p !== path);
		setData('executablePaths', newExecutablePathsArray);
		setIsDirty(true);
	}
	function addPath(path: string) {
		setData('executablePaths', [...$data.executablePaths, path]);
		setIsDirty(true);
	}
</script>

<main class="w-full h-full p-12 flex-col justify-center items-center">
	<div class="flex gap-2 mb-8 items-center">
		<a href="/" class="btn btn-circle btn-ghost"><ArrowLeft size={32} /></a>
		<h1 class="text-3xl font-bold">Settings</h1>
	</div>
	<form use:form class="flex flex-col gap-8">
		<section class="flex flex-col gap-2">
			<div class="flex justify-between mb-2">
				<h2 class="text-2xl font-bold">Monitoring</h2>
			</div>
			<div class="form-control">
				<label class="label cursor-pointer">
					<span class="label-text">Enable Game Monitoring</span>
					<input
						type="checkbox"
						class="toggle"
						name="processMonitoringEnabled"
						checked={$data.processMonitoringEnabled}
					/>
				</label>
			</div>
			<div class="flex justify-between items-center mb-2">
				<h3 class="text-xl font-bold">Monitoring Paths</h3>
				<button type="button" class="btn" on:click={newDirectoryDialog}>Add New Path</button>
			</div>
			{#if $data.executablePaths.length !== 0}
				<div class="w-full border border-b-0 flex flex-col items-center rounded-md">
					{#each $data.executablePaths as path}
						<div class="w-full border-b flex items-center">
							<p class="flex-1 p-4">{path}</p>
							<div class="flex gap-2 p-4">
								<button
									type="button"
									class="btn"
									on:click={async () => await editDirectoryDialog(path)}
									><PencilIcon height={12} width={12} /></button
								>
								<button type="button" class="btn" on:click={() => removePath(path)}
									><Trash width={12} height={12} /></button
								>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</section>
		<dialog bind:this={reloadApplicationModal} class="modal">
			<div class="modal-box">
				<h3 class="font-bold text-lg">Take a Refresher</h3>
				<p class="py-4">
					Looks like you changed some settings that require a reload to go into effect, please
					reload at your earliest convenience.
				</p>
				<div class="modal-action">
					<button class="btn" on:click={() => WindowReloadApp()}>Reload now</button>
					<form method="dialog">
						<button class="btn">I'll wait</button>
					</form>
				</div>
			</div>
			<form method="dialog" class="modal-backdrop">
				<button class="cursor-default">close</button>
			</form>
		</dialog>
		<button type="submit" class="btn" disabled={!$isValid || !$isDirty}>Apply changes</button>
	</form>
</main>
