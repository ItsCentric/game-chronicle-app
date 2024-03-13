<script lang="ts">
	import { main } from '$lib/wailsjs/go/models';
	import { ArrowDownUp, Filter, Search, Settings, X } from 'lucide-svelte';
	import { EventsOn } from '$lib/wailsjs/runtime/runtime';
    import { toast } from "svelte-sonner";
	import { AuthenticateWithTwitch, SearchForGame } from '$lib/wailsjs/go/main/App';
	import * as Dialog from '$lib/components/ui/dialog';
	import { superForm, defaults, type Infer } from 'sveltekit-superforms';
	import {
		filterFormSchema,
		gameSearchSchema,
		newLogSchema,
		sortFormSchema,
		statusOptions,
		type SortFormSchema,
		type FilterFormSchema
	} from '$lib/schemas';
	import { zod, zodClient } from 'sveltekit-superforms/adapters';
	import { Input } from '$lib/components/ui/input';
	import * as Form from '$lib/components/ui/form';
	import * as Carousel from '$lib/components/ui/carousel/index.js';
	import GameCard from '$lib/components/GameCard.svelte';
	import Combobox from '$lib/components/Combobox.svelte';
	import Textarea from '$lib/components/ui/textarea/textarea.svelte';
	import DatePicker from '$lib/components/DatePicker.svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button';
	import { useMutation, useQuery, useQueryClient } from '@sveltestack/svelte-query';
	import {
		GetGameLogs,
		InsertExecutableDetails,
		InsertGameLog
	} from '$lib/wailsjs/go/main/Database';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import Badge from '$lib/components/ui/badge/badge.svelte';
	import * as Card from '$lib/components/ui/card';
	import * as Select from '$lib/components/ui/select';
	import { getLocalTimeZone, parseDate, today } from '@internationalized/date';
	import * as RadioGroup from '$lib/components/ui/radio-group';

	let selectedGame: main.IgdbGame | undefined;
	let searchPromise: ReturnType<typeof SearchForGame> | undefined;
	let executableData: { title: string; minutesPlayed: number } = { title: '', minutesPlayed: 0 };
	const newLogMutation = useMutation(InsertGameLog);
	const queryClient = useQueryClient();
	let openGameSearchModal = false;
	async function authenticateAndSearchForGame(gameTitle: string) {
		const authenticateRes = await AuthenticateWithTwitch();
		if (!authenticateRes.access_token) {
			console.error('Failed to authenticate with Twitch');
		}
		const queriedGames = await SearchForGame(gameTitle, authenticateRes.access_token);
		if (queriedGames.length === 0) {
			console.error('Failed to search for game');
		}
		return queriedGames;
	}
	function applySortAndFilter(data: Infer<SortFormSchema> | Infer<FilterFormSchema>) {
		currentSortAndFilter = { ...currentSortAndFilter, ...data };
		queryClient.invalidateQueries('logs');
	}

	const gameSearchForm = superForm(defaults(zod(gameSearchSchema)), {
		validators: zodClient(gameSearchSchema),
		SPA: true,
		onUpdate: ({ form }) => {
			if (form.valid) {
				searchPromise = authenticateAndSearchForGame(form.data.gameTitle);
			}
		}
	});
	const {
		form: gameSearchFormData,
		enhance: gameSearchEnhance,
		reset: gameSearchFormReset
	} = gameSearchForm;
	const newLogForm = superForm(defaults(zod(newLogSchema)), {
		validators: zodClient(newLogSchema),
		SPA: true,
		onUpdate: async ({ form }) => {
			if (form.valid && selectedGame) {
				let candidateLog = new main.LogData();
				const candidateTimePlayed = new main.TimePlayed();
				candidateTimePlayed.hours = form.data.timePlayedHours;
				candidateTimePlayed.minutes = form.data.timePlayedMinutes;
				candidateLog.title = selectedGame.name;
				candidateLog.rating = form.data.rating;
				candidateLog.status = form.data.status;
				candidateLog.notes = form.data.notes;
				candidateLog.startedOn = form.data.startedOn;
				candidateLog.finishedOn = form.data.finishedOn;
				candidateLog.timePlayed = candidateTimePlayed;
				$newLogMutation.mutate(candidateLog);
				if (executableData.title != '') {
					await InsertExecutableDetails({
						executableName: executableData.title,
						minutesPlayed: executableData.minutesPlayed,
						title: selectedGame.name
					});
				}
			}
		}
	});
	const {
		form: newLogFormData,
		enhance: newLogEnhance,
		reset: newLogFormReset,
		validate: validateNewLogForm
	} = newLogForm;
	const sortForm = superForm(defaults(zod(sortFormSchema)), {
		validators: zodClient(sortFormSchema),
		SPA: true,
		onUpdate: ({ form }) => {
			if (form.valid) {
				applySortAndFilter(form.data);
			}
		},
		resetForm: false
	});
	const { form: sortFormData, enhance: sortFormEnhance } = sortForm;
	const filterForm = superForm(defaults(zod(filterFormSchema)), {
		validators: zodClient(filterFormSchema),
		SPA: true,
		onUpdate: ({ form }) => {
			if (form.valid) {
				applySortAndFilter(form.data);
			}
		},
		resetForm: false
	});
	const { form: filterFormData, enhance: filterFormEnhance } = filterForm;

	let currentSortAndFilter = {
		sortBy: 'created_at',
		sortOrder: 'desc',
		status: [...statusOptions]
	};
	const logsQueryResult = useQuery(
		[
			'logs',
			currentSortAndFilter.sortBy,
			currentSortAndFilter.sortOrder,
			currentSortAndFilter.status
		],
		async () =>
			await GetGameLogs(
				currentSortAndFilter.sortBy,
				currentSortAndFilter.sortOrder,
				currentSortAndFilter.status
			)
	);
	$: openLogModal = !!selectedGame;
	$: if (openLogModal && openGameSearchModal) {
		openGameSearchModal = false;
	}
	$: if ($newLogMutation.isSuccess) {
		openLogModal = false;
		queryClient.invalidateQueries('logs');
		toast.success('Log created!');
	}
	$: if ($newLogMutation.isError) {
		toast.error('Something went wrong!');
	}
	EventsOn('game-stopped', async (data) => {
		if (data.isNewGame) {
			selectedGame = undefined;
			openGameSearchModal = true;
			executableData = data;
			toast("Looks like you're playing a new title, help us out by telling us what it is!");
		} else {
			const gameSearchResults = await authenticateAndSearchForGame(data.title);
			executableData = { title: data.title, minutesPlayed: data.minutesPlayed };
			selectedGame = gameSearchResults[0];
		}
	});
	if (executableData.minutesPlayed) {
		$newLogFormData.timePlayedHours = Math.floor(executableData.minutesPlayed / 60);
		$newLogFormData.timePlayedMinutes = executableData.minutesPlayed % 60;
	}
	$: selectedStatuses = $filterFormData.status.map((status) => ({ value: status, label: status }));
	$: console.log($newLogFormData.rating);
</script>

<main class="flex flex-col justify-center items-center h-full p-12">
	<Button href="/settings" class="mb-2"><Settings /></Button>
	<Dialog.Root
		bind:open={openGameSearchModal}
		onOpenChange={() => {
			searchPromise = undefined;
			gameSearchFormReset();
		}}
	>
		<Dialog.Trigger class={buttonVariants({ variant: 'default' })}>Create Log</Dialog.Trigger>
		<Dialog.Content>
			<Dialog.Header>
				<Dialog.Title>Search for a Game</Dialog.Title>
				<Dialog.Description>Search for a game to create a log for</Dialog.Description>
			</Dialog.Header>
			<div class="flex flex-col gap-4">
				<form method="post" class="flex justify-center" use:gameSearchEnhance>
					<Form.Field form={gameSearchForm} name="gameTitle">
						<Form.Control let:attrs>
							<Input {...attrs} bind:value={$gameSearchFormData.gameTitle} class="rounded-r-none" />
						</Form.Control>
					</Form.Field>
					<Form.Button class="rounded-l-none"><Search size={24} /></Form.Button>
				</form>
				{#if searchPromise}
					{#await searchPromise}
						<span class=""></span>
						<p>Searching...</p>
					{:then searchResult}
						{#if searchResult.length === 0}
							<p>No results found</p>
						{:else}
							<Carousel.Root class="w-5/6 mx-auto">
								<Carousel.Content>
									{#each searchResult as game}
										<Carousel.Item class="basis-1/3">
											<GameCard data={game} on:click={() => (selectedGame = game)} />
										</Carousel.Item>
									{/each}
								</Carousel.Content>
								<Carousel.Previous />
								<Carousel.Next />
							</Carousel.Root>
						{/if}
					{:catch error}
						<p>Something went wrong</p>
						<p>{error}</p>
					{/await}
				{/if}
			</div>
		</Dialog.Content>
	</Dialog.Root>
	{#if selectedGame}
		<Dialog.Root
			bind:open={openLogModal}
			onOpenChange={() => {
				selectedGame = undefined;
				newLogFormReset();
			}}
		>
			<Dialog.Content>
				<Dialog.Header>
					<Dialog.Title>Create Log</Dialog.Title>
					<Dialog.Description>How was today's session with {selectedGame.name}?</Dialog.Description>
				</Dialog.Header>
				<form id="newLog" method="post" class="grid-cols-[33%,_1fr] grid gap-4" use:newLogEnhance>
					<div>
						<img
							src={'https://images.igdb.com/igdb/image/upload/t_cover_big/' +
								selectedGame.cover.image_id +
								'.jpg'}
							alt="cover"
							class="aspect-[3/4] rounded-3xl mb-4"
						/>
						<Form.Field form={newLogForm} name="status">
							<Form.Control let:attrs>
								<Combobox
									options={statusOptions.map((status) => ({ value: status, label: status }))}
									{...attrs}
									placeholder="I am currently..."
									emptyText="No status found!"
									bind:value={$newLogFormData.status}
								/>
							</Form.Control>
						</Form.Field>
					</div>
					<div class="grid grid-cols-2 gap-2">
						<div class="col-span-2">
							<p class="text-2xl font-semibold">{selectedGame.name}</p>
							<Form.Fieldset form={newLogForm} name="rating">
								<RadioGroup.Root
									value={`${$newLogFormData.rating}`}
									class="flex items-center"
									onValueChange={(newValue) =>
										validateNewLogForm('rating', { value: parseInt(newValue) })}
								>
									{#each Array(5) as _, i}
										<Form.Control let:attrs>
											<RadioGroup.Item class="hidden" value={`${i + 1}`} {...attrs} />
											<Form.Label>
												{#if $newLogFormData.rating >= i + 1}
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
							<div>
								<Form.Field form={newLogForm} name="startedOn">
									<Form.Control let:attrs>
										<Form.Label>Started on</Form.Label>
										<DatePicker
											{...attrs}
											maxValue={$newLogFormData.finishedOn
												? parseDate($newLogFormData.finishedOn.toISOString().substring(0, 10))
												: today(getLocalTimeZone())}
											bind:value={$newLogFormData.startedOn}
										/>
									</Form.Control>
									<Form.FieldErrors />
								</Form.Field>
							</div>
							<div>
								<Form.Field form={newLogForm} name="finishedOn">
									<Form.Control let:attrs>
										<Form.Label>Finished on</Form.Label>
										<DatePicker
											{...attrs}
											minValue={$newLogFormData.startedOn
												? parseDate($newLogFormData.startedOn.toISOString().substring(0, 10))
												: undefined}
											maxValue={today(getLocalTimeZone())}
											bind:value={$newLogFormData.finishedOn}
										/>
									</Form.Control>
									<Form.FieldErrors />
								</Form.Field>
							</div>
							<div>
								<Form.Field class="inline-block w-16" form={newLogForm} name="timePlayedHours">
									<Form.Control let:attrs>
										<Form.Label>Time Played</Form.Label>
										<Input
											{...attrs}
											type="number"
											placeholder="HH"
											min="0"
											bind:value={$newLogFormData.timePlayedHours}
											on:change={(newValue) => {
												validateNewLogForm('timePlayedHours', {
													value: parseInt(newValue.currentTarget.value)
												});
											}}
										/>
									</Form.Control>
									<Form.FieldErrors />
								</Form.Field>
								<Form.Field class="inline-block w-16" form={newLogForm} name="timePlayedMinutes">
									<Form.Control let:attrs>
										<Input
											{...attrs}
											type="number"
											placeholder="MM"
											min="0"
											max="59"
											bind:value={$newLogFormData.timePlayedMinutes}
											on:change={(newValue) => {
												validateNewLogForm('timePlayedMinutes', {
													value: parseInt(newValue.currentTarget.value)
												});
											}}
										/>
									</Form.Control>
									<Form.FieldErrors />
								</Form.Field>
							</div>
							<div class="col-span-2">
								<Form.Field form={newLogForm} name="notes">
									<Form.Control let:attrs>
										<Form.Label>Notes</Form.Label>
										<Textarea {...attrs} placeholder="Notes" bind:value={$newLogFormData.notes} />
									</Form.Control>
									<Form.FieldErrors />
								</Form.Field>
							</div>
						</div>
					</div>
				</form>
				<Dialog.Footer>
					<Button type="submit" form="newLog">Save</Button>
					<Button
						type="button"
						on:click={() => {
							selectedGame = undefined;
							newLogFormReset();
						}}>Cancel</Button
					>
				</Dialog.Footer>
			</Dialog.Content>
		</Dialog.Root>
	{/if}
	<div class="flex justify-end gap-2 mb-2">
		<form id="sortForm" method="post" use:sortFormEnhance />
		<DropdownMenu.Root closeOnItemClick={false}>
			<DropdownMenu.Trigger
				class={buttonVariants({ variant: 'default' }) + ' flex gap-2 items-center'}
			>
				<ArrowDownUp size={16} />
				<p>Sort</p>
			</DropdownMenu.Trigger>
			<DropdownMenu.Content>
				<DropdownMenu.Label>Sort by</DropdownMenu.Label>
				<DropdownMenu.Separator />
				<DropdownMenu.RadioGroup bind:value={$sortFormData.sortBy}>
					<DropdownMenu.RadioItem value="title">Alphabetical</DropdownMenu.RadioItem>
					<DropdownMenu.RadioItem value="time_played_minutes">Time Played</DropdownMenu.RadioItem>
					<DropdownMenu.RadioItem value="started_on">Date Started</DropdownMenu.RadioItem>
					<DropdownMenu.RadioItem value="finished_on">Date Finished</DropdownMenu.RadioItem>
					<DropdownMenu.RadioItem value="created_at">Entry Added</DropdownMenu.RadioItem>
				</DropdownMenu.RadioGroup>
				<DropdownMenu.Separator />
				<DropdownMenu.Label>Sort order</DropdownMenu.Label>
				<DropdownMenu.Separator />
				<DropdownMenu.RadioGroup bind:value={$sortFormData.sortOrder}>
					<DropdownMenu.RadioItem value="asc">Ascending</DropdownMenu.RadioItem>
					<DropdownMenu.RadioItem value="desc">Descending</DropdownMenu.RadioItem>
				</DropdownMenu.RadioGroup>
				<div class="mt-2 flex items-center">
					<Button type="submit" form="sortForm" class="rounded-r-none">Apply</Button>
					<Button
						type="button"
						variant="destructive"
						class="rounded-l-none"
						on:click={() => sortForm.reset()}><X /></Button
					>
				</div>
			</DropdownMenu.Content>
		</DropdownMenu.Root>
		<DropdownMenu.Root closeOnItemClick={false}>
			<DropdownMenu.Trigger
				class={buttonVariants({ variant: 'default' }) + ' flex gap-2 items-center'}
			>
				<Filter size={16} />
				<p>Filter</p>
			</DropdownMenu.Trigger>
			<DropdownMenu.Content class="max-w-xs">
				<form id="filterForm" method="post" use:filterFormEnhance />
				<Select.Root
					selected={selectedStatuses}
					onSelectedChange={(status) => {
						if (status) {
							$filterFormData.status = status.map((status) => status.value);
						} else {
							$filterFormData.status = [];
						}
					}}
					multiple
				>
					<Select.Trigger>
						<Select.Value placeholder="Status to filter..." />
					</Select.Trigger>
					<Select.Content>
						{#each statusOptions as status}
							<Select.Item value={status}>{status}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
				<div class="mt-2 flex items-center">
					<Button type="submit" form="filterForm" class="rounded-r-none">Apply</Button>
					<Button
						type="button"
						variant="destructive"
						class="rounded-l-none"
						on:click={() => filterForm.reset()}><X /></Button
					>
				</div>
			</DropdownMenu.Content>
		</DropdownMenu.Root>
	</div>
	<div class="grid grid-cols-3 gap-4">
		{#if $logsQueryResult.data && !$logsQueryResult.isLoading}
			{#each $logsQueryResult.data as log}
				{@const logStatus = log.statusId}
				<Card.Root>
					<Card.Header>
						<Card.Title>{log.title}</Card.Title>
						{#if logStatus === 'Wishlist'}
							<Badge class="w-fit">{log.statusId}</Badge>
						{:else if logStatus === 'Backlog'}
							<Badge class="w-fit bg-gray-500">{log.statusId}</Badge>
						{:else if logStatus === 'Playing'}
							<Badge class="w-fit bg-yellow-500">{log.statusId}</Badge>
						{:else if logStatus === 'Completed'}
							<Badge class="w-fit bg-green-500">{log.statusId}</Badge>
						{:else if logStatus === 'Played'}
							<Badge class="w-fit bg-green-700">{log.statusId}</Badge>
						{:else if logStatus === 'Abandoned'}
							<Badge class="w-fit bg-red-500">{log.statusId}</Badge>
						{:else if logStatus === 'Retired'}
							<Badge class="w-fit bg-red-700">{log.statusId}</Badge>
						{/if}
					</Card.Header>
				</Card.Root>
			{/each}
		{/if}
	</div>
</main>
