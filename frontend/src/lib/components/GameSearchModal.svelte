<script lang="ts">
	import { AuthenticateWithTwitch, SearchForGame } from '$lib/wailsjs/go/main/App';
	import type { main } from '$lib/wailsjs/go/models';
	import Modal from './Modal.svelte';
	import { createForm } from 'felte';
	import { z } from 'zod';
	import { validator } from '@felte/validator-zod';
	import GameCarousel from './GameCarousel.svelte';

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
	<svelte:fragment slot="heading">Search for a Game</svelte:fragment>
	<svelte:fragment slot="content">
		<div class="flex flex-col gap-4">
			<form method="post" class="flex justify-center" use:form>
				<div class="join">
					<input
						name="game"
						class="input input-bordered join-item"
						type="text"
						placeholder="The latest game"
					/>
					<button class="btn join-item rounded-r-full input-bordered">Search</button>
				</div>
			</form>
			<div class="flex justify-center gap-4">
				{#if searchPromise}
					{#await searchPromise}
						<span class="loading loading-spinner loading-lg mx-auto"></span>
					{:then searchResult}
						{#if searchResult.length === 0}
							<p>No results found</p>
						{:else}
							<GameCarousel
								games={searchResult}
								on:gameClick={({ detail: game }) => (selectedGame = game)}
							/>
						{/if}
					{:catch error}
						<p>Something went wrong</p>
						<p>{error}</p>
					{/await}
				{/if}
			</div>
		</div>
	</svelte:fragment>
</Modal>
