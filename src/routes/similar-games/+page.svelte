<script lang="ts">
	import GameCard from '$lib/components/GameCard.svelte';
	import type { PageData } from './$types';
	import * as Pagination from '$lib/components/ui/pagination';
	import { ArrowLeft, ChevronLeft, ChevronRight } from 'lucide-svelte';
	import Button from '$lib/components/ui/button/button.svelte';
	import { goto } from '$app/navigation';

	export let data: PageData;
	let currentGamePage = 1;
	$: start = (currentGamePage - 1) * 18;
	$: end = currentGamePage * 18;
</script>

<main class="container p-12">
	<div class="mb-6 flex gap-2 items-center">
		<Button href="/" size="icon" variant="ghost">
			<ArrowLeft size={48} />
		</Button>
		<h1 class="font-heading font-bold text-3xl">Similar Games</h1>
	</div>
	<div class="grid grid-cols-6 gap-2 mb-4">
		{#each data.similarGames.slice(start, end) as game}
			<GameCard data={game} on:click={() => goto(`/logs/edit?gameId=${game.id}`)} />
		{/each}
	</div>
	<Pagination.Root
		count={data.similarGames.length}
		perPage={18}
		let:pages
		bind:page={currentGamePage}
	>
		<Pagination.Content>
			<Pagination.Item>
				<Pagination.PrevButton>
					<ChevronLeft class="h-4 w-4" />
					<span class="hidden sm:block">Previous</span>
				</Pagination.PrevButton>
			</Pagination.Item>
			{#each pages as page (page.key)}
				{#if page.type === 'ellipsis'}
					<Pagination.Item>
						<Pagination.Ellipsis />
					</Pagination.Item>
				{:else}
					<Pagination.Item>
						<Pagination.Link {page} isActive={currentGamePage === page.value}>
							{page.value}
						</Pagination.Link>
					</Pagination.Item>
				{/if}
			{/each}
			<Pagination.Item>
				<Pagination.NextButton>
					<span class="hidden sm:block">Next</span>
					<ChevronRight class="h-4 w-4" />
				</Pagination.NextButton>
			</Pagination.Item>
		</Pagination.Content>
	</Pagination.Root>
</main>
