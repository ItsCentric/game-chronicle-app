<script lang="ts">
	import type { main } from '$lib/wailsjs/go/models';
	import { Swipe, SwipeItem } from 'svelte-swipe';
	import GameCard from './GameCard.svelte';
	import { createEventDispatcher, onMount } from 'svelte';
	import { ArrowLeft, ArrowRight } from 'lucide-svelte';

	export let games: main.IgdbGame[];
	let paginatedGames: main.IgdbGame[][] = [];
	let swipeElement: Swipe;
	let swipeContainer: HTMLDivElement | undefined;
    let swipeItemInner: Element | null;

    onMount(() => {
        swipeItemInner = document.querySelector('.swipeable-item-inner');
    });

	$: {
		paginatedGames = [];
		for (let i = 0; i < games.length; i += 3) {
			paginatedGames.push(games.slice(i, i + 3));
		}
	}
	$: if (swipeItemInner?.clientHeight && swipeContainer) {
		swipeContainer.style.height = swipeItemInner.clientHeight + 'px';
	}

	const gameClickDispatcher = createEventDispatcher();
</script>

<div class="h-full w-full flex items-center gap-2" bind:this={swipeContainer}>
	{#if paginatedGames.length > 1}
		<button class="btn btn-circle btn-outline" on:click={() => swipeElement.prevItem()}
			><ArrowLeft size={32} /></button
		>
	{/if}
	<Swipe bind:this={swipeElement}>
		{#each paginatedGames as gamesList}
			<SwipeItem>
				<div class="flex justify-center gap-4 pointer-events-auto">
					{#each gamesList as game}
						<GameCard data={game} on:click={() => gameClickDispatcher('gameClick', game)} />
					{/each}
				</div>
			</SwipeItem>
		{/each}
	</Swipe>
	{#if paginatedGames.length > 1}
		<button class="btn btn-circle btn-outline" on:click={() => swipeElement.nextItem()}
			><ArrowRight size={32} /></button>
	{/if}
</div>
