<script lang="ts">
	import Modal from './Modal.svelte';
	import RatingInput from './RatingInput.svelte';
	import { DateInput } from 'date-picker-svelte';
	import Select from 'svelte-select';
	import { main } from '$lib/wailsjs/go/models';
	import { InsertGameLog } from '$lib/wailsjs/go/main/Database';
	import { createForm } from 'felte';
	import { validator } from '@felte/validator-zod';
	import { reporter, ValidationMessage } from '@felte/reporter-svelte';
	import { z } from 'zod';
	import toast from 'svelte-french-toast';
	import { ArrowLeft } from 'lucide-svelte';
	import { createEventDispatcher } from 'svelte';

	export let open = false;
	export let game: main.IgdbGame;
	const formSchema = z.object({
		rating: z
			.number()
			.max(5, { message: 'Rating must be 5 or less' })
			.nonnegative({ message: 'Rating must be positive' }),
		status: z.enum(['Playing', 'Completed', 'Abandoned']),
		notes: z.string().max(1000, { message: 'Notes must be less than 1000 characters' }).optional(),
		startedOn: z.date().max(new Date(), { message: 'Started on cannot be in the future' }),
		finishedOn: z.date().max(new Date(new Date().setDate(new Date().getDate() + 1)), {
			message: 'Finished on cannot be in the future'
		}),
		timePlayedHours: z.number({ invalid_type_error: 'Invalid value for hour' }).min(0),
		timePlayedMinutes: z.number({ invalid_type_error: 'Invalid value for minute' }).min(0)
	});
	const { form, isValid, data, reset } = createForm<z.infer<typeof formSchema>>({
		extend: [validator({ schema: formSchema }), reporter],
		onSubmit: async (data) => {
			let candidateLog = new main.LogData();
			const candidateTimePlayed = new main.TimePlayed();
			candidateTimePlayed.hours = data.timePlayedHours;
			candidateTimePlayed.minutes = data.timePlayedMinutes;
			candidateLog.title = game.name;
			candidateLog.rating = data.rating;
			candidateLog.status = data.status;
			candidateLog.notes = data.notes;
			candidateLog.startedOn = data.startedOn;
			candidateLog.finishedOn = data.finishedOn;
			candidateLog.timePlayed = candidateTimePlayed;
			const res = await InsertGameLog(candidateLog);

			if (res.errors?.server) {
				toast.error(res.errors.server);
			} else {
				open = false;
				toast.success('Log created!');
			}
		},
		transform: (data) => {
			const formData = data as z.infer<typeof formSchema>;
			return {
				...formData,
				rating: parseFloat(formData.rating as unknown as string)
			};
		}
	});
	const dateInputProps = {
		format: 'MM/dd/yyyy',
		required: true,
		dynamicPositioning: true
	};
	const statusSelectOptions = [
		'Wishlist',
		'Backlog',
		'Playing',
		'Completed',
		'Played',
		'Shelved',
		'Retired',
		'Abandoned'
	];
	const backDispatcher = createEventDispatcher();
</script>

<Modal {open} on:back on:close on:close={() => reset()}>
	<svelte:fragment slot="heading">
				<button
					on:click={() => backDispatcher('back', true)}
				>
					<ArrowLeft size={32} />
				</button>
		<p>Create a Log</p>
	</svelte:fragment>
	<svelte:fragment slot="content">
		<form id="log" method="post" class="grid-cols-[25%,_1fr] grid gap-4" use:form>
			<div>
				<img
					src={'https://images.igdb.com/igdb/image/upload/t_cover_big/' +
						game.cover.image_id +
						'.jpg'}
					alt="cover"
					class="aspect-[3/4] rounded-3xl mb-4"
				/>
				<Select
					id="status"
					name="status"
					placeholder="Game status"
					bind:justValue={$data.status}
					items={statusSelectOptions}
				/>
				<ValidationMessage for="status" let:messages={message}>
					<span class="text-sm text-red-500">{message ? 'Please select a status' : ''}</span>
				</ValidationMessage>
			</div>
			<div class="grid grid-cols-2 gap-2">
				<div class="col-span-2">
					<p class="text-3xl font-semibold">{game.name}</p>
					<RatingInput />
				</div>
				<div>
					<label for="startedOn">Started on</label>
					<DateInput
						id="startedOn"
						bind:value={$data.startedOn}
						max={new Date()}
						placeholder="I started on..."
						{...dateInputProps}
					/>
					<ValidationMessage for="startedOn" let:messages={message}>
						<span class="text-sm text-red-500">{message || ''}</span>
					</ValidationMessage>
				</div>
				<div>
					<label for="finishedOn">Finished on</label>
					<DateInput
						id="finishedOn"
						bind:value={$data.finishedOn}
						min={$data.startedOn}
						max={new Date()}
						placeholder="I finished on..."
						{...dateInputProps}
						required={false}
					/>
					<ValidationMessage for="finishedOn" let:messages={message}>
						<span class="text-sm text-red-500">{message || ''}</span>
					</ValidationMessage>
				</div>
				<div>
					<div>
						<label for="timePlayedHours">Time Played</label>
					</div>
					<div class="inline-block w-16">
						<input
							type="number"
							id="timePlayedHours"
							name="timePlayedHours"
							class="input input-bordered w-full"
							placeholder="HH"
							min="0"
						/>
						<ValidationMessage for="timePlayedHours" let:messages={message}>
							<span class="text-sm text-red-500">{message || ''}</span>
						</ValidationMessage>
					</div>
					<div class="inline-block w-16">
						<input
							type="number"
							id="timePlayedMinutes"
							name="timePlayedMinutes"
							class="input input-bordered w-full"
							placeholder="MM"
							max="59"
							min="0"
						/>
						<ValidationMessage for="timePlayedMinutes" let:messages={message}>
							<span class="text-sm text-red-500">{message || ''}</span>
						</ValidationMessage>
					</div>
				</div>
				<div class="col-span-2">
					<label for="notes">Notes</label>
					<textarea
						id="notes"
						name="notes"
						class="textarea textarea-bordered w-full"
						placeholder="Notes"
					></textarea>
					<ValidationMessage for="notes" let:messages={message}>
						<span class="text-sm text-red-500">{message || ''}</span>
					</ValidationMessage>
				</div>
			</div>
		</form>
		<div class="modal-action">
			<button type="submit" form="log" class="btn btn-primary" disabled={!$isValid}>Save</button>
			<button type="button" class="btn" on:click={() => (open = false)}>Cancel</button>
		</div>
	</svelte:fragment>
</Modal>
