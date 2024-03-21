<script lang="ts">
	import GameCard from '$lib/components/GameCard.svelte';
	import * as Form from '$lib/components/ui/form';
	import { Input } from '$lib/components/ui/input';
	import { gameSearchSchema } from '$lib/schemas';
	import { AuthenticateWithTwitch, GetRandomGames, SearchForGame } from '$lib/wailsjs/go/main/App';
	import { ArrowLeft, LoaderCircle, Search, SearchX, X } from 'lucide-svelte';
	import { onMount } from 'svelte';
	import { defaults, superForm } from 'sveltekit-superforms';
	import { zod, zodClient } from 'sveltekit-superforms/adapters';
	import * as Pagination from '$lib/components/ui/pagination';
	import Button from '$lib/components/ui/button/button.svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';

	const gamesPerPage = 18;
	let searchPromise: ReturnType<typeof SearchForGame> | undefined;
	let currentPage = 1;
	$: beginningPageIndex = (currentPage - 1) * gamesPerPage;
	const gameSearchForm = superForm(defaults(zod(gameSearchSchema)), {
		validators: zodClient(gameSearchSchema),
		SPA: true,
		resetForm: false,
		onUpdate: ({ form }) => {
			if (form.valid) {
				searchPromise = authenticateAndSearchForGame(form.data.gameTitle);
			}
		}
	});
	const { form: gameSearchFormData, enhance: gameSearchEnhance } = gameSearchForm;

	onMount(async () => {
		const authenticationResponse = await AuthenticateWithTwitch();
		if (!authenticationResponse.access_token) {
			console.error('Failed to authenticate with Twitch');
		}
		searchPromise = GetRandomGames(gamesPerPage * 4, authenticationResponse.access_token);
	});
	async function authenticateAndSearchForGame(gameTitle: string) {
		const authenticateRes = await AuthenticateWithTwitch();
		if (!authenticateRes.access_token) {
			console.error('Failed to authenticate with Twitch');
		}
		const queriedGames = await SearchForGame(gameTitle, authenticateRes.access_token);
		if (queriedGames.error) {
			console.error('Failed to search for game');
		}
		return queriedGames;
	}
</script>

<main class="min-h-full px-16 py-8">
	<div class="text-center mb-4">
		<h1 class="text-3xl font-bold">Find a Game</h1>
		<p class="text-gray-500">Let's find that game you've been playing</p>
	</div>
	<form method="post" class="flex justify-center mb-8 mx-auto" use:gameSearchEnhance>
		<Button href="/" variant="ghost" class="mr-4"><ArrowLeft size={32} /></Button>
		<Form.Field form={gameSearchForm} name="gameTitle">
			<Form.Control let:attrs>
				<Input {...attrs} bind:value={$gameSearchFormData.gameTitle} class="rounded-r-none" />
			</Form.Control>
		</Form.Field>
		<Form.Button class="rounded-l-none"><Search size={24} /></Form.Button>
	</form>
	<div class="flex w-full justify-center items-center"></div>
	{#if searchPromise}
		{#await searchPromise}
			<LoaderCircle size={64} class="animate-spin mx-auto" />
		{:then searchResult}
			{#if searchResult.games.length === 0}
				<div class="mx-auto text-center">
					<SearchX size={64} class="mb-2 mx-auto" />
					<h2 class="text-2xl font-bold">No games found</h2>
					<p class="text-gray-500">Try searching for something else</p>
				</div>
			{:else}
				{@const executableName = $page.url.searchParams.get('executableName')}
				{@const minutesPlayed = $page.url.searchParams.get('minutesPlayed')}
				{@const isNewGame = !executableName && !minutesPlayed}
				<div class="mx-auto grid grid-cols-6 gap-4 container">
					{#each searchResult.games.slice(beginningPageIndex, beginningPageIndex + gamesPerPage) as game}
						<GameCard
							data={game}
							on:click={() =>
								goto(
									`/log?gameId=${game.id}` +
										(isNewGame
											? `&executableName=${executableName}&minutesPlayed=${minutesPlayed}`
											: '')
								)}
						/>
					{/each}
				</div>
				<Pagination.Root
					count={searchResult.games.length}
					perPage={gamesPerPage}
					let:pages
					bind:page={currentPage}
					class="mt-8"
				>
					<Pagination.Content>
						<Pagination.Item>
							<Pagination.PrevButton />
						</Pagination.Item>
						{#each pages as page (page.key)}
							{#if page.type === 'ellipsis'}
								<Pagination.Item>
									<Pagination.Ellipsis />
								</Pagination.Item>
							{:else}
								<Pagination.Item>
									<Pagination.Link {page} isActive={currentPage == page.value}>
										{page.value}
									</Pagination.Link>
								</Pagination.Item>
							{/if}
						{/each}
						<Pagination.Item>
							<Pagination.NextButton />
						</Pagination.Item>
					</Pagination.Content>
				</Pagination.Root>
			{/if}
		{:catch _}
			<div class="mx-auto text-center">
				<X size={64} class="text-red-500 rounded-full border-2 border-red-500 mx-auto mb-2" />
				<h2 class="text-2xl font-bold">Uh oh!</h2>
				<p class="text-gray-500">Something went wrong. Please try again later.</p>
			</div>
		{/await}
	{/if}
</main>
