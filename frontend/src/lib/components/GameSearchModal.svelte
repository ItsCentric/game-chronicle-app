<script lang="ts">
	import { AuthenticateWithTwitch, SearchForGame } from '$lib/wailsjs/go/main/App';
	import GameCard from './GameCard.svelte';
	import { slide } from 'svelte/transition';
	import type { main } from '$lib/wailsjs/go/models';
	import Modal from './Modal.svelte';
	import { createForm } from 'felte';
	import { z } from 'zod';
	import { validator } from '@felte/validator-zod';

	export let open = false;
	export let selectedGame: main.IgdbGame | undefined;

	const formSchema = z.object({
		game: z.string().min(1)
	});
	const { form, reset } = createForm<z.infer<typeof formSchema>>({
		extend: validator({ schema: formSchema }),
		onSubmit: async (data) => {
			const accessTokenResponse = await AuthenticateWithTwitch();
			searchPromise = SearchForGame(data.game, accessTokenResponse.access_token);
		}
	});
	function resetModal() {
		reset();
		searchPromise = undefined;
	}
	let searchPromise: ReturnType<typeof SearchForGame> | undefined;
</script>

<Modal {open} on:open on:close on:close={resetModal}>
	<p class="text-2xl font-semibold mb-4">Search for a Game</p>
	<div class="flex flex-col gap-4">
		<form method="post" class="flex justify-center" use:form>
			<div class="join">
				<input
					name="game"
					class="input input-bordered join-item"
					type="text"
					placeholder="The latest game"
				/>
				<button class="btn join-item rounded-r-full">Search</button>
			</div>
		</form>
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
