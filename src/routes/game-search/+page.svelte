<script lang="ts">
	import GameCard from '$lib/components/GameCard.svelte';
	import * as Form from '$lib/components/ui/form';
	import { Input } from '$lib/components/ui/input';
	import { gameSearchSchema } from '$lib/schemas';
	import { ArrowLeft, LoaderCircle, Search, SearchX } from 'lucide-svelte';
	import { defaults, superForm } from 'sveltekit-superforms';
	import { zod, zodClient } from 'sveltekit-superforms/adapters';
	import * as Pagination from '$lib/components/ui/pagination';
	import Button from '$lib/components/ui/button/button.svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { useQuery } from '@sveltestack/svelte-query';
	import ErrorMessage from '$lib/components/ErrorMessage.svelte';
	import type { PageData } from './$types';
	import { authenticateWithTwitch, searchGame } from '$lib/rust-bindings/igdb';

	export let data: PageData;
	let queriedGame = '';
	const gamesPerPage = 18;
	const gameSearchQuery = useQuery(
		['gameSearch', queriedGame],
		async () => {
			const authenticateRes = await authenticateWithTwitch();
			const matchingGames = await searchGame(authenticateRes.access_token, queriedGame);
			return matchingGames;
		},
		{ enabled: queriedGame.length > 0 }
	);
	let currentPage = 1;
	$: beginningPageIndex = (currentPage - 1) * gamesPerPage;
	const gameSearchForm = superForm(defaults(zod(gameSearchSchema)), {
		validators: zodClient(gameSearchSchema),
		SPA: true,
		resetForm: false,
		onUpdate: async ({ form }) => {
			if (form.valid) {
				queriedGame = form.data.gameTitle;
				$gameSearchQuery.refetch();
			}
		}
	});
	const { form: gameSearchFormData, enhance: gameSearchEnhance } = gameSearchForm;

	$: games = $gameSearchQuery.data?.length ?? 0 > 0 ? $gameSearchQuery.data : data.randomGames;
</script>

<main class="min-h-full px-16 py-8">
	<div class="text-center mb-4">
		<h1 class="text-3xl font-heading font-bold">Find a Game</h1>
		<p class="text-gray-500 font-heading">Let's find that game you've been playing</p>
	</div>
	<form method="post" class="flex justify-center mb-8 mx-auto" use:gameSearchEnhance>
		<Button
			variant="ghost"
			class="mr-4"
			on:click={() => {
				goto('/logs');
				$gameSearchQuery.remove();
			}}><ArrowLeft size={32} /></Button
		>
		<Form.Field form={gameSearchForm} name="gameTitle">
			<Form.Control let:attrs>
				<Input {...attrs} bind:value={$gameSearchFormData.gameTitle} class="rounded-r-none" />
			</Form.Control>
		</Form.Field>
		<Form.Button class="rounded-l-none"><Search size={24} /></Form.Button>
	</form>
	<div class="flex w-full justify-center items-center"></div>
	{#if $gameSearchQuery.isLoading || $gameSearchQuery.isRefetching}
		<LoaderCircle size={64} class="animate-spin mx-auto" />
	{:else if $gameSearchQuery.isError || !games}
		<div class="mx-auto grid grid-cols-6 gap-4 container relative">
			{#each Array(gamesPerPage) as _}
				<span class="h-full aspect-[3/4] rounded-3xl bg-white/5" />
			{/each}
			<ErrorMessage error={$gameSearchQuery.error}>Couldn't get any games</ErrorMessage>
		</div>
	{:else if games.length === 0}
		<div class="mx-auto text-center">
			<SearchX size={64} class="mb-2 mx-auto" />
			<h2 class="text-2xl font-bold">No games found</h2>
			<p class="text-gray-500">Try searching for something else</p>
		</div>
	{:else}
		{@const executableName = $page.url.searchParams.get('executableName')}
		{@const minutesPlayed = $page.url.searchParams.get('minutesPlayed')}
		{@const isNewGame = executableName && minutesPlayed}
		<div class="mx-auto grid grid-cols-6 gap-4 container">
			{#each games.slice(beginningPageIndex, beginningPageIndex + gamesPerPage) as game}
				<GameCard
					data={game}
					on:click={() =>
						goto(
							`/logs/edit?game=${JSON.stringify(game)}` +
								(isNewGame
									? `&executableName=${executableName}&minutesPlayed=${minutesPlayed}`
									: '')
						)}
				/>
			{/each}
		</div>
		<Pagination.Root
			count={games.length}
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
</main>
