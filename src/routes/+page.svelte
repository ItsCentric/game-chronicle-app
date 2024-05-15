<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import GameCard from '$lib/components/GameCard.svelte';
	import Statistic from '$lib/components/Statistic.svelte';
	import Skeleton from '$lib/components/ui/skeleton/skeleton.svelte';
	import { goto } from '$app/navigation';
	import { statusOptions, type StatusOption } from '$lib/schemas';
	import { useQuery } from '@sveltestack/svelte-query';
	import ErrorMessage from '$lib/components/ErrorMessage.svelte';
	import { getDashboardStatistics, getLogs, getRecentLogs } from '$lib/rust-bindings/database';
	import { authenticateWithTwitch, getGamesById, getSimilarGames } from '$lib/rust-bindings/igdb';
	import type { PageData } from './$types';

	export let data: PageData;
	const dashboardStatisticsQuery = useQuery('dashboardStatistics', getDashboardStatistics, {
		initialData: data.dashboardStatistics
	});
	const recentLogsQuery = useQuery(
		'recentLogs',
		async () => {
			const accessTokenResponse = await authenticateWithTwitch();
			const recentLogs = await getRecentLogs(
				6,
				statusOptions.filter((status) => status != 'wishlist')
			);
			const recentGameIds = recentLogs.map((log) => log.game.id);
			const games = await getGamesById(accessTokenResponse.access_token, recentGameIds);
			const sortedGames = [];
			for (let i = 0; i < recentGameIds.length; i++) {
				const game = games.find((game) => game.id === recentGameIds[i]);
				if (game) {
					sortedGames.push(game);
				}
			}
			return sortedGames;
		},
		{ initialData: data.recentGames }
	);
	const similarGamesQuery = useQuery(
		'similarGames',
		async () => {
			const logs = await getLogs('date', 'desc', statusOptions as unknown as StatusOption[]);
			const gameIds = logs.map((log) => log.game.id);
			const accessTokenResponse = await authenticateWithTwitch();
			const similarGames = await getSimilarGames(accessTokenResponse.access_token, gameIds);
			return similarGames;
		},
		{ initialData: data.similarGames }
	);
</script>

<main class="flex flex-col gap-12 h-full p-12 container">
	<div>
		<h1 class="font-heading font-bold text-3xl">
			Hello, <span class="capitalize">{data.username}</span>
		</h1>
		<h2 class="text-xl font-heading font-semibold mb-4">Welcome to your journal</h2>
		<div class="flex gap-2">
			<Button href="/logs" data-testid="view-logs">View logs</Button>
			<Button href="/settings" data-testid="settings">Settings</Button>
		</div>
	</div>
	<div class="flex justify-around items-center border-y border-slate-800 py-4 relative">
		{#if $dashboardStatisticsQuery.isLoading}
			<div class="flex flex-col gap-3">
				<Skeleton class="h-4 w-44" />
				<Skeleton class="h-9 w-16" />
				<Skeleton class="h-3 w-48" />
			</div>
			<div class="flex flex-col gap-3">
				<Skeleton class="h-4 w-44" />
				<Skeleton class="h-9 w-16" />
				<Skeleton class="h-3 w-48" />
			</div>
			<div class="flex flex-col gap-3">
				<Skeleton class="h-4 w-44" />
				<Skeleton class="h-9 w-16" />
				<Skeleton class="h-3 w-48" />
			</div>
		{:else if $dashboardStatisticsQuery.isError || !$dashboardStatisticsQuery.data}
			<div class="flex flex-col gap-3">
				<span class="h-4 w-44 bg-white/5 rounded-xl" />
				<span class="h-9 w-16 bg-white/5 rounded-xl" />
				<span class="h-3 w-48 bg-white/5 rounded-xl" />
			</div>
			<div class="flex flex-col gap-3">
				<span class="h-4 w-44 bg-white/5 rounded-xl" />
				<span class="h-9 w-16 bg-white/5 rounded-xl" />
				<span class="h-3 w-48 bg-white/5 rounded-xl" />
			</div>
			<div class="flex flex-col gap-3">
				<span class="h-4 w-44 bg-white/5 rounded-xl" />
				<span class="h-9 w-16 bg-white/5 rounded-xl" />
				<span class="h-3 w-48 bg-white/5 rounded-xl" />
			</div>
			<ErrorMessage error={$dashboardStatisticsQuery.error}
				>Couldn't get your statistics</ErrorMessage
			>
		{:else}
			{@const [thisMonthStatistics, lastMonthStatistics] = $dashboardStatisticsQuery.data}
			{@const hoursPlayed = Math.floor(thisMonthStatistics.total_minutes_played / 60)}
			<Statistic
				lastMonthStat={lastMonthStatistics.total_games_played}
				currentStat={thisMonthStatistics.total_games_played}>Total games</Statistic
			>
			<Statistic
				lastMonthStat={Math.floor(lastMonthStatistics.total_minutes_played / 60)}
				currentStat={hoursPlayed}
				timeStat>Total hours played</Statistic
			>
			<Statistic
				lastMonthStat={lastMonthStatistics.total_games_completed}
				currentStat={thisMonthStatistics.total_games_completed}>Completed games</Statistic
			>
		{/if}
	</div>
	<div>
		<div class="flex justify-between items-center mb-2">
			<h3 class="text-xl font-heading font-semibold">Recently Played</h3>
			<Button variant="link" href="/logs">View all games</Button>
		</div>
		<div class="grid grid-cols-6 gap-4 relative">
			{#if $recentLogsQuery.isLoading}
				{#each Array(6) as _}
					<Skeleton class="h-full w-full aspect-[3/4] rounded-3xl" />
				{/each}
			{:else if $recentLogsQuery.isError || !$recentLogsQuery.data}
				{#each Array(6) as _}
					<span class="h-full w-full aspect-[3/4] bg-white/5 rounded-3xl" />
				{/each}
				<ErrorMessage error={$recentLogsQuery.error}
					>Couldn't get your recently played games</ErrorMessage
				>
			{:else if $recentLogsQuery.data.length === 0}
				{#each Array(6) as _}
					<div class="h-full w-full aspect-[3/4] bg-white/5 rounded-3xl"></div>
				{/each}
				<div
					class="absolute px-4 py-2 bg-black/80 shadow-lg rounded-xl text-center top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2"
				>
					<p class="font-semibold font-heading text-lg">No recently played games</p>
					<p>Start logging your games to see them here</p>
				</div>
			{:else}
				{#each $recentLogsQuery.data as game}
					<GameCard data={game} on:click={() => goto(`/logs/edit?gameId=${game.id}`)} />
				{/each}
			{/if}
		</div>
	</div>
	<div class="pb-16">
		<div class="flex justify-between items-center mb-2">
			<h3 class="text-xl font-heading font-semibold">Similar to What You Play</h3>
			<Button variant="link" href="">View all similar titles</Button>
		</div>
		<div class="grid grid-cols-6 gap-4 relative">
			{#if $similarGamesQuery.isLoading}
				{#each Array(6) as _}
					<Skeleton class="h-full w-full aspect-[3/4] rounded-3xl" />
				{/each}
			{:else if $similarGamesQuery.isError || !$similarGamesQuery.data}
				{#each Array(6) as _}
					<div class="h-full w-full aspect-[3/4] bg-white/5 rounded-3xl"></div>
				{/each}
				<ErrorMessage error={$similarGamesQuery.error}
					>Couldn't get your recommendations</ErrorMessage
				>
			{:else if $similarGamesQuery.data.length === 0}
				{#each Array(6) as _}
					<div class="h-full w-full aspect-[3/4] bg-white/5 rounded-3xl"></div>
				{/each}
				<div
					class="absolute px-4 py-2 bg-black/80 shadow-lg rounded-xl text-center top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2"
				>
					<p class="font-semibold font-heading text-lg">No similar games</p>
					<p>Start logging your games to see your suggestions here</p>
				</div>
			{:else}
				{#each $similarGamesQuery.data.slice(0, 6) as game}
					<GameCard data={game} on:click={() => goto(`/logs/edit?gameId=${game.id}`)} />
				{/each}
			{/if}
		</div>
	</div>
</main>
