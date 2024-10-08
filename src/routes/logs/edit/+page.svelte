<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { logSchema, statusOptions } from '$lib/schemas';
	import { toast } from 'svelte-sonner';
	import { defaults, superForm } from 'sveltekit-superforms';
	import { zod, zodClient } from 'sveltekit-superforms/adapters';
	import * as Form from '$lib/components/ui/form';
	import Combobox from '$lib/components/Combobox.svelte';
	import { Input } from '$lib/components/ui/input';
	import Textarea from '$lib/components/ui/textarea/textarea.svelte';
	import { RangeCalendar } from '$lib/components/ui/range-calendar/index.js';
	import { getLocalTimeZone, today } from '@internationalized/date';
	import * as RadioGroup from '$lib/components/ui/radio-group';
	import { Button } from '$lib/components/ui/button';
	import { logDataFromForm } from '$lib';
	import { useMutation, useQueryClient } from '@sveltestack/svelte-query';
	import { addExecutableDetails, addLog, updateLog } from '$lib/rust-bindings/database';
	import type { PageData } from './$types';
	import { toTitleCase } from '$lib';
	import { onMount } from 'svelte';
	import * as Popover from '$lib/components/ui/popover';
	import { CalendarIcon } from 'lucide-svelte';
	import { cn } from '$lib/utils';

	export let data: PageData;
	const searchParams = $page.url.searchParams;
	const isEditing = searchParams.has('id');
	const insertLogMutation = useMutation(addLog, {
		onSuccess: async () => {
			queryClient.invalidateQueries('logs');
			const executableName = searchParams.get('executableName');
			const minutesPlayed = searchParams.get('minutesPlayed');
			if (executableName && minutesPlayed) {
				await addExecutableDetails({
					name: executableName,
					game_id: data.igdbGame.id,
					minutes_played: parseInt(minutesPlayed)
				});
			}
		}
	});
	const updateLogMutation = useMutation(updateLog, {
		onSuccess: () => queryClient.invalidateQueries('logs')
	});
	const queryClient = useQueryClient();
	const logForm = superForm(defaults(zod(logSchema)), {
		validators: zodClient(logSchema),
		SPA: true,
		onUpdate: async ({ form }) => {
			if (form.valid) {
				const candidateLog = logDataFromForm(data.igdbGame, form.data);
				if (!isEditing) {
					toast.promise($insertLogMutation.mutateAsync(candidateLog), {
						loading: 'Creating log...',
						success: 'Log created successfully!',
						error: 'Failed to create log'
					});
					goto('/logs');
				} else {
					toast.promise(
						$updateLogMutation.mutateAsync({
							id: parseInt(searchParams.get('id') as string),
							...candidateLog
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

	let isNewLogFormValid = false;
	$: if ($logFormData)
		validateLogForm({ update: false }).then((superValidated) => {
			isNewLogFormValid = superValidated.valid;
		});
	onMount(() => {
		$logFormData.logStartDate = data.form.data.logStartDate;
		$logFormData.logEndDate = data.form.data.logEndDate;
	});
	const timeZone = getLocalTimeZone();
	let dateRange = {
		start: today(timeZone),
		end: today(timeZone)
	};
	$: if (dateRange.start && dateRange.end) {
		$logFormData.logStartDate = dateRange.start.toDate(timeZone);
		$logFormData.logEndDate = dateRange.end.toDate(timeZone);
	}
</script>

<main class="min-h-full container py-8 px-16">
	<div class="mb-4">
		<h1 class="text-3xl font-heading font-bold">{isEditing ? 'Edit' : 'New'} Log</h1>
		<p class="text-gray-500 text-lg font-heading">
			What was it like playing {data.igdbGame.title}?
		</p>
	</div>
	<form method="post" class="grid-cols-[25%,_1fr] grid gap-4" id="logForm" use:logEnhance>
		<div>
			<img
				src={'https://images.igdb.com/igdb/image/upload/t_cover_big/' +
					data.igdbGame.cover_image_id +
					'.jpg'}
				alt="cover"
				class="aspect-[3/4] rounded-3xl mb-4 w-full"
			/>
			<Form.Field form={logForm} name="status">
				<Form.Control let:attrs>
					<Combobox
						{...attrs}
						options={statusOptions.map((status) => ({ value: status, label: toTitleCase(status) }))}
						placeholder="Pick a status"
						emptyText="No status found!"
						bind:value={$logFormData.status}
					/>
				</Form.Control>
			</Form.Field>
		</div>
		<div class="flex flex-col gap-2">
			<div>
				<p class="text-2xl font-heading font-semibold">{data.igdbGame.title}</p>
				<Form.Fieldset form={logForm} name="rating">
					<RadioGroup.Root
						value={`${$logFormData.rating}`}
						class="flex items-center"
						onValueChange={(newValue) =>
							validateLogFormField('rating', { value: parseInt(newValue) })}
					>
						{#each Array(5) as _, i}
							<Form.Control let:attrs>
								<RadioGroup.Item class="hidden" value={`${i + 1}`} {...attrs} />
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
			<div class="flex gap-2">
				<Form.Field form={logForm} name="logStartDate">
					<Form.Control>
						<Form.Label class="block mt-2">Day(s) played</Form.Label>
						<Popover.Root openFocus>
							<Popover.Trigger asChild let:builder>
								<Button
									variant="outline"
									class={cn(
										'w-[300px] justify-start text-left font-normal',
										!dateRange && 'text-muted-foreground'
									)}
									builders={[builder]}
								>
									<CalendarIcon class="mr-2 h-4 w-4" />
									{#if dateRange.start && dateRange.end}
										{dateRange.start
											.toDate(timeZone)
											.toLocaleDateString('en-US', { month: 'long', day: 'numeric' })} -
										{dateRange.end
											.toDate(timeZone)
											.toLocaleDateString('en-US', { month: 'long', day: 'numeric' })}
									{:else}
										Pick a date
									{/if}
								</Button>
							</Popover.Trigger>
							<Popover.Content class="w-auto p-0" align="start">
								<RangeCalendar
									onValueChange={(value) => {
										validateLogFormField('logStartDate', { value: value.start?.toDate(timeZone) });
										validateLogFormField('logEndDate', { value: value.end?.toDate(timeZone) });
									}}
									bind:value={dateRange}
								/>
							</Popover.Content>
						</Popover.Root>
					</Form.Control>
					<Form.FieldErrors />
					<Form.Field form={logForm} name="logEndDate">
						<Form.FieldErrors />
					</Form.Field>
				</Form.Field>
			</div>
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
							placeholder="I particularly struggled on the part where..."
							bind:value={$logFormData.notes}
						/>
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>
			</div>
		</div>
	</form>
	<div class="float-right">
		<Button variant="secondary" on:click={() => window.history.back()}>Cancel</Button>
		<Button
			type="submit"
			form="logForm"
			data-testid="save-log"
			class="mt-4"
			disabled={!isNewLogFormValid}>Save</Button
		>
	</div>
</main>
