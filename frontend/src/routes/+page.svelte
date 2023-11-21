<script lang="ts">
	import GameSelectModal from '$lib/components/GameSelectModal.svelte';

	let openGameSelectModal = false;
</script>

<main class="flex justify-center items-center h-full">
	<GameSelectModal
		open={openGameSelectModal}
		on:close={() => (openGameSelectModal = !openGameSelectModal)}
	/>
	<button class="btn" on:click={() => (openGameSelectModal = !openGameSelectModal)}
		>Create Log</button
	>
</main>
