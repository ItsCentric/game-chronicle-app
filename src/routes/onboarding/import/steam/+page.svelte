<script lang="ts">
	import { getSteamData, importIgdbGames } from '$lib/rust-bindings/data_import';
	import { listen } from '@tauri-apps/api/event';

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

	let totalGamesToRetrieve: number | undefined;
	let gamesRetrieved = 0;
	let retrievalFinished = false;
	let gamesImported = 0;
	let importFinished = false;

	async function importSteamGameData() {
		const retrievalUnlisten = await listen('retrieval', (event) => {
			const payload = event.payload as RetrieveEvent;
			if (payload.status === 'started') {
				totalGamesToRetrieve = payload.total_games;
			} else {
				gamesRetrieved += payload.games_retrieved;
			}
		});
		const steamGames = await getSteamData('76561199051741234');
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

	importSteamGameData();
</script>

<main>
	{#if !retrievalFinished}
		<p>Retrieving data...</p>
		{#if totalGamesToRetrieve}
			<p>Retrieved {gamesRetrieved} of {totalGamesToRetrieve} games</p>
		{/if}
	{:else if retrievalFinished && !importFinished}
		<p>Importing data...</p>
		<p>Imported {gamesImported} games</p>
	{:else if retrievalFinished && importFinished}
		<p>Finished importing data</p>
		<p>Imported {gamesImported} games</p>
		<a href="/logs">View logs</a>
	{:else}
		<p>Something went wrong</p>
	{/if}
</main>
