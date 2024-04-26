<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import GameCard from '$lib/components/GameCard.svelte';
	import {
		AuthenticateWithTwitch,
		GetCurrentUsername,
		GetGamesById,
		GetSimilarGames,
		GetDashboardStatistics,
		GetRecentLogs,
		GetGameLogs
	} from '$lib/wailsjs/go/main/App';
	import Statistic from '$lib/components/Statistic.svelte';
	import Skeleton from '$lib/components/ui/skeleton/skeleton.svelte';
	import { goto } from '$app/navigation';
	import { statusOptions } from '$lib/schemas';
	import { useQuery } from '@sveltestack/svelte-query';

	const dashboardStatisticsQuery = useQuery('dashboardStatistics', async () => {
		const response = await GetDashboardStatistics();
		if (response.error) {
			throw new Error(response.error);
		}
		return response;
	});
	const recentLogsQuery = useQuery('recentLogs', async () => {
		const authResponse = await AuthenticateWithTwitch();
		const response = await GetRecentLogs(
			6,
			statusOptions.filter((status) => status != 'Wishlist')
		);
		if (response.error) {
			throw new Error(response.error);
		}
		const recentGameIds = response.logs.map((log) => log.gameId);
		const gamesResponse = await GetGamesById(recentGameIds, authResponse.access_token);
		if (gamesResponse.error) {
			throw new Error(gamesResponse.error);
		}
		const sortedGames = [];
		for (let i = 0; i < recentGameIds.length; i++) {
			const game = gamesResponse.games.find((game) => game.id === recentGameIds[i]);
			if (game) {
				sortedGames.push(game);
			}
		}
		return sortedGames;
	});
	const similarGamesQuery = useQuery('similarGames', async () => {
		const authResponse = await AuthenticateWithTwitch();
		const logsResponse = await GetGameLogs(
			'',
			'',
			statusOptions.filter((status) => status != 'Wishlist')
		);
		const gameIds = logsResponse.map((log) => log.gameId);
		const response = await GetSimilarGames(gameIds, authResponse.access_token);
		if (response.error) {
			throw new Error(response.error);
		}
		return response.games.slice(0, 6);
	});
	const usernameQuery = useQuery('username', async () => {
		const response = await GetCurrentUsername();
		if (response.error) {
			throw new Error(response.error);
		}
		return response.username;
	});
</script>

<main class="flex flex-col gap-12 h-full p-12 container">
	<div>
		<h1 class="font-heading font-bold text-3xl">
			Hello, <span class="capitalize">{$usernameQuery.data}</span>
		</h1>
		<h2 class="text-xl font-heading font-semibold mb-4">Welcome to your journal</h2>
		<div class="flex gap-2">
			<Button href="/logs" data-testid="view-logs">View logs</Button>
			<Button href="/settings" data-testid="settings">Settings</Button>
		</div>
	</div>
	<div class="flex justify-around items-center border-y border-slate-800 py-4">
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
			<p>error</p>
		{:else}
			{@const hoursPlayed = Math.floor(
				$dashboardStatisticsQuery.data.thisMonthStatistics.timePlayed / 60
			)}
			<Statistic
				lastMonthStat={$dashboardStatisticsQuery.data.lastMonthStatistics.totalGames}
				currentStat={$dashboardStatisticsQuery.data.thisMonthStatistics.totalGames}
				>Total games</Statistic
			>
			<Statistic
				lastMonthStat={Math.floor(
					$dashboardStatisticsQuery.data.lastMonthStatistics.timePlayed / 60
				)}
				currentStat={hoursPlayed}
				timeStat>Total hours played</Statistic
			>
			<Statistic
				lastMonthStat={$dashboardStatisticsQuery.data.lastMonthStatistics.completedGames}
				currentStat={$dashboardStatisticsQuery.data.thisMonthStatistics.completedGames}
				>Completed games</Statistic
			>
		{/if}
	</div>
	<div>
		<div class="flex justify-between items-center mb-2">
			<h3 class="text-xl font-heading font-semibold">Recently Played</h3>
			<Button variant="link" href="/logs">View all games</Button>
		</div>
		<div class="flex gap-4">
			{#if $recentLogsQuery.isLoading}
				{#each Array(6) as _}
					<Skeleton class="h-full w-full aspect-[3/4] rounded-3xl" />
				{/each}
			{:else if $recentLogsQuery.isError || !$recentLogsQuery.data}
				<p>{$recentLogsQuery.error}</p>
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
		<div class="flex gap-4">
			{#if $similarGamesQuery.isLoading}
				{#each Array(6) as _}
					<Skeleton class="h-full w-full aspect-[3/4] rounded-3xl" />
				{/each}
			{:else if $similarGamesQuery.isError || !$similarGamesQuery.data}
				<p>{$similarGamesQuery.error}</p>
			{:else}
				{#each $similarGamesQuery.data as game}
					<GameCard data={game} on:click={() => goto(`/logs/edit?gameId=${game.id}`)} />
				{/each}
			{/if}
		</div>
	</div>
</main>
