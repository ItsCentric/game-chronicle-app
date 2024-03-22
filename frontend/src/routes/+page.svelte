<script lang="ts">
	import { ArrowDownUp, Filter, Settings, X } from 'lucide-svelte';
	import { superForm, defaults, type Infer } from 'sveltekit-superforms';
	import {
		filterFormSchema,
		sortFormSchema,
		statusOptions,
		type SortFormSchema,
		type FilterFormSchema
	} from '$lib/schemas';
	import { zod, zodClient } from 'sveltekit-superforms/adapters';
	import { Button, buttonVariants } from '$lib/components/ui/button';
	import { useMutation, useQuery, useQueryClient } from '@sveltestack/svelte-query';
	import { GetGameLogs, InsertGameLog } from '$lib/wailsjs/go/main/Database';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import Badge from '$lib/components/ui/badge/badge.svelte';
	import * as Card from '$lib/components/ui/card';
	import * as Select from '$lib/components/ui/select';
	import { toast } from 'svelte-sonner';

	const newLogMutation = useMutation(InsertGameLog);
	const queryClient = useQueryClient();
	function applySortAndFilter(data: Infer<SortFormSchema> | Infer<FilterFormSchema>) {
		currentSortAndFilter = { ...currentSortAndFilter, ...data };
		queryClient.invalidateQueries('logs');
	}

	const sortForm = superForm(defaults(zod(sortFormSchema)), {
		validators: zodClient(sortFormSchema),
		SPA: true,
		onUpdate: ({ form }) => {
			if (form.valid) {
				applySortAndFilter(form.data);
			}
		},
		resetForm: false
	});
	const { form: sortFormData, enhance: sortFormEnhance } = sortForm;
	const filterForm = superForm(defaults(zod(filterFormSchema)), {
		validators: zodClient(filterFormSchema),
		SPA: true,
		onUpdate: ({ form }) => {
			if (form.valid) {
				applySortAndFilter(form.data);
			}
		},
		resetForm: false
	});
	const { form: filterFormData, enhance: filterFormEnhance } = filterForm;

	let currentSortAndFilter = {
		sortBy: 'created_at',
		sortOrder: 'desc',
		status: [...statusOptions]
	};
	const logsQueryResult = useQuery(
		[
			'logs',
			currentSortAndFilter.sortBy,
			currentSortAndFilter.sortOrder,
			currentSortAndFilter.status
		],
		async () =>
			await GetGameLogs(
				currentSortAndFilter.sortBy,
				currentSortAndFilter.sortOrder,
				currentSortAndFilter.status
			)
	);
	$: if ($newLogMutation.isSuccess) {
		queryClient.invalidateQueries('logs');
		toast.success('Log created!');
	}
	$: if ($newLogMutation.isError) {
		toast.error('Something went wrong!');
	}
	$: selectedStatuses = $filterFormData.status.map((status) => ({ value: status, label: status }));
</script>

<main class="flex flex-col justify-center items-center h-full p-12">
	<Button href="/settings" class="mb-2"><Settings /></Button>
	<Button href="/game-search" class="mb-2">Create a log</Button>
	<div class="flex justify-end gap-2 mb-2">
		<form id="sortForm" method="post" use:sortFormEnhance />
		<DropdownMenu.Root closeOnItemClick={false}>
			<DropdownMenu.Trigger
				class={buttonVariants({ variant: 'default' }) + ' flex gap-2 items-center'}
			>
				<ArrowDownUp size={16} />
				<p>Sort</p>
			</DropdownMenu.Trigger>
			<DropdownMenu.Content>
				<DropdownMenu.Label>Sort by</DropdownMenu.Label>
				<DropdownMenu.Separator />
				<DropdownMenu.RadioGroup bind:value={$sortFormData.sortBy}>
					<DropdownMenu.RadioItem value="title">Alphabetical</DropdownMenu.RadioItem>
					<DropdownMenu.RadioItem value="time_played_minutes">Time Played</DropdownMenu.RadioItem>
					<DropdownMenu.RadioItem value="created_at">Entry Added</DropdownMenu.RadioItem>
				</DropdownMenu.RadioGroup>
				<DropdownMenu.Separator />
				<DropdownMenu.Label>Sort order</DropdownMenu.Label>
				<DropdownMenu.Separator />
				<DropdownMenu.RadioGroup bind:value={$sortFormData.sortOrder}>
					<DropdownMenu.RadioItem value="asc">Ascending</DropdownMenu.RadioItem>
					<DropdownMenu.RadioItem value="desc">Descending</DropdownMenu.RadioItem>
				</DropdownMenu.RadioGroup>
				<div class="mt-2 flex items-center">
					<Button type="submit" form="sortForm" class="rounded-r-none">Apply</Button>
					<Button
						type="button"
						variant="destructive"
						class="rounded-l-none"
						on:click={() => sortForm.reset()}><X /></Button
					>
				</div>
			</DropdownMenu.Content>
		</DropdownMenu.Root>
		<DropdownMenu.Root closeOnItemClick={false}>
			<DropdownMenu.Trigger
				class={buttonVariants({ variant: 'default' }) + ' flex gap-2 items-center'}
			>
				<Filter size={16} />
				<p>Filter</p>
			</DropdownMenu.Trigger>
			<DropdownMenu.Content class="max-w-xs">
				<form id="filterForm" method="post" use:filterFormEnhance />
				<Select.Root
					selected={selectedStatuses}
					onSelectedChange={(status) => {
						if (status) {
							$filterFormData.status = status.map((status) => status.value);
						} else {
							$filterFormData.status = [];
						}
					}}
					multiple
				>
					<Select.Trigger>
						<Select.Value placeholder="Status to filter..." />
					</Select.Trigger>
					<Select.Content>
						{#each statusOptions as status}
							<Select.Item value={status}>{status}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
				<div class="mt-2 flex items-center">
					<Button type="submit" form="filterForm" class="rounded-r-none">Apply</Button>
					<Button
						type="button"
						variant="destructive"
						class="rounded-l-none"
						on:click={() => filterForm.reset()}><X /></Button
					>
				</div>
			</DropdownMenu.Content>
		</DropdownMenu.Root>
	</div>
	<div class="grid grid-cols-3 gap-4">
		{#if $logsQueryResult.data && !$logsQueryResult.isLoading}
			{#each $logsQueryResult.data as log}
				{@const logStatus = log.statusId}
				<Card.Root>
					<Card.Header>
						<Card.Title>{log.title}</Card.Title>
						<Badge class="w-fit">{logStatus}</Badge>
					</Card.Header>
					<Card.Content class="line-clamp-2">{log.notes}</Card.Content>
				</Card.Root>
			{/each}
		{/if}
	</div>
</main>
