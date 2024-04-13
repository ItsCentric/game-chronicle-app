<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import GameCard from '$lib/components/GameCard.svelte';
	import { onMount } from 'svelte';
	import {
		AuthenticateWithTwitch,
		GetCurrentUsername,
		GetGamesById,
		GetSimilarGames
	} from '$lib/wailsjs/go/main/App';
	import type { main } from '$lib/wailsjs/go/models';
	import {
		GetDashboardStatistics,
		GetGameLogs,
		GetRecentLogs
	} from '$lib/wailsjs/go/main/Database';
	import Statistic from '$lib/components/Statistic.svelte';
	import Skeleton from '$lib/components/ui/skeleton/skeleton.svelte';
	import { LoaderCircle } from 'lucide-svelte';
	import { goto } from '$app/navigation';

	let dashboardStatsPromise: Promise<main.GetDashboardStatisticsResponse> | undefined = undefined;
	let recentLogsPromise: Promise<main.IgdbGame[]> | undefined = undefined;
	let similarGamesPromise: Promise<main.IgdbGame[]> | undefined = undefined;
	let username: string | undefined;

	onMount(async () => {
		const userResponse = await GetCurrentUsername();
		if (userResponse.error) {
			console.error('Failed to get current user');
			return;
		}
		username = userResponse.username;
		const authResponse = await AuthenticateWithTwitch();
		if (!authResponse.access_token) {
			console.error('Failed to authenticate with Twitch');
			return;
		}
		dashboardStatsPromise = new Promise((resolve, reject) => {
			GetDashboardStatistics().then((response) => {
				if (response.error) {
					console.error(response);
					reject(response.error);
				}
				resolve(response);
			});
		});
		const recentLogsResponse = await GetRecentLogs(6);
		if (recentLogsResponse.error) {
			console.error('Failed to get recent logs');
			return;
		}
		recentLogsPromise = new Promise((resolve, reject) => {
			let recentGameIds = recentLogsResponse.logs.map((log) => log.gameId);
			GetGamesById(recentGameIds, authResponse.access_token).then((response) => {
				if (response.error) {
					reject(response.error);
				}
				const sortedGames = [];
				for (let i = 0; i < recentGameIds.length; i++) {
					const game = response.games.find((game) => game.id === recentGameIds[i]);
					if (game) {
						sortedGames.push(game);
					}
				}
				resolve(sortedGames);
			});
		});
		const logsResponse = await GetGameLogs('', '', []);
		if (logsResponse.length === 0) {
			console.error('Failed to get game logs');
			return;
		}
		const gameIds = logsResponse.map((log) => log.gameId);
		similarGamesPromise = new Promise((resolve, reject) => {
			GetSimilarGames(gameIds, authResponse.access_token).then((response) => {
				if (response.error) {
					reject(response.error);
				}
				resolve(response.games.slice(0, 6));
			});
		});
	});
</script>

{#if !dashboardStatsPromise || !recentLogsPromise || !similarGamesPromise || !username}
	<div class="h-full w-full flex justify-center items-center">
		<LoaderCircle class="w-16 h-16 animate-spin" />
	</div>
{:else}
	<main class="flex flex-col gap-12 h-full p-12 container">
		<div>
			<h1 class="font-heading font-bold text-3xl">
				Hello, <span class="capitalize">{username}</span>
			</h1>
			<h2 class="text-xl font-heading font-semibold mb-4">Welcome to your journal</h2>
			<Button href="/game-search">Create a log</Button>
		</div>
		<div class="flex justify-around items-center border-y border-slate-800 py-4">
			{#if dashboardStatsPromise}
				{#await dashboardStatsPromise}
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
				{:then statisticsResponse}
					{@const hoursPlayed = Math.floor(statisticsResponse.thisMonthStatistics.timePlayed / 60)}
					<Statistic
						lastMonthStat={statisticsResponse.lastMonthStatistics.totalGames}
						currentStat={statisticsResponse.thisMonthStatistics.totalGames}>Total games</Statistic
					>
					<Statistic
						lastMonthStat={Math.floor(statisticsResponse.lastMonthStatistics.timePlayed / 60)}
						currentStat={hoursPlayed}
						timeStat>Total hours played</Statistic
					>
					<Statistic
						lastMonthStat={statisticsResponse.lastMonthStatistics.completedGames}
						currentStat={statisticsResponse.thisMonthStatistics.completedGames}
						>Completed games</Statistic
					>
				{:catch error}
					<div>Error: {error}</div>
				{/await}
			{/if}
		</div>
		<div>
			<div class="flex justify-between items-center mb-2">
				<h3 class="text-xl font-heading font-semibold">Recent Games</h3>
				<Button variant="link" href="">View all games</Button>
			</div>
			<div class="flex gap-4">
				{#if recentLogsPromise}
					{#await recentLogsPromise}
						{#each Array(6) as _}
							<Skeleton class="h-full w-full aspect-[3/4] rounded-3xl" />
						{/each}
					{:then recentLogsResponse}
						{#each recentLogsResponse as game}
							<GameCard data={game} on:click={() => goto(`/log?gameId=${game.id}`)} />
						{/each}
					{:catch error}
						<div>Error: {error}</div>
					{/await}
				{/if}
			</div>
		</div>
		<div class="pb-16">
			<div class="flex justify-between items-center mb-2">
				<h3 class="text-xl font-heading font-semibold">Similar to What You Play</h3>
				<Button variant="link" href="">View all similar titles</Button>
			</div>
			<div class="flex gap-4">
				{#if similarGamesPromise}
					{#await similarGamesPromise}
						{#each Array(6) as _}
							<Skeleton class="h-full w-full aspect-[3/4] rounded-3xl" />
						{/each}
					{:then similarGames}
						{#each similarGames as game}
							<GameCard data={game} on:click={() => goto(`/log?gameId=${game.id}`)} />
						{/each}
					{:catch error}
						<div>Error: {error}</div>
					{/await}
				{/if}
			</div>
		</div>
	</main>
{/if}
