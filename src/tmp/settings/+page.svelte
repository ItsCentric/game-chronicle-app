<script lang="ts">
	import { settingsSchema } from '$lib/schemas';
	import {
		OpenDirectoryDialog,
		GetUserSettings,
		SaveUserSettings,
		GetCurrentUsername
	} from '$lib/wailsjs/go/main/App';
	import { WindowReloadApp } from '$lib/wailsjs/runtime/runtime';
	import { PencilIcon, Plus, Trash } from 'lucide-svelte';
	import { defaults, superForm } from 'sveltekit-superforms';
	import * as Form from '$lib/components/ui/form';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Table from '$lib/components/ui/table';
	import { Switch } from '$lib/components/ui/switch';
	import { Button } from '$lib/components/ui/button';
	import { zod } from 'sveltekit-superforms/adapters';
	import Input from '$lib/components/ui/input/input.svelte';
	import { useMutation, useQuery, useQueryClient } from '@sveltestack/svelte-query';
	import { toast } from 'svelte-sonner';
	import type { main } from '$lib/wailsjs/go/models';

	let openReloadApplicationModal = false;

	const queryClient = useQueryClient();
	const userPreferencesMutation = useMutation(
		'userPreferences',
		async (newSettings: main.UserSettingsData) => {
			const response = await SaveUserSettings(newSettings);
			if (response.error != '') {
				throw new Error(response.error);
			}
			return response.newSettings;
		},
		{
			onSuccess: (data) => {
				queryClient.invalidateQueries('userPreferences');
				if (data.username !== $usernameQuery.data) {
					queryClient.invalidateQueries('username');
				}
				openReloadApplicationModal = true;
			}
		}
	);
	useQuery(
		'userPreferences',
		async () => {
			const response = await GetUserSettings();
			if (response.error) {
				throw new Error(response.error);
			}
			return response.preferences;
		},
		{
			onSuccess: (data) => {
				const executablePathsString = data.executablePaths;
				let executablePathsArray = executablePathsString.split(';');
				executablePathsArray = executablePathsArray.filter((path) => path !== '');
				$settingsFormData.processMonitoringEnabled = data.processMonitoringEnabled;
				$settingsFormData.executablePaths = executablePathsArray;
				$settingsFormData.processMonitoringDirectoryDepth = data.processMonitoringDirectoryDepth;
				if ($settingsFormData.username === '') {
					$settingsFormData.username = data.username;
				} else {
					$settingsFormData.username = $usernameQuery.data ?? '';
				}
			},
			onError: () => {
				toast.error('Failed to load user preferences');
			}
		}
	);
	const usernameQuery = useQuery(
		'username',
		async () => {
			const response = await GetCurrentUsername();
			if (response.error) {
				throw new Error(response.error);
			}
			return response.username;
		},
		{
			onSuccess: (data) => {
				if ($settingsFormData.username === '') {
					$settingsFormData.username = data;
				}
			}
		}
	);
	const settingsForm = superForm(defaults(zod(settingsSchema)), {
		validators: zod(settingsSchema),
		SPA: true,
		onUpdate: async ({ form }) => {
			if (form.valid) {
				var newSettings = {
					processMonitoringEnabled: form.data.processMonitoringEnabled,
					executablePaths: form.data.executablePaths.join(';'),
					username: form.data.username,
					processMonitoringDirectoryDepth: form.data.processMonitoringDirectoryDepth
				};
				toast.promise($userPreferencesMutation.mutateAsync(newSettings), {
					loading: 'Saving new settings...',
					success: 'Settings saved successfully',
					error: 'Failed to save settings'
				});
			}
		}
	});
	const {
		form: settingsFormData,
		enhance: settingsFormEnhance,
		validate: validateSettingsFormField
	} = settingsForm;

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

<main class="w-full h-full py-12 flex-col justify-center container items-center">
	<div class="flex gap-2 mb-8 items-center">
		<h1 class="text-3xl font-heading font-bold">Settings</h1>
	</div>
	<form method="post" use:settingsFormEnhance class="flex flex-col gap-8">
		<section class="flex flex-col gap-2">
			<div class="flex justify-between mb-2">
				<h2 class="text-2xl font-heading font-bold">General</h2>
			</div>
			<Form.Field form={settingsForm} name="username">
				<Form.Control let:attrs>
					<div class="flex justify-between items-center">
						<Form.Label>Username</Form.Label>
						<Input {...attrs} bind:value={$settingsFormData.username} class="max-w-xs" />
					</div>
				</Form.Control>
			</Form.Field>
		</section>
		<section class="flex flex-col gap-2">
			<div class="flex justify-between mb-2">
				<h2 class="text-2xl font-heading font-bold">Monitoring</h2>
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
			<Form.Field form={settingsForm} name="processMonitoringDirectoryDepth">
				<Form.Control let:attrs>
					<div class="flex justify-between items-center">
						<Form.Label>Directory Depth</Form.Label>
						<Input
							{...attrs}
							bind:value={$settingsFormData.processMonitoringDirectoryDepth}
							type="number"
							min="1"
							max="99"
							class="w-16"
							on:change={({ currentTarget }) =>
								validateSettingsFormField('processMonitoringDirectoryDepth', {
									value: parseInt(currentTarget.value)
								})}
						/>
					</div>
				</Form.Control>
			</Form.Field>
			<div class="flex justify-between items-center my-2">
				<h3 class="text-xl font-heading font-bold">Monitoring Paths</h3>
				<Button type="button" on:click={newDirectoryDialog} size="sm">
					<Plus size="1.5em" class="mr-1" />
					<p>Add Path</p>
				</Button>
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
				</Table.Root>
			{/if}
		</section>
		<div class="flex justify-end gap-2">
			<Button type="submit">Save</Button>
			<Button variant="destructive" type="reset" on:click={() => window.history.back()}
				>Cancel</Button
			>
		</div>
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
