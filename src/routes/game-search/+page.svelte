<script lang="ts">
	import GameCard from '$lib/components/GameCard.svelte';
	import * as Form from '$lib/components/ui/form';
	import { Input } from '$lib/components/ui/input';
	import { gameSearchSchema } from '$lib/schemas';
	import { ArrowLeft, Plus, Search, SearchX } from 'lucide-svelte';
	import { defaults, superForm } from 'sveltekit-superforms';
	import { zod, zodClient } from 'sveltekit-superforms/adapters';
	import * as Pagination from '$lib/components/ui/pagination';
	import Button from '$lib/components/ui/button/button.svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { useQuery } from '@sveltestack/svelte-query';
	import ErrorMessage from '$lib/components/ErrorMessage.svelte';
	import type { PageData } from './$types';
	import { searchGame } from '$lib/rust-bindings/igdb';
	import * as Tooltip from '$lib/components/ui/tooltip';

	export let data: PageData;
	let queriedGame = '';
	const gamesPerPage = 9;
	const gameSearchQuery = useQuery(
		['gameSearch', queriedGame],
		async () => {
			const matchingGames = await searchGame(queriedGame);
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

	$: games = $gameSearchQuery.data ? $gameSearchQuery.data : data.randomGames;
</script>

<div class="min-h-full px-16 py-8 container flex flex-col">
	<div class="flex justify-between items-end mb-8">
		<div>
			<h1 class="text-3xl font-heading font-bold">Find a Game</h1>
			<p class="text-muted-foreground">Let's find that game you've been playing...</p>
		</div>
		<form method="post" class="flex justify-center w-full max-w-md" use:gameSearchEnhance>
			<Button
				variant="ghost"
				class="mr-2"
				on:click={() => {
					goto('/logs');
					$gameSearchQuery.remove();
				}}><ArrowLeft size={32} /></Button
			>
			<Form.Field form={gameSearchForm} name="gameTitle" class="flex-1">
				<Form.Control let:attrs>
					<Input
						{...attrs}
						bind:value={$gameSearchFormData.gameTitle}
						class="rounded-r-none"
						placeholder="Search..."
					/>
				</Form.Control>
			</Form.Field>
			<Form.Button class="rounded-l-none"><Search size={24} /></Form.Button>
		</form>
	</div>
	{#if $gameSearchQuery.isError || !games}
		<div class="grid grid-cols-2 lg:grid-cols-3 gap-4">
			{#each Array(gamesPerPage) as _, i}
				<div
					class={`px-4 py-2 border relative rounded-lg group flex gap-4 ${
						i >= gamesPerPage - 3 ? 'hidden lg:flex' : ''
					}`}
				>
					<div class="relative aspect-[3/4] bg-muted flex-1 h-full rounded-lg" />
					<div class="flex-1">
						<div class="mb-4">
							<div class="text-lg bg-muted mb-1 font-semibold h-5 w-24 rounded" />
							<div class="text-sm bg-muted h-3 w-20 rounded" />
						</div>
						<div class="mb-4 h-4 rounded bg-muted w-12" />
						<div class="h-4 w-full bg-muted mb-1 rounded" />
						<div class="h-4 w-full bg-muted mb-1 rounded" />
						<div class="h-4 w-full bg-muted mb-1 rounded" />
						<div class="h-4 w-full bg-muted mb-1 rounded" />
						<div class="h-4 w-full bg-muted mb-1 rounded" />
					</div>
				</div>
			{/each}
			<ErrorMessage error={$gameSearchQuery.error}>Couldn't get any games</ErrorMessage>
		</div>
	{:else if games.length === 0}
		<div class="flex justify-center items-center flex-grow">
			<div class="mx-auto px-16 py-8 w-fit border rounded-xl">
				<SearchX size={64} class="mb-2 mx-auto" />
				<h2 class="text-2xl font-bold">No games found</h2>
				<p class="text-gray-500">Try searching for something else</p>
			</div>
		</div>
	{:else}
		{@const executableName = $page.url.searchParams.get('executableName')}
		{@const minutesPlayed = $page.url.searchParams.get('minutesPlayed')}
		{@const isNewGame = executableName && minutesPlayed}
		<div class="grid grid-cols-3 gap-4">
			{#each games.slice(beginningPageIndex, beginningPageIndex + gamesPerPage) as game}
				<GameCard
					title={game.title}
					cover={game.cover_image_id}
					rating={(game.total_rating ?? 0) / 10 / 2}
				>
					<svelte:fragment slot="actions">
						<Tooltip.Root disableHoverableContent>
							<Tooltip.Trigger>
								<Button
									on:click={() =>
										goto(
											`/logs/edit?gameId=${game.id}` +
												(isNewGame
													? `&executableName=${executableName}&minutesPlayed=${minutesPlayed}`
													: '')
										)}
									variant="ghost"
									size="action"
								>
									<Plus size={16} />
								</Button>
							</Tooltip.Trigger>
							<Tooltip.Content sideOffset={6}>Create log</Tooltip.Content>
						</Tooltip.Root>
					</svelte:fragment>
				</GameCard>
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
</div>
