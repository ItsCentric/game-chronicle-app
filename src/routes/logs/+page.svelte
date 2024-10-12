<script lang="ts">
	import GameCard from '$lib/components/GameCard.svelte';
	import Button from '$lib/components/ui/button/button.svelte';
	import { statusOptions, type StatusOption } from '$lib/schemas';
	import {
		ArrowDownUp,
		ArrowLeft,
		ChevronLeft,
		ChevronRight,
		Pencil,
		Plus,
		SearchX,
		Trash
	} from 'lucide-svelte';
	import * as Pagination from '$lib/components/ui/pagination';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { goto } from '$app/navigation';
	import { useMutation, useQuery, useQueryClient } from '@sveltestack/svelte-query';
	import { toast } from 'svelte-sonner';
	import { deleteLog, getLogs } from '$lib/rust-bindings/database';
	import type { PageData } from './$types';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import ErrorMessage from '$lib/components/ErrorMessage.svelte';
	import { toTitleCase } from '$lib';
	import { getGamesById } from '$lib/rust-bindings/igdb';
	import * as Tooltip from '$lib/components/ui/tooltip';

	export let data: PageData;

	let filteredLogs: PageData['logsAndGames'] = [];
	let statusFilter: StatusOption[] = [];
	let currentLogPage = 1;
	let sortBy = 'end_date';
	let sortOrder: 'desc' | 'asc' = 'desc';
	const queryClient = useQueryClient();
	const deleteLogMutation = useMutation(deleteLog, {
		onSuccess: (deletedLogId) => {
			queryClient.invalidateQueries('logs');
			data.logsAndGames = data.logsAndGames.filter((log) => log.id !== deletedLogId);
		}
	});
	const logsQuery = useQuery(
		'logs',
		async () => {
			const logs = await getLogs(sortBy, sortOrder, statusFilter);
			const games = await getGamesById(logs.map((log) => log.game_id));
			let logsAndGames = logs.map((log) => {
				let associatedGame = games.find((game) => game.id === log.game_id);
				if (!associatedGame) {
					throw new Error(`Game with id ${log.game_id} not found`);
				}
				return { ...log, game: associatedGame };
			});
			return logsAndGames;
		},
		{ initialData: data.logsAndGames }
	);

	const dateFormatter = new Intl.DateTimeFormat('en-US', {
		year: 'numeric',
		month: 'long',
		day: 'numeric'
	});
	$: if (statusFilter) {
		if (statusFilter.length === 0) {
			filteredLogs = data.logsAndGames;
		} else {
			filteredLogs = data.logsAndGames.filter((log) => {
				return statusFilter.includes(log.status);
			});
		}
	}
	$: filteredLogs = ($logsQuery.data ?? data.logsAndGames).sort((a, b) => {
		switch (sortBy) {
			case 'title':
				if (sortOrder === 'desc') {
					return b.game.title.localeCompare(a.game.title);
				}
				return a.game.title.localeCompare(b.game.title);
			case 'end_date':
				if (sortOrder === 'desc') {
					return new Date(b.end_date).getTime() - new Date(a.end_date).getTime();
				}
				return new Date(a.end_date).getTime() - new Date(b.end_date).getTime();
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

<main class="min-h-full flex flex-col gap-4 container py-12 px-16 xl:px-8">
	<div class="flex gap-4 items-center">
		<Button variant="ghost" size="icon" on:click={() => goto('/')}><ArrowLeft size={32} /></Button>
		<h1 class="font-heading text-3xl font-bold">Your Logs</h1>
	</div>
	<div id="filterContainer" class="flex justify-between items-center gap-4">
		<div class="flex gap-2 items-center min-w-0 overflow-auto">
			{#each statusOptions as status}
				{@const active = statusFilter.includes(status)}
				<Button
					variant="outline"
					size="sm"
					class={`rounded-3xl px-4 py-2 ${active ? 'bg-accent text-accent-foreground' : ''}`}
					disabled={$logsQuery.isLoading || $logsQuery.isError}
					on:click={() => {
						if (active) {
							statusFilter = statusFilter.filter((s) => s !== status);
						} else {
							statusFilter = [...statusFilter, status];
						}
					}}
				>
					{toTitleCase(status)}
				</Button>
			{/each}
		</div>
		<div class="flex gap-2 items-center">
			<DropdownMenu.Root>
				<DropdownMenu.Trigger asChild let:builder>
					<Button
						builders={[builder]}
						variant="secondary"
						disabled={$logsQuery.isLoading || $logsQuery.isError}
					>
						<ArrowDownUp size="1.5em" class="mr-1" />
						<p>Sort</p>
					</Button>
				</DropdownMenu.Trigger>
				<DropdownMenu.Content class="w-56">
					<DropdownMenu.Label>Sort by</DropdownMenu.Label>
					<DropdownMenu.Separator />
					<DropdownMenu.RadioGroup bind:value={sortBy}>
						<DropdownMenu.RadioItem value="title">Name</DropdownMenu.RadioItem>
						<DropdownMenu.RadioItem value="end_date">Date</DropdownMenu.RadioItem>
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
			<Button href="/game-search" data-testid="add-log">
				<Plus size="1.5em" class="mr-1" />
				<p>Add log</p>
			</Button>
		</div>
	</div>
	{#if $logsQuery.isLoading}
		<div class="grid gap-2 grid-cols-3">
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
			<div class="grid gap-2 grid-cols-2 xl:grid-cols-3">
				{#each filteredLogs.slice(start, end) as gameLog}
					<GameCard
						title={gameLog.game.title ?? ''}
						cover={gameLog.game.cover_image_id}
						rating={gameLog.rating}
						status={gameLog.status}
						on:click={() => goto(`/logs/edit?gameId=${gameLog.game_id}`)}
					>
						<p slot="sub-title" class="text-sm text-muted-foreground">
							{dateFormatter.format(new Date(gameLog.end_date))}
						</p>
						<p slot="description" class="line-clamp-3 lg:line-clamp-4 text-ellipsis">
							{gameLog.notes}
						</p>
						<svelte:fragment slot="actions">
							<Tooltip.Root disableHoverableContent>
								<Tooltip.Trigger>
									<Button href={`/logs/edit?id=${gameLog.id}`} variant="ghost" size="icon">
										<Pencil size={16} />
									</Button>
								</Tooltip.Trigger>
								<Tooltip.Content sideOffset={6}>Edit log</Tooltip.Content>
							</Tooltip.Root>
							<AlertDialog.Root>
								<AlertDialog.Trigger asChild let:builder>
									<Tooltip.Root disableHoverableContent>
										<Tooltip.Trigger>
											<Button builders={[builder]} variant="ghost" size="icon">
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
												toast.promise($deleteLogMutation.mutateAsync(gameLog.id), {
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
		<div class="grid grid-cols-6 gap-2 relative">
			<ErrorMessage error={$logsQuery.error}>Couldn't get your logs</ErrorMessage>
			{#each Array(18) as _}
				<span class="rounded-3xl bg-white/5 aspect-[3/4]" />
			{/each}
		</div>
	{/if}
</main>

<style>
	#filterContainer::-webkit-scrollbar:horizontal {
		width: 12px;
	}

	#filterContainer::-webkit-scrollbar-track:horizontal {
		border-radius: 8px;
		background-color: #e7e7e7;
		border: 1px solid #cacaca;
		box-shadow: inset 0 0 6px rgba(0, 0, 0, 0.3);
	}

	#filterContainer::-webkit-scrollbar-thumb:horizontal {
		border-radius: 8px;
		background-color: #363636;
	}
</style>
