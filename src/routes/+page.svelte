<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import GameCard from '$lib/components/GameCard.svelte';
	import Statistic from '$lib/components/Statistic.svelte';
	import { goto } from '$app/navigation';
	import { statusOptions } from '$lib/schemas';
	import { useMutation, useQuery, useQueryClient } from '@sveltestack/svelte-query';
	import ErrorMessage from '$lib/components/ErrorMessage.svelte';
	import {
		deleteLog,
		getDashboardStatistics,
		getLogs,
		getRecentLogs
	} from '$lib/rust-bindings/database';
	import { getGamesById } from '$lib/rust-bindings/igdb';
	import type { PageData } from './$types';
	import { BaseDirectory, readTextFile, exists, remove } from '@tauri-apps/plugin-fs';
	import * as Dialog from '$lib/components/ui/dialog';
	import { onMount } from 'svelte';
	import SvelteMarkdown from 'svelte-markdown';
	import Separator from '$lib/components/ui/separator/separator.svelte';
	import { Library, Pencil, Plus, Settings, Trash } from 'lucide-svelte';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import { toast } from 'svelte-sonner';
	import * as AlertDialog from '$lib/components/ui/alert-dialog';

	export let data: PageData;
	let changeLogContents: string | undefined;
	const dateFormatter = new Intl.DateTimeFormat('en-US', {
		year: 'numeric',
		month: 'long',
		day: 'numeric'
	});
	onMount(async () => {
		if (await exists('resources/changelog.md', { baseDir: BaseDirectory.Resource })) {
			changeLogContents = await readTextFile('resources/changelog.md', {
				baseDir: BaseDirectory.Resource
			});
			await remove('resources/changelog.md', { baseDir: BaseDirectory.Resource });
		}
	});
	const dashboardStatisticsQuery = useQuery(
		'dashboardStatistics',
		async () => {
			const now = new Date();
			const endOfLastMonth = new Date(now.getFullYear(), now.getMonth(), 0);
			const startOfNextMonth = new Date(now.getFullYear(), now.getMonth() + 1, 1);
			const thisMonthStatistics = await getDashboardStatistics(endOfLastMonth, startOfNextMonth);
			const startOfLastMonth = new Date(
				endOfLastMonth.getFullYear(),
				endOfLastMonth.getMonth() - 1,
				1
			);
			const lastMonthStatistics = await getDashboardStatistics(
				new Date(startOfLastMonth.getFullYear(), startOfLastMonth.getMonth(), 0),
				new Date(startOfNextMonth.getFullYear(), startOfNextMonth.getMonth() - 1, 1)
			);

			return [lastMonthStatistics, thisMonthStatistics];
		},
		{
			initialData: data.dashboardStatistics
		}
	);
	const recentLogsQuery = useQuery(
		'recentLogs',
		async () => {
			const recentLogs = await getRecentLogs(
				3,
				statusOptions.filter((status) => status != 'wishlist' && status != 'backlog')
			);
			const recentGameIds = recentLogs.map((log) => log.game_id);
			const games = await getGamesById(recentGameIds);
			const logs = recentLogs.map((log) => {
				const game = games.find((game) => game.id === log.game_id);
				if (!game) throw new Error('Game not found');
				return { ...log, game };
			});
			return logs;
		},
		{ initialData: data.recentLogs }
	);
	const queryClient = useQueryClient();
	const deleteLogMutation = useMutation(deleteLog, {
		onSuccess: () => {
			queryClient.invalidateQueries('recentLogs');
			queryClient.invalidateQueries('dashboardStatistics');
			queryClient.invalidateQueries('similarGames');
		}
	});
	const similarGamesQuery = useQuery(
		'similarGames',
		async () => {
			const logs = await getLogs(
				'end_date',
				'desc',
				statusOptions.filter((status) => status != 'wishlist' && status != 'backlog')
			);
			const gameIds = logs.map((log) => log.game_id);
			const games = await getGamesById(gameIds);
			const similarGameIds = games
				.filter((game) => (game.similar_games?.length ?? 0) > 0)
				.map((game) => game.similar_games as number[])
				.flat();
			const similarGames = await getGamesById(similarGameIds);
			return similarGames;
		},
		{ initialData: data.similarGames }
	);
</script>

<div class="bg-primary py-8 text-primary-foreground">
	<div class="container flex items-center justify-between">
		<div>
			<h1 class="font-heading font-bold text-3xl">
				{#if data.settings.new}
					Welcome, <span class="capitalize">{data.settings.username}</span>
				{:else}
					Welcome back, <span class="capitalize">{data.settings.username}</span>
				{/if}
			</h1>
			{#if data.settings.new && data.settings.process_monitoring.enabled}
				<p class="text-xl">Play some games and check back later!</p>
			{:else if data.settings.new && !data.settings.process_monitoring.enabled}
				<p class="text-xl">Don't forget to log your games after you play!</p>
			{:else}
				<p class="text-xl">Here's what you've been up to lately</p>
			{/if}
		</div>
		<div class="flex gap-4">
			<Tooltip.Root openDelay={0} disableHoverableContent>
				<Tooltip.Trigger>
					<Button
						href="/logs"
						size="icon"
						class="rounded-full h-11 w-11 bg-primary-foreground/20 hover:bg-primary-foreground/30 shadow hover:scale-110 transition-transform"
						><Library size={32} /></Button
					>
				</Tooltip.Trigger>
				<Tooltip.Content sideOffset={6} transitionConfig={{ y: 8, duration: 200 }}
					>Logs</Tooltip.Content
				>
			</Tooltip.Root>
			<Tooltip.Root openDelay={0} disableHoverableContent>
				<Tooltip.Trigger>
					<Button
						href="/settings"
						size="icon"
						class="rounded-full h-11 w-11 bg-primary-foreground/20 hover:bg-primary-foreground/30 shadow hover:scale-110 transition-transform"
						><Settings size={32} /></Button
					>
				</Tooltip.Trigger>
				<Tooltip.Content sideOffset={6} transitionConfig={{ y: 8, duration: 200 }}
					>Settings</Tooltip.Content
				>
			</Tooltip.Root>
		</div>
	</div>
</div>
<div class="flex flex-col gap-12 h-full p-12 container">
	{#if changeLogContents}
		<Dialog.Root open>
			<Dialog.Content class="overflow-auto max-h-[80vh] max-w-prose">
				<Dialog.Header>
					<Dialog.Title class="text-2xl font-heading font-bold">Changelog</Dialog.Title>
					<Dialog.Description class="text-lg">Here's what's new in this update</Dialog.Description>
				</Dialog.Header>
				<Separator />
				<span class="prose prose-invert prose-headings:font-heading">
					<SvelteMarkdown source={changeLogContents} />
				</span>
			</Dialog.Content>
		</Dialog.Root>
	{/if}
	<div class="flex justify-around items-center border-y py-4 relative">
		{#if $dashboardStatisticsQuery.isError || !$dashboardStatisticsQuery.data}
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
			{@const [lastMonthStatistics, thisMonthStatistics] = $dashboardStatisticsQuery.data}
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
			<Button variant="link" href="/logs">View all logs</Button>
		</div>
		<div class="grid grid-cols-2 xl:grid-cols-3 gap-4 relative">
			{#if $recentLogsQuery.isError || !$recentLogsQuery.data}
				{#each Array(3) as _, i}
					<div
						class={`px-4 py-2 border relative rounded-lg group flex gap-4 ${
							i === 2 ? 'hidden xl:flex' : ''
						}`}
					>
						<div class="relative aspect-[3/4] bg-muted flex-1 h-full rounded-lg" />
						<div class="flex-1">
							<div class="mb-4">
								<div class="text-lg bg-muted mb-1 font-semibold h-5 w-24 rounded" />
								<div class="text-sm bg-muted h-3 w-20 rounded" />
							</div>
							<div class="mb-4 h-4 rounded bg-muted w-12" />
							<div class="h-4 w-32 bg-muted mb-1 rounded" />
							<div class="h-4 w-32 bg-muted mb-1 rounded" />
							<div class="h-4 w-32 bg-muted mb-1 rounded" />
							<div class="h-4 w-32 bg-muted mb-1 rounded" />
							<div class="h-4 w-32 bg-muted mb-1 rounded" />
						</div>
					</div>
				{/each}
				<ErrorMessage error={$recentLogsQuery.error}
					>Couldn't get your recently played games</ErrorMessage
				>
			{:else if $recentLogsQuery.data.length === 0}
				{#each Array(3) as _, i}
					<div
						class={`px-4 py-2 border relative rounded-lg group flex gap-4 ${
							i === 2 ? 'hidden xl:flex' : ''
						}`}
					>
						<div class="relative aspect-[3/4] bg-muted flex-1 h-full rounded-lg" />
						<div class="flex-1">
							<div class="mb-4">
								<div class="text-lg bg-muted mb-1 font-semibold h-5 w-24 rounded" />
								<div class="text-sm bg-muted h-3 w-20 rounded" />
							</div>
							<div class="mb-4 h-4 rounded bg-muted w-12" />
							<div class="h-4 w-32 bg-muted mb-1 rounded" />
							<div class="h-4 w-32 bg-muted mb-1 rounded" />
							<div class="h-4 w-32 bg-muted mb-1 rounded" />
							<div class="h-4 w-32 bg-muted mb-1 rounded" />
							<div class="h-4 w-32 bg-muted mb-1 rounded" />
						</div>
					</div>
				{/each}
				<div
					class="absolute px-4 py-2 bg-black/80 shadow-lg rounded-xl text-center top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2"
				>
					<p class="font-semibold font-heading text-lg">No recently played games</p>
					<p>Start logging your games to see them here</p>
				</div>
			{:else}
				{#each $recentLogsQuery.data.slice(0, 3) as log, i}
					<GameCard
						title={log.game.title ?? ''}
						cover={log.game.cover_image_id}
						rating={log.rating}
						status={log.status}
						on:click={() => goto(`/logs/edit?gameId=${log.game_id}`)}
						class={i === 2 ? 'hidden xl:flex' : ''}
					>
						<p slot="sub-title" class="text-sm text-muted-foreground">
							{dateFormatter.format(new Date(log.end_date))}
						</p>
						<p slot="description" class="line-clamp-3 lg:line-clamp-4 text-ellipsis">{log.notes}</p>
						<svelte:fragment slot="actions">
							<Tooltip.Root disableHoverableContent>
								<Tooltip.Trigger>
									<Button href={`/logs/edit?id=${log.id}`} variant="ghost" size="action">
										<Pencil size={16} />
									</Button>
								</Tooltip.Trigger>
								<Tooltip.Content sideOffset={6}>Edit log</Tooltip.Content>
							</Tooltip.Root>
							<AlertDialog.Root>
								<AlertDialog.Trigger asChild let:builder>
									<Tooltip.Root disableHoverableContent>
										<Tooltip.Trigger>
											<Button builders={[builder]} variant="ghost" size="action">
												<Trash size={16} />
											</Button>
										</Tooltip.Trigger>
										<Tooltip.Content sideOffset={6}>Delete log</Tooltip.Content>
									</Tooltip.Root>
								</AlertDialog.Trigger>
								<AlertDialog.Content>
									<AlertDialog.Header>Delete Log</AlertDialog.Header>
									<AlertDialog.Description>
										Are you sure you want to delete this log?
									</AlertDialog.Description>
									<AlertDialog.Footer>
										<AlertDialog.Action
											data-testid="confirm-delete"
											on:click={() =>
												toast.promise($deleteLogMutation.mutateAsync(log.id), {
													loading: 'Deleting log...',
													success: 'Log was successfully deleted!',
													error: 'Something went wrong deleting your log.'
												})}
										>
											Delete
										</AlertDialog.Action>
										<AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
									</AlertDialog.Footer>
								</AlertDialog.Content>
							</AlertDialog.Root>
						</svelte:fragment>
					</GameCard>
				{/each}
			{/if}
		</div>
	</div>
	<div class="pb-16">
		<div class="flex justify-between items-center mb-2">
			<h3 class="text-xl font-heading font-semibold">Similar to What You Play</h3>
			<Button variant="link" href="/similar-games">View all similar titles</Button>
		</div>
		<div class="grid grid-cols-2 xl:grid-cols-3 gap-4 relative">
			{#if $similarGamesQuery.isError || !$similarGamesQuery.data}
				{#each Array(3) as _, i}
					<div
						class={`px-4 py-2 border relative rounded-lg group flex gap-4 ${
							i === 2 ? 'hidden xl:flex' : ''
						}`}
					>
						<div class="relative aspect-[3/4] bg-muted flex-1 h-full rounded-lg" />
						<div class="flex-1">
							<div class="mb-4">
								<div class="text-lg bg-muted mb-1 font-semibold h-5 w-24 rounded" />
								<div class="text-sm bg-muted h-3 w-20 rounded" />
							</div>
							<div class="mb-4 h-4 rounded bg-muted w-12" />
							<div class="h-4 w-32 bg-muted mb-1 rounded" />
							<div class="h-4 w-32 bg-muted mb-1 rounded" />
							<div class="h-4 w-32 bg-muted mb-1 rounded" />
							<div class="h-4 w-32 bg-muted mb-1 rounded" />
							<div class="h-4 w-32 bg-muted mb-1 rounded" />
						</div>
					</div>
				{/each}
				<ErrorMessage error={$similarGamesQuery.error}
					>Couldn't get your recommendations</ErrorMessage
				>
			{:else if $similarGamesQuery.data.length === 0}
				{#each Array(3) as _, i}
					<div
						class={`px-4 py-2 border relative rounded-lg group flex gap-4 ${
							i === 2 ? 'hidden xl:flex' : ''
						}`}
					>
						<div class="relative aspect-[3/4] bg-muted flex-1 h-full rounded-lg" />
						<div class="flex-1">
							<div class="mb-4">
								<div class="text-lg bg-muted mb-1 font-semibold h-5 w-24 rounded" />
								<div class="text-sm bg-muted h-3 w-20 rounded" />
							</div>
							<div class="mb-4 h-4 rounded bg-muted w-12" />
							<div class="h-4 w-32 bg-muted mb-1 rounded" />
							<div class="h-4 w-32 bg-muted mb-1 rounded" />
							<div class="h-4 w-32 bg-muted mb-1 rounded" />
							<div class="h-4 w-32 bg-muted mb-1 rounded" />
							<div class="h-4 w-32 bg-muted mb-1 rounded" />
						</div>
					</div>
				{/each}
				<div
					class="absolute px-4 py-2 bg-black/80 shadow-lg rounded-xl text-center top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2"
				>
					<p class="font-semibold font-heading text-lg">No similar games</p>
					<p>Start logging your games to see your suggestions here</p>
				</div>
			{:else}
				{#each $similarGamesQuery.data.slice(0, 3) as game, i}
					<GameCard
						title={game.title}
						cover={game.cover_image_id}
						rating={(game.total_rating ?? 0) / 10 / 2}
						on:click={() => goto(`/logs/edit?gameId=${game.id}`)}
						class={i === 2 ? 'hidden xl:flex' : ''}
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
			{/if}
		</div>
	</div>
</div>
