<script lang="ts">
	import GameSearchModal from '$lib/components/GameSearchModal.svelte';
	import LogModal from '$lib/components/LogModal.svelte';
	import LogsGrid from '$lib/components/LogsGrid.svelte';
	import type { main } from '$lib/wailsjs/go/models';
	import { Settings } from 'lucide-svelte';
	import { EventsOn } from '$lib/wailsjs/runtime/runtime';
	import toast from 'svelte-french-toast';
	import { AuthenticateWithTwitch, SearchForGame } from '$lib/wailsjs/go/main/App';

	let openGameSearchModal = false;
	let selectedGame: main.IgdbGame | undefined;
    let executableData: { title: string, minutesPlayed: number } = { title: "", minutesPlayed: 0 };
	$: openLogModal = !!selectedGame;
	$: if (openLogModal && openGameSearchModal) {
		openGameSearchModal = false;
	}
    EventsOn("game-stopped", async (data) => {
        if (data.isNewGame) {
            selectedGame = undefined;
            openGameSearchModal = true;
            executableData = data;
            toast("Looks like you're playing a new title, help us out by telling us what it is!");
        } else {
            const authenticateRes = await AuthenticateWithTwitch();
            if (!authenticateRes.access_token) {
                console.error("Failed to authenticate with Twitch");
            }
            const queriedGames = await SearchForGame(data.title, authenticateRes.access_token);
            if (queriedGames.length === 0) {
                console.error("Failed to search for game");
            }
            executableData = { title: data.title, minutesPlayed: data.minutesPlayed };
            selectedGame = queriedGames[0];
        }
    })
</script>

<main class="flex flex-col justify-center items-center h-full p-12">
    <a href="/settings" class='btn mb-2'><Settings /></a>
	<GameSearchModal
		open={openGameSearchModal}
		bind:selectedGame
		on:open={() => (openGameSearchModal = true)}
		on:close={() => (openGameSearchModal = false)}
	/>
	{#if selectedGame}
		<LogModal
			game={selectedGame}
			open={openLogModal}
            {executableData}
			on:open={() => (openLogModal = true)}
			on:close={() => (openLogModal = false)}
			on:back={() => {
				selectedGame = undefined;
				openGameSearchModal = true;
			}}
		/>
	{/if}
	<button class="btn" on:click={() => (openGameSearchModal = !openGameSearchModal)}
		>Create Log</button
	>
    <LogsGrid />
</main>
