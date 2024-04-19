<script lang="ts">
	import GameCard from '$lib/components/GameCard.svelte';
	import Button from '$lib/components/ui/button/button.svelte';
	import { AuthenticateWithTwitch, DeleteLog, GetGamesById } from '$lib/wailsjs/go/main/App';
	import { GetGameLogs } from '$lib/wailsjs/go/main/App';
	import type { main } from '$lib/wailsjs/go/models';
	import { statusOptions, type StatusOption } from '$lib/schemas';
	import {
		ArrowDownUp,
		ArrowLeft,
		ChevronLeft,
		ChevronRight,
		Plus,
		SearchX,
		Trash
	} from 'lucide-svelte';
	import Skeleton from '$lib/components/ui/skeleton/skeleton.svelte';
	import * as Pagination from '$lib/components/ui/pagination';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { goto } from '$app/navigation';
	import { useMutation, useQuery, useQueryClient } from '@sveltestack/svelte-query';
	import { toast } from 'svelte-sonner';

	type GameLog = Omit<main.Log & { game: main.IgdbGame; statusId: StatusOption }, 'convertValues'>;
	let filteredLogs: GameLog[] = [];
	let statusFilter: StatusOption[] = [];
	let currentLogPage = 1;
	let sortBy = 'date';
	let sortOrder = 'desc';
	const queryClient = useQueryClient();
	const logsQuery = useQuery(['logs', sortBy, sortOrder, statusFilter], async ({ queryKey }) => {
		const [_key, sortBy, sortOrder, statusFilter] = queryKey;
		const logs = await GetGameLogs(
			sortBy as string,
			sortOrder as string,
			statusFilter as StatusOption[]
		);
		const accessTokenResponse = await AuthenticateWithTwitch();
		const gameIds = logs.map((log) => log.gameId);
		const gamesResponse = await GetGamesById(gameIds, accessTokenResponse.access_token);
		if (gamesResponse.error) {
			throw new Error(gamesResponse.error);
		}
		return logs.map((log) => {
			const game = gamesResponse.games.find((game) => game.id === log.gameId);
			if (!game) {
				throw new Error('Game not found');
			}
			return { ...log, statusId: log.statusId as StatusOption, game };
		});
	});
	const deleteLogMutation = useMutation(
		async (id: number) => {
			const deleteLogError = await DeleteLog(id);
			if (deleteLogError != '') {
				throw new Error(deleteLogError);
			}
		},
		{ onSuccess: () => queryClient.invalidateQueries('logs') }
	);
	const logStatusColorMap: Record<StatusOption, string> = {
		Backlog: 'bg-gray-500',
		Wishlist: 'bg-blue-500',
		Playing: 'bg-green-500',
		Played: 'bg-green-500',
		Completed: 'bg-green-500',
		Abandoned: 'bg-red-500',
		Retired: 'bg-yellow-500'
	};

	function logStatusColor(status: StatusOption) {
		return logStatusColorMap[status];
	}
	$: if (statusFilter && $logsQuery.data) {
		if (statusFilter.length === 0) {
			filteredLogs = $logsQuery.data;
		} else {
			filteredLogs = $logsQuery.data.filter((log) => {
				return statusFilter.includes(log.statusId as StatusOption);
			});
		}
	}
	$: filteredLogs = filteredLogs.sort((a, b) => {
		switch (sortBy) {
			case 'title':
				if (sortOrder === 'desc') {
					return b.game.name.localeCompare(a.game.name);
				}
				return a.game.name.localeCompare(b.game.name);
			case 'date':
				if (sortOrder === 'desc') {
					return new Date(b.date).getTime() - new Date(a.date).getTime();
				}
				return new Date(a.date).getTime() - new Date(b.date).getTime();
			case 'rating':
				if (sortOrder === 'desc') {
					return b.rating - a.rating;
				}
				return a.rating - b.rating;
			default:
				return 0;
		}
	});
	$: start = (currentLogPage - 1) * 18;
	$: end = currentLogPage * 18;
</script>

<main class="min-h-full flex flex-col gap-4 container py-12 px-16 xl:px-0">
	<div class="flex justify-between items-center relative">
		<Button
			variant="ghost"
			size="icon"
			class="absolute -left-4 -translate-x-full"
			on:click={() => window.history.back()}><ArrowLeft size={32} /></Button
		>
		<h1 class="font-heading text-3xl font-bold">Your Logs</h1>
	</div>
	<div class="flex justify-between items-center">
		<div class="flex gap-2 items-center">
			{#each statusOptions as status}
				{@const active = statusFilter.includes(status)}
				<Button
					variant="outline"
					size="sm"
					class={`rounded-3xl px-4 py-2 ${active ? 'bg-accent text-accent-foreground' : ''}`}
					on:click={() => {
						if (active) {
							statusFilter = statusFilter.filter((s) => s !== status);
						} else {
							statusFilter = [...statusFilter, status];
						}
					}}
				>
					{status}
				</Button>
			{/each}
		</div>
		<div class="flex gap-2 items-center">
			<DropdownMenu.Root>
				<DropdownMenu.Trigger asChild let:builder>
					<Button builders={[builder]}>
						<ArrowDownUp size="1.5em" class="mr-1" />
						<p>Sort</p>
					</Button>
				</DropdownMenu.Trigger>
				<DropdownMenu.Content class="w-56">
					<DropdownMenu.Label>Sort by</DropdownMenu.Label>
					<DropdownMenu.Separator />
					<DropdownMenu.RadioGroup bind:value={sortBy}>
						<DropdownMenu.RadioItem value="title">Name</DropdownMenu.RadioItem>
						<DropdownMenu.RadioItem value="date">Date</DropdownMenu.RadioItem>
						<DropdownMenu.RadioItem value="rating">Rating</DropdownMenu.RadioItem>
					</DropdownMenu.RadioGroup>
					<DropdownMenu.Separator />
					<DropdownMenu.Label>Order</DropdownMenu.Label>
					<DropdownMenu.Separator />
					<DropdownMenu.RadioGroup bind:value={sortOrder}>
						<DropdownMenu.RadioItem value="asc">Ascending</DropdownMenu.RadioItem>
						<DropdownMenu.RadioItem value="desc">Descending</DropdownMenu.RadioItem>
					</DropdownMenu.RadioGroup>
				</DropdownMenu.Content>
			</DropdownMenu.Root>
			<Button href="/game-search">
				<Plus size="1.5em" class="mr-1" />
				<p>Add log</p>
			</Button>
		</div>
	</div>
	{#if $logsQuery.isLoading}
		<div class="grid gap-2 grid-cols-6">
			{#each Array(18) as _}
				<Skeleton class="rounded-3xl aspect-[3/4]" />
			{/each}
		</div>
	{:else if $logsQuery.isSuccess}
		{#if filteredLogs.length === 0}
			<div class="flex-1 flex flex-col gap-1 items-center justify-center">
				<SearchX size={64} />
				<div class="text-center">
					<h2 class="text-2xl font-heading font-bold">Nothing here...</h2>
					<p class="text-lg font-heading font-semibold text-slate-500">
						Try <a href="game-search" class="hover:underline text-accent">adding a new log</a> to get
						started!
					</p>
				</div>
			</div>
		{:else}
			<div class="grid gap-2 grid-cols-6">
				{#each filteredLogs.slice(start, end) as gameLog}
					<GameCard
						data={gameLog.game}
						on:click={() => goto(`/logs/edit?id=${gameLog.ID}&gameId=${gameLog.game.id}`)}
					>
						<AlertDialog.Root>
							<AlertDialog.Trigger asChild let:builder>
								<Button
									on:click={(e) => e.stopPropagation()}
									builders={[builder]}
									variant="ghost"
									size="icon"
									class="z-30 absolute top-0 right-0 opacity-0 group-hover:opacity-100"
								>
									<Trash size={24} />
								</Button>
							</AlertDialog.Trigger>
							<AlertDialog.Content>
								<AlertDialog.Header>Delete Log</AlertDialog.Header>
								<AlertDialog.Description>
									Are you sure you want to delete this log?
								</AlertDialog.Description>
								<AlertDialog.Footer>
									<AlertDialog.Action
										on:click={() =>
											toast.promise($deleteLogMutation.mutateAsync(gameLog.ID), {
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
						<span
							class={`absolute left-2 shadow-black shadow text-black bottom-2 rounded-2xl px-2 py-1 text-sm pointer-events-none ${logStatusColor(
								gameLog.statusId
							)}`}>{gameLog.statusId}</span
						>
					</GameCard>
				{/each}
			</div>
		{/if}
		<Pagination.Root count={filteredLogs.length} perPage={18} let:pages bind:page={currentLogPage}>
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
							<Pagination.Link {page} isActive={currentLogPage === page.value}>
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
	{:else}
		<p class="text-red-500">{$logsQuery.error}</p>
	{/if}
</main>
