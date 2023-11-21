<script lang="ts">
	import { AuthenticateWithTwitch, SearchForGame } from '$lib/wailsjs/go/main/App';
	import GameCard from './GameCard.svelte';

	export let open = false;
	let gameSelectModal: HTMLDialogElement | undefined;
	let gameName = '';
	let searchPromise: ReturnType<typeof SearchForGame> | undefined;

	$: if (open) {
		gameSelectModal?.showModal();
	} else {
		gameSelectModal?.close();
	}

	async function handleSearch() {
		if (gameName === '') {
			return;
		}

		const accessTokenResponse = await AuthenticateWithTwitch();
		searchPromise = SearchForGame(gameName, accessTokenResponse.access_token);
	}
</script>

<dialog class="modal" bind:this={gameSelectModal} on:close>
	<div class="modal-box">
		<p class="text-2xl font-semibold mb-4">Choose a Game</p>
		<div class="flex flex-col gap-4">
			<div class="join mx-auto">
				<input
					class="input input-bordered join-item"
					type="text"
					placeholder="The latest game"
					bind:value={gameName}
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
								<GameCard data={game} />
							{/each}
						{/if}
					{:catch error}
						<p>Something went wrong</p>
						<p>{error}</p>
					{/await}
				{/if}
			</div>
		</div>
	</div>
</dialog>
