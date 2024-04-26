<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { logSchema, statusOptions, type StatusOption } from '$lib/schemas';
	import type { main } from '$lib/wailsjs/go/models';
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { defaults, superForm } from 'sveltekit-superforms';
	import { zod, zodClient } from 'sveltekit-superforms/adapters';
	import * as Form from '$lib/components/ui/form';
	import Combobox from '$lib/components/Combobox.svelte';
	import { Input } from '$lib/components/ui/input';
	import Textarea from '$lib/components/ui/textarea/textarea.svelte';
	import DatePicker from '$lib/components/DatePicker.svelte';
	import { getLocalTimeZone, today } from '@internationalized/date';
	import * as RadioGroup from '$lib/components/ui/radio-group';
	import { Button } from '$lib/components/ui/button';
	import { Checkbox } from '$lib/components/ui/checkbox/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import {
		AuthenticateWithTwitch,
		GetGamesById,
		InsertGameLog,
		InsertExecutableDetails,
		GetLogById,
		UpdateLog
	} from '$lib/wailsjs/go/main/App';
	import { logDataFromForm } from '$lib';
	import { useMutation } from '@sveltestack/svelte-query';

	const searchParams = $page.url.searchParams;
	const isEditing = searchParams.has('id');
	const insertLogMutation = useMutation(async (data: main.LogData) => {
		const insertResponse = await InsertGameLog(data);
		if (parseInt(insertResponse.errors?.length) > 0) {
			throw new Error('Failed to insert log');
		}
		return insertResponse.log;
	});
	const updateLogMutation = useMutation(async (log: { id: number; data: main.LogData }) => {
		const updateLogError = await UpdateLog(log.id, log.data);
		if (updateLogError != '') {
			throw new Error('Failed to update log');
		}
	});
	let selectedGame: main.IgdbGame | null = null;
	const logForm = superForm(defaults(zod(logSchema)), {
		validators: zodClient(logSchema),
		SPA: true,
		onUpdate: async ({ form }) => {
			if (form.valid && selectedGame) {
				const candidateLog = logDataFromForm(selectedGame, form.data);
				if (!isEditing) {
					toast.promise($insertLogMutation.mutateAsync(candidateLog), {
						loading: 'Creating log...',
						success: 'Log created successfully!',
						error: 'Failed to create log'
					});
					goto('/logs');
					const executableName = searchParams.get('executableName');
					const minutesPlayed = searchParams.get('minutesPlayed');
					if (executableName && minutesPlayed) {
						await InsertExecutableDetails({
							executableName: executableName,
							gameId: selectedGame.id,
							minutesPlayed: parseInt(minutesPlayed)
						});
					}
				} else {
					toast.promise(
						$updateLogMutation.mutateAsync({
							id: parseInt(searchParams.get('id') as string),
							data: candidateLog
						}),
						{
							loading: 'Updating log...',
							success: 'Log updated successfully!',
							error: 'Failed to update log'
						}
					);
					goto('/logs');
				}
			}
		}
	});
	const {
		form: logFormData,
		enhance: logEnhance,
		validate: validateLogFormField,
		validateForm: validateLogForm
	} = logForm;
	onMount(async () => {
		if (isEditing) {
			const getLogResponse = await GetLogById(parseInt(searchParams.get('id') as string));
			if (getLogResponse.error) {
				console.error('Failed to get log by ID');
				return;
			}
			$logFormData.status = getLogResponse.log.statusId as StatusOption;
			$logFormData.rating = getLogResponse.log.rating;
			$logFormData.logDate = new Date(getLogResponse.log.date);
			$logFormData.finished = getLogResponse.log.finished;
			$logFormData.timePlayedHours = Math.floor(getLogResponse.log.timePlayedMinutes / 60);
			$logFormData.timePlayedMinutes = getLogResponse.log.timePlayedMinutes % 60;
			$logFormData.notes = getLogResponse.log.notes;
		}
		const gameIdString = searchParams.get('gameId');
		if (!gameIdString) return;
		const tokenRes = await AuthenticateWithTwitch();
		if (!tokenRes.access_token) {
			console.error('Failed to authenticate with Twitch');
		}
		const gameId = parseInt(gameIdString);
		const gameResponse = await GetGamesById([gameId], tokenRes.access_token);
		if (gameResponse.error) {
			console.error('Failed to get game by ID');
		}
		selectedGame = gameResponse.games[0];
		const minutesPlayed = searchParams.get('minutesPlayed');
		if (minutesPlayed) {
			$logFormData.timePlayedHours = Math.floor(parseInt(minutesPlayed) / 60);
			$logFormData.timePlayedMinutes = parseInt(minutesPlayed) % 60;
		}
	});

	let isNewLogFormValid = false;
	$: if ($logFormData)
		validateLogForm({ update: false }).then(
			(superValidated) => (isNewLogFormValid = superValidated.valid)
		);
</script>

<main class="min-h-full container py-8 px-16">
	<div class="mb-4">
		<h1 class="text-3xl font-heading font-bold">{isEditing ? 'Edit' : 'New'} Log</h1>
		{#if !selectedGame}
			<Skeleton class="w-72 h-4 mt-2" />
		{:else}
			<p class="text-gray-500 text-lg font-heading">
				What was it like playing {selectedGame.name}?
			</p>
		{/if}
	</div>
	<form method="post" class="grid-cols-[25%,_1fr] grid gap-4" id="logForm" use:logEnhance>
		<div>
			{#if !selectedGame}
				<Skeleton class="aspect-[3/4] rounded-3xl w-full mb-4" />
			{:else}
				<img
					src={'https://images.igdb.com/igdb/image/upload/t_cover_big/' +
						selectedGame.cover.image_id +
						'.jpg'}
					alt="cover"
					class="aspect-[3/4] rounded-3xl mb-4 w-full"
				/>
			{/if}
			<Form.Field form={logForm} name="status">
				<Form.Control let:attrs>
					<Combobox
						{...attrs}
						options={statusOptions.map((status) => ({ value: status, label: status }))}
						placeholder="Pick a status"
						emptyText="No status found!"
						bind:value={$logFormData.status}
						disabled={!selectedGame}
					/>
				</Form.Control>
			</Form.Field>
		</div>
		<div class="flex flex-col gap-2">
			<div>
				{#if !selectedGame}
					<Skeleton class="w-52 h-6 mb-2" />
				{:else}
					<p class="text-2xl font-heading font-semibold">{selectedGame.name}</p>
				{/if}
				<Form.Fieldset form={logForm} name="rating">
					<RadioGroup.Root
						value={`${$logFormData.rating}`}
						class="flex items-center"
						onValueChange={(newValue) =>
							validateLogFormField('rating', { value: parseInt(newValue) })}
					>
						{#each Array(5) as _, i}
							<Form.Control let:attrs>
								<RadioGroup.Item
									class="hidden"
									disabled={!selectedGame}
									value={`${i + 1}`}
									{...attrs}
								/>
								<Form.Label>
									{#if $logFormData.rating >= i + 1}
										<svg
											xmlns="http://www.w3.org/2000/svg"
											width="24"
											height="24"
											viewBox="0 0 26 26"
										>
											<path
												fill="#eab308"
												d="M25.326 10.137a1.001 1.001 0 0 0-.807-.68l-7.34-1.066l-3.283-6.651c-.337-.683-1.456-.683-1.793 0L8.82 8.391L1.48 9.457a1 1 0 0 0-.554 1.705l5.312 5.178l-1.254 7.31a1.001 1.001 0 0 0 1.451 1.054L13 21.252l6.564 3.451a1 1 0 0 0 1.451-1.054l-1.254-7.31l5.312-5.178a.998.998 0 0 0 .253-1.024z"
											/>
										</svg>
									{:else}
										<svg
											xmlns="http://www.w3.org/2000/svg"
											width="24"
											height="24"
											viewBox="0 0 26 26"
										>
											<path
												fill="#454545"
												d="M25.326 10.137a1.001 1.001 0 0 0-.807-.68l-7.34-1.066l-3.283-6.651c-.337-.683-1.456-.683-1.793 0L8.82 8.391L1.48 9.457a1 1 0 0 0-.554 1.705l5.312 5.178l-1.254 7.31a1.001 1.001 0 0 0 1.451 1.054L13 21.252l6.564 3.451a1 1 0 0 0 1.451-1.054l-1.254-7.31l5.312-5.178a.998.998 0 0 0 .253-1.024z"
											/>
										</svg>
									{/if}
								</Form.Label>
							</Form.Control>
						{/each}
					</RadioGroup.Root>
				</Form.Fieldset>
			</div>
			<Form.Field form={logForm} name="logDate">
				<Form.Control let:attrs>
					<Form.Label>Date</Form.Label>
					<DatePicker
						{...attrs}
						bind:value={$logFormData.logDate}
						placeholder="Log date"
						max={today(getLocalTimeZone())}
						disabled={!selectedGame}
					/>
				</Form.Control>
				<Form.FieldErrors />
			</Form.Field>
			<Form.Field form={logForm} name="finished">
				<Form.Control let:attrs>
					<Form.Label class="mr-2">Finished?</Form.Label>
					<Checkbox {...attrs} bind:checked={$logFormData.finished} disabled={!selectedGame} />
				</Form.Control>
				<Form.FieldErrors />
			</Form.Field>
			<div>
				<p class="text-sm mb-2 pointer-events-none">Time played</p>
				<Form.Field form={logForm} name="timePlayedHours" class="w-14 inline-block mr-1">
					<Form.Control let:attrs>
						<Input
							{...attrs}
							type="number"
							placeholder="HH"
							min="0"
							bind:value={$logFormData.timePlayedHours}
							disabled={!selectedGame}
							data-testid="hours-played"
							on:change={(newValue) => {
								validateLogFormField('timePlayedHours', {
									value: parseInt(newValue.currentTarget.value)
								});
							}}
						/>
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>
				<Form.Field form={logForm} name="timePlayedMinutes" class="w-14 inline-block">
					<Form.Control let:attrs>
						<Input
							{...attrs}
							type="number"
							placeholder="MM"
							min="0"
							max="59"
							disabled={!selectedGame}
							data-testid="minutes-played"
							bind:value={$logFormData.timePlayedMinutes}
							on:change={(newValue) => {
								validateLogFormField('timePlayedMinutes', {
									value: parseInt(newValue.currentTarget.value)
								});
							}}
						/>
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>
			</div>
			<div>
				<Form.Field form={logForm} name="notes">
					<Form.Control let:attrs>
						<Form.Label>Notes</Form.Label>
						<Textarea
							{...attrs}
							placeholder="Notes"
							disabled={!selectedGame}
							bind:value={$logFormData.notes}
						/>
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>
			</div>
		</div>
	</form>
	<div class="float-right">
		<Button
			type="submit"
			form="logForm"
			data-testid="save-log"
			class="mt-4"
			disabled={!isNewLogFormValid || !selectedGame}>Save</Button
		>
		<Button variant="destructive" on:click={() => window.history.back()}>Cancel</Button>
	</div>
</main>
