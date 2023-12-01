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

	export let open = false;
	export let game: main.IgdbGame;
	const formSchema = z.object({
		rating: z.number().min(1).max(10),
		status: z.enum(['Playing', 'Completed', 'Abandoned'], {
			required_error: 'Please select a state',
			invalid_type_error: 'Invalid state'
		}),
		notes: z.string().min(1).max(1000),
		timePlayedHours: z.number().min(0),
		timePlayedMinutes: z.number().min(0)
	});
	const { form } = createForm<z.infer<typeof formSchema>>({
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
			candidateLog.startedOn = startedOn;
			candidateLog.finishedOn = finishedOn;
			candidateLog.timePlayed = candidateTimePlayed;
			const res = await InsertGameLog(candidateLog);

			if (res.errors.server) {
				toast.error(res.errors.server);
			} else {
				open = false;
				toast.success('Log created!');
			}
		}
	});
	let startedOn = new Date();
	let finishedOn = new Date();
	const dateInputProps = {
		format: 'MM/dd/yyyy',
		required: true,
		dynamicPositioning: true
	};
	const statusSelectOptions = [
		{ value: 'Playing', label: 'Playing' },
		{ value: 'Completed', label: 'Completed' },
		{ value: 'Abandoned', label: 'Abandoned' }
	];
</script>

<Modal {open} on:close>
	<p class="text-2xl font-semibold mb-4">Create a Log</p>
	<form id="log" method="post" class="grid-cols-[25%,_1fr] grid gap-4" use:form>
		<div>
			<img
				src={'https://images.igdb.com/igdb/image/upload/t_cover_big/' +
					game.cover.image_id +
					'.jpg'}
				alt="cover"
				class="aspect-[3/4] rounded-3xl mb-4"
			/>
			<Select id="status" name="status" placeholder="Game status" items={statusSelectOptions} />
			<ValidationMessage for="status" let:messages={message}>
				<span class="text-sm text-red-500">{message || ''}</span>
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
					bind:value={startedOn}
					max={finishedOn}
					placeholder="I started on..."
					{...dateInputProps}
				/>
			</div>
			<div>
				<label for="finishedOn">Finished on</label>
				<DateInput
					id="finishedOn"
					bind:value={finishedOn}
					min={startedOn}
					placeholder="I finished on..."
					{...dateInputProps}
					required={false}
				/>
			</div>
			<div>
				<div>
					<label for="timePlayedHours">Time Played</label>
				</div>
				<div class="inline-block w-24">
					<input
						type="number"
						id="timePlayedHours"
						name="timePlayedHours"
						class="input input-bordered w-full"
						placeholder="Hours"
					/>
					<ValidationMessage for="timePlayedHours" let:messages={message}>
						<span class="text-sm text-red-500">{message || ''}</span>
					</ValidationMessage>
				</div>
				<div class="inline-block w-24">
					<input
						type="number"
						id="timePlayedMinutes"
						name="timePlayedMinutes"
						class="input input-bordered w-full"
						placeholder="Minutes"
					/>
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
			</div>
		</div>
	</form>
	<div class="modal-action">
		<button type="submit" form="log" class="btn btn-primary">Save</button>
		<button type="button" class="btn" on:click={() => (open = false)}>Cancel</button>
	</div>
</Modal>
