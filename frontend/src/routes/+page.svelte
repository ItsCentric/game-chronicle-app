<script lang="ts">
	import GameSearchModal from '$lib/components/GameSearchModal.svelte';
	import LogModal from '$lib/components/LogModal.svelte';
	import LogsGrid from '$lib/components/LogsGrid.svelte';
	import type { main } from '$lib/wailsjs/go/models';

	let openGameSearchModal = false;
	let selectedGame: main.IgdbGame | undefined;
	$: openLogModal = !!selectedGame;
	$: if (openLogModal && openGameSearchModal) {
		openGameSearchModal = false;
	}
</script>

<main class="flex flex-col justify-center items-center h-full p-12">
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
