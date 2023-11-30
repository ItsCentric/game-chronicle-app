<script lang="ts">
	import { AuthenticateWithTwitch, SearchForGame } from '$lib/wailsjs/go/main/App';
	import GameCard from './GameCard.svelte';
	import { slide } from 'svelte/transition';
	import type { main } from '$lib/wailsjs/go/models';
	import Modal from './Modal.svelte';

	export let open = false;
	export let selectedGame: main.IgdbGame | undefined;

	let queriedGame = '';
	let searchPromise: ReturnType<typeof SearchForGame> | undefined;

	async function handleSearch() {
		if (queriedGame === '') {
			return;
		}

		const accessTokenResponse = await AuthenticateWithTwitch();
		searchPromise = SearchForGame(queriedGame, accessTokenResponse.access_token);
	}
</script>

<Modal {open} on:open on:close>
	<p class="text-2xl font-semibold mb-4">Search for a Game</p>
	<div class="flex flex-col gap-4" transition:slide={{ axis: 'x' }}>
		<div class="join mx-auto">
			<input
				class="input input-bordered join-item"
				type="text"
				placeholder="The latest game"
				bind:value={queriedGame}
			/>
			<button class="btn join-item rounded-r-full" on:click={handleSearch}>Search</button>
		</div>
		<div class="flex gap-4">
			{#if searchPromise}
				{#await searchPromise}
					<p>Searching...</p>
				{:then searchResult}
					{#if searchResult.length === 0}
						<p>No results found</p>
					{:else}
						{#each searchResult as game}
							<GameCard
								data={game}
								on:click={() => {
									selectedGame = game;
								}}
							/>
						{/each}
					{/if}
				{:catch error}
					<p>Something went wrong</p>
					<p>{error}</p>
				{/await}
			{/if}
		</div>
	</div>
</Modal>
