<script lang="ts">
	import { settingsSchema } from '$lib/schemas';
	import { OpenDirectoryDialog } from '$lib/wailsjs/go/main/App';
	import { GetUserSettings, SaveUserSettings } from '$lib/wailsjs/go/main/Database';
	import { WindowReloadApp } from '$lib/wailsjs/runtime/runtime';
	import { ArrowLeft, PencilIcon, Trash } from 'lucide-svelte';
	import { defaults, superForm } from 'sveltekit-superforms';
	import * as Form from '$lib/components/ui/form';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Table from '$lib/components/ui/table';
	import { Switch } from '$lib/components/ui/switch';
	import { Button } from '$lib/components/ui/button';
	import { zod } from 'sveltekit-superforms/adapters';
	import { onMount } from 'svelte';

	let openReloadApplicationModal = false;
	const settingsForm = superForm(defaults(zod(settingsSchema)), {
		validators: zod(settingsSchema),
		SPA: true,
		onUpdate: async ({ form }) => {
			if (form.valid) {
				var newSettings = {
					processMonitoringEnabled: form.data.processMonitoringEnabled,
					executablePaths: form.data.executablePaths.join(';')
				};
				await SaveUserSettings(newSettings);
				openReloadApplicationModal = true;
			}
		}
	});
	const { form: settingsFormData, enhance: settingsFormEnhance } = settingsForm;

	onMount(async () => {
		const userPreferencesRes = await GetUserSettings();
		if (!userPreferencesRes.preferences) return;
		const executablePathsString = userPreferencesRes.preferences.executablePaths;
		let executablePathsArray = executablePathsString.split(';');
		executablePathsArray = executablePathsArray.filter((path) => path !== '');
		$settingsFormData.processMonitoringEnabled =
			userPreferencesRes.preferences.processMonitoringEnabled;
		$settingsFormData.executablePaths = executablePathsArray;
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
	function removePath(path: string) {
		$settingsFormData.executablePaths = $settingsFormData.executablePaths.filter((p) => p !== path);
	}
	function addPath(path: string) {
		$settingsFormData.executablePaths = [...$settingsFormData.executablePaths, path];
	}
</script>

<main class="w-full h-full p-12 flex-col justify-center items-center">
	<div class="flex gap-2 mb-8 items-center">
		<Button href="/" variant="ghost"><ArrowLeft size={32} /></Button>
		<h1 class="text-3xl font-bold">Settings</h1>
	</div>
	<form method="post" use:settingsFormEnhance class="flex flex-col gap-8">
		<section class="flex flex-col gap-2">
			<div class="flex justify-between mb-2">
				<h2 class="text-2xl font-bold">Monitoring</h2>
			</div>
			<Form.Field form={settingsForm} name="processMonitoringEnabled">
				<Form.Control let:attrs>
					<div class="flex justify-between items-center">
						<Form.Label>Enable Game Monitoring</Form.Label>
						<Switch
							includeInput
							{...attrs}
							bind:checked={$settingsFormData.processMonitoringEnabled}
						/>
					</div>
				</Form.Control>
			</Form.Field>
			<div class="flex justify-between items-center mb-2">
				<h3 class="text-xl font-bold">Monitoring Paths</h3>
				<Button type="button" on:click={newDirectoryDialog}>Add New Path</Button>
			</div>
			{#if $settingsFormData.executablePaths.length !== 0}
				<Table.Root>
					<Table.Caption>Edit the system paths that should be monitored here.</Table.Caption>
					<Table.Header>
						<Table.Row>
							<Table.Head>Path</Table.Head>
							<Table.Head class="text-right">Actions</Table.Head>
						</Table.Row>
					</Table.Header>
					<Table.Body>
						{#each $settingsFormData.executablePaths as path}
							<Table.Row>
								<Table.Cell class="w-3/4">{path}</Table.Cell>
								<Table.Cell class="text-right">
									<Button type="button" on:click={async () => await editDirectoryDialog(path)}
										><PencilIcon size={12} /></Button
									>
									<Button type="button" on:click={() => removePath(path)}
										><Trash size={12} /></Button
									>
								</Table.Cell>
							</Table.Row>
						{/each}
					</Table.Body>
				</Table.Root>
			{/if}
		</section>
		<Button type="submit">Apply changes</Button>
	</form>
	<Dialog.Root bind:open={openReloadApplicationModal}>
		<Dialog.Content>
			<Dialog.Header>
				<Dialog.Title>Take a Refresher</Dialog.Title>
				<Dialog.Description>
					Looks like you changed some settings that require a reload to go into effect, please
					reload at your earliest convenience.
				</Dialog.Description>
			</Dialog.Header>
			<Dialog.Footer>
				<Button on:click={() => WindowReloadApp()}>Reload now</Button>
				<Button on:click={() => (openReloadApplicationModal = false)}>I'll wait</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>
</main>
