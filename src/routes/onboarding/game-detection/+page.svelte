<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { ArrowLeft, PencilIcon, Plus, Trash } from 'lucide-svelte';
	import gameDetectionShowcase from '$lib/assets/game-detection-showcase.gif';
	import { getUserSettings, saveUserSettings } from '$lib/rust-bindings/helpers';
	import * as Table from '$lib/components/ui/table';
	import * as Form from '$lib/components/ui/form';
	import type { PageData } from './$types';
	import { gameDetectionSchema } from '$lib/schemas';
	import { superForm } from 'sveltekit-superforms';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import { Switch } from '$lib/components/ui/switch';
	import { open } from '@tauri-apps/plugin-dialog';

	export let data: PageData;

	const gameDetectionForm = superForm(data.form, {
		SPA: true,
		validators: zodClient(gameDetectionSchema),
		onUpdate: async ({ form }) => {
			if (!form.valid) return;
			const settings = await getUserSettings();
			settings.executable_paths = form.data.executablePaths.join(';');
			settings.process_monitoring.enabled = form.data.processMonitoringEnabled;
			await saveUserSettings(settings);
		}
	});
	const { form: gameDetectionFormData, enhance } = gameDetectionForm;

	async function newDirectoryDialog() {
		const selectedDirectory = await open({
			directory: true,
			multiple: false
		});
		if (selectedDirectory) {
			addPath(selectedDirectory as string);
		}
	}
	async function editDirectoryDialog(pathToEdit: string) {
		const selectedDirectory = await open({
			directory: true,
			multiple: false
		});
		if (selectedDirectory) {
			removePath(pathToEdit);
			addPath(selectedDirectory as string);
		}
	}
	function removePath(path: string) {
		$gameDetectionFormData.executablePaths = $gameDetectionFormData.executablePaths.filter(
			(p) => p !== path
		);
	}
	function addPath(path: string) {
		$gameDetectionFormData.executablePaths = [...$gameDetectionFormData.executablePaths, path];
	}
</script>

<main class="max-w-prose mx-auto py-16">
	<div class="relative mb-4">
		<Button
			href="/onboarding/twitch"
			class="absolute top-2 -translate-x-full -left-4"
			variant="ghost"
			size="icon"><ArrowLeft size={48} /></Button
		>
		<h1 class="text-3xl font-heading font-bold">Enable game detection</h1>
		<p>
			Game detection makes it easier to log your gaming sessions by automatically detecting the
			games you play.
		</p>
	</div>
	<img src={gameDetectionShowcase} alt="Game detection showcase" class="mb-4 w-full" />
	<form method="post" class="flex flex-col gap-2" use:enhance>
		<Form.Field name="processMonitoringEnabled" form={gameDetectionForm}>
			<Form.Control let:attrs>
				<div class="flex items-center justify-between">
					<Form.Label>Enable Game Detection</Form.Label>
					<Switch
						{...attrs}
						includeInput
						bind:checked={$gameDetectionFormData.processMonitoringEnabled}
					/>
				</div>
			</Form.Control>
			<Form.FieldErrors />
		</Form.Field>
		{#if $gameDetectionFormData.processMonitoringEnabled}
			<div class="flex justify-between items-center my-2">
				<h3 class="text-xl font-heading font-bold">Monitoring Paths</h3>
				<Button type="button" on:click={newDirectoryDialog} size="sm">
					<Plus size="1.5em" class="mr-1" />
					<p>Add Path</p>
				</Button>
			</div>
			{#if $gameDetectionFormData.executablePaths.length !== 0}
				<Table.Root>
					<Table.Caption>Edit the system paths that should be monitored here.</Table.Caption>
					<Table.Header>
						<Table.Row>
							<Table.Head>Path</Table.Head>
							<Table.Head class="text-right">Actions</Table.Head>
						</Table.Row>
					</Table.Header>
					<Table.Body>
						{#each $gameDetectionFormData.executablePaths as path}
							<Table.Row>
								<Table.Cell class="w-3/4">{path}</Table.Cell>
								<Table.Cell class="text-right">
									<Button
										type="button"
										size="icon"
										class="mr-1"
										on:click={async () => await editDirectoryDialog(path)}
										><PencilIcon size={16} /></Button
									>
									<Button type="button" size="icon" on:click={() => removePath(path)}
										><Trash size={16} /></Button
									>
								</Table.Cell>
							</Table.Row>
						{/each}
					</Table.Body>
				</Table.Root>
			{:else}
				<Table.Root class="relative">
					<Table.Header>
						<Table.Row>
							<Table.Head>Path</Table.Head>
							<Table.Head class="text-right">Actions</Table.Head>
						</Table.Row>
					</Table.Header>
					<Table.Body>
						{#each Array(3) as _}
							<Table.Row>
								<Table.Cell class="w-3/4"
									><span class="w-64 h-4 bg-white/5 block rounded-xl" /></Table.Cell
								>
								<Table.Cell class="text-right">
									<Button type="button" size="icon" class="mr-1" disabled
										><PencilIcon size={16} /></Button
									>
									<Button type="button" size="icon" disabled><Trash size={16} /></Button>
								</Table.Cell>
							</Table.Row>
						{/each}
					</Table.Body>
					<div
						class="absolute top-0 left-0 bg-black/30 rounded-xl w-full h-full flex justify-center items-center"
					>
						<p class="text-lg font-semibold font-heading">No paths found, try adding some!</p>
					</div>
					<Table.Caption
						>You can edit the system paths that should be monitored by game detection here.</Table.Caption
					>
				</Table.Root>
			{/if}
		{/if}
		<Form.Button class="self-end">Next</Form.Button>
	</form>
</main>
