<script lang="ts">
	import GameCard from '$lib/components/GameCard.svelte';
	import type { PageData } from './$types';
	import * as Pagination from '$lib/components/ui/pagination';
	import { ArrowLeft, ChevronLeft, ChevronRight, Plus } from 'lucide-svelte';
	import Button from '$lib/components/ui/button/button.svelte';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import { goto } from '$app/navigation';

	export let data: PageData;
	let currentGamePage = 1;
	$: start = (currentGamePage - 1) * 12;
	$: end = currentGamePage * 12;
</script>

<main class="container p-12">
	<div class="mb-6 flex gap-2 items-center">
		<Button href="/" size="icon" variant="ghost">
			<ArrowLeft size={48} />
		</Button>
		<h1 class="font-heading font-bold text-3xl">Similar Games</h1>
	</div>
	<div class="grid grid-cols-3 gap-2 mb-4">
		{#each data.similarGames.slice(start, end) as game}
			<GameCard
				title={game.title}
				cover={game.cover_image_id}
				rating={(game.total_rating ?? 0) / 10 / 2}
				on:click={() => goto(`/logs/edit?gameId=${game.id}`)}
			>
				<svelte:fragment slot="actions">
					<Tooltip.Root disableHoverableContent>
						<Tooltip.Trigger>
							<Button href={`/logs/edit?gameId=${game.id}`} variant="ghost" size="action">
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
		count={data.similarGames.length}
		perPage={12}
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
