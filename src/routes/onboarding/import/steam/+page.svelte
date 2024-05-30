<script>
	import { getSteamData, importIgdbGames } from '$lib/rust-bindings/data_import';

	async function importSteamGameData() {
		const steamGames = await getSteamData('76561199051741234');
		importIgdbGames(steamGames);
	}

	let importPromise = importSteamGameData();
</script>

<main>
	{#await importPromise}
		<p>Importing data...</p>
	{:then}
		<p>Imported data successfully!</p>
		<a href="/logs">View logs</a>
	{:catch error}
		<p>Failed to import data: {error}</p>
	{/await}
</main>
