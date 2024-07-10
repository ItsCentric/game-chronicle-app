<script lang="ts">
	import Input from '$lib/components/ui/input/input.svelte';
	import { getSteamData, importIgdbGames } from '$lib/rust-bindings/data_import';
	import { listen } from '@tauri-apps/api/event';
	import stepTwoGif from '$lib/assets/steam-import-step-2.gif';
	import type { PageData } from './$types';
	import { superForm } from 'sveltekit-superforms';
	import * as Form from '$lib/components/ui/form';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import { steamImportFormSchema } from '$lib/schemas';
	import { ArrowLeft, Check, LoaderCircle, X } from 'lucide-svelte';
	import { Button } from '$lib/components/ui/button';

	type RetrieveStarted = {
		status: 'started';
		total_games: number;
	};

	type RetrieveProgress = {
		status: 'progress';
		games_retrieved: number;
	};

	type RetrieveEvent = RetrieveStarted | RetrieveProgress;

	type ImportEvent = {
		games_imported: number;
	};

	export let data: PageData;

	let totalGamesToRetrieve: number | undefined;
	let gamesRetrieved = 0;
	let retrievalFinished = false;
	let gamesImported = 0;
	let importFinished = false;
	let importPromise: Promise<void> | undefined;
	const importForm = superForm(data.form, {
		SPA: true,
		validators: zodClient(steamImportFormSchema),
		onUpdate: async ({ form }) => {
			if (!form.valid) return;
			importPromise = importSteamGameData(form.data.steamId, form.data.steamKey);
		}
	});
	const { form: importFormData, enhance } = importForm;

	async function importSteamGameData(steamId: string, steamKey: string) {
		const retrievalUnlisten = await listen('retrieval', (event) => {
			const payload = event.payload as RetrieveEvent;
			if (payload.status === 'started') {
				totalGamesToRetrieve = payload.total_games;
			} else {
				gamesRetrieved += payload.games_retrieved;
			}
		});
		const steamGames = await getSteamData(steamId, steamKey);
		retrievalUnlisten();
		gamesRetrieved = steamGames.length;
		retrievalFinished = true;
		const importUnlisten = await listen('import', (event) => {
			const payload = event.payload as ImportEvent;
			gamesImported += payload.games_imported;
		});
		gamesImported = await importIgdbGames(steamGames);
		importUnlisten();
		importFinished = true;
	}
</script>

<main class="py-16">
	<div class="max-w-prose mx-auto">
		<div class="relative mb-8">
			<Button
				href="/onboarding"
				class="absolute top-2 -translate-x-full -left-4"
				variant="ghost"
				size="icon"><ArrowLeft size={48} /></Button
			>
			<h1 class="text-3xl font-heading font-bold">Import your Steam data</h1>
			<p>
				Import your Steam data to get a head start on your game library. We'll retrieve all the
				games you've played and import them into your library.
			</p>
		</div>
		<form method="post" use:enhance class="relative flex flex-col gap-16">
			<div class="absolute -translate-x-full -left-6">
				<div
					class="text-3xl flex justify-center items-center font-bold aspect-square h-auto w-16 text-center rounded-full border border-muted"
				>
					<p>1</p>
				</div>
				<div class="mx-auto h-40 bg-muted w-px"></div>
				<div
					class="text-3xl flex justify-center items-center font-bold aspect-square h-auto w-16 text-center rounded-full border border-muted"
				>
					<p>2</p>
				</div>
				<div class="mx-auto h-[560px] bg-muted w-px"></div>
				<div
					class="text-3xl flex justify-center items-center font-bold aspect-square h-auto w-16 text-center rounded-full border border-muted"
				>
					<p>3</p>
				</div>
			</div>
			<div>
				<h2 class="text-2xl font-heading font-bold">Provide your Steam API key</h2>
				<p class="mb-4">
					Please <a
						href="https://steamcommunity.com/dev/apikey"
						target="_blank"
						rel="noopener noreferrer"
						class="text-accent">register for a Steam API key</a
					> and paste it here. You can enter any domain name (such as localhost:3000) for the website
					field.
				</p>
				<Form.Field name="steamKey" form={importForm}>
					<Form.Control let:attrs>
						<Input {...attrs} bind:value={$importFormData.steamKey} placeholder="Steam API Key" />
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>
			</div>
			<div>
				<h2 class="text-2xl font-heading font-bold">Provide your Steam ID</h2>
				<p class="mb-4">
					Go to your Steam profile and copy your Steam ID. You can find it in the URL of your
					profile page. If you have a custom URL, you will have to go into your profile settings and
					clear your custom URL to see your Steam ID.
				</p>
				<img src={stepTwoGif} alt="How to find your Steam ID" class="mb-4" />
				<Form.Field name="steamId" form={importForm}>
					<Form.Control let:attrs>
						<Input {...attrs} bind:value={$importFormData.steamId} placeholder="Steam ID" />
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>
			</div>
			<div>
				<h2 class="text-2xl font-heading font-bold">Import your data</h2>
				<p class="mb-2">All set! Click that button below and we'll start importing your data.</p>
				<div class="flex items-center gap-2">
					<Form.Button disabled={!!importPromise}>
						{#if importPromise}
							{#await importPromise}
								<LoaderCircle class="animate-spin mr-2" />
								<p>Importing data...</p>
							{:then}
								<Check class="mr-2" />
								<p>Data imported!</p>
							{:catch}
								<X class="mr-2" />
								<p>Failed import</p>
							{/await}
						{:else}
							<p>Import data</p>
						{/if}
					</Form.Button>
					{#if importPromise}
						{#if !retrievalFinished}
							{#if totalGamesToRetrieve}
								<p>Retrieved {gamesRetrieved} of {totalGamesToRetrieve} games</p>
							{/if}
						{:else if retrievalFinished && !importFinished}
							<p>Imported {gamesImported} games</p>
						{:else if retrievalFinished && importFinished}
							<p>Imported {gamesImported} games</p>
						{:else}
							<p>Something went wrong</p>
						{/if}
					{/if}
				</div>
			</div>
		</form>
		<Button href="/onboarding/game-detection" class="float-right">Next</Button>
	</div>
</main>
