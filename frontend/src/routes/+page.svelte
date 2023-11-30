<script lang="ts">
	import GameSearchModal from '$lib/components/GameSearchModal.svelte';
	import type { main } from '$lib/wailsjs/go/models';

	let openGameSearchModal = false;
	let selectedGame: main.IgdbGame | undefined;
    console.log(selectedGame);
</script>

<main class="flex justify-center items-center h-full">
	<GameSearchModal
		open={openGameSearchModal}
		bind:selectedGame
		on:open={() => (openGameSearchModal = true)}
		on:close={() => (openGameSearchModal = false)}
	/>
	<button class="btn" on:click={() => (openGameSearchModal = !openGameSearchModal)}
		>Create Log</button
	>
</main>
