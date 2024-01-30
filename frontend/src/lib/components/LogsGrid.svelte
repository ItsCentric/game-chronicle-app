<script lang="ts">
	import { GetGameLogs } from '$lib/wailsjs/go/main/Database';
	import { ArrowDownUp, Filter, X } from 'lucide-svelte';
	import GameStatusBadge from './GameStatusBadge.svelte';
	import { useQuery, useQueryClient } from '@sveltestack/svelte-query';
	import { createForm } from 'felte';
	import { validator } from '@felte/validator-zod';
	import { reporter } from '@felte/reporter-svelte';
	import Select from 'svelte-select';
	import {
		type SortFormData,
		type FilterFormData,
		sortFormSchema,
		filterFormSchema,
		statusOptions
	} from '$lib/schemas';

	let selectElement: Select | undefined;
	const queryClient = useQueryClient();
	function applySortAndFilter(data: SortFormData | FilterFormData) {
		currentSortAndFilter = { ...currentSortAndFilter, ...data };
		queryClient.invalidateQueries('logs');
	}
	const {
		form: sortForm,
		isValid: isSortValid,
		reset: sortReset
	} = createForm<SortFormData>({
		extend: [validator({ schema: sortFormSchema }), reporter],
		onSubmit: async (data) => applySortAndFilter(data)
	});
	const {
		form: filterForm,
		data: filterFormData,
		isValid: isFilterValid,
		reset: filterReset
	} = createForm<FilterFormData>({
		extend: [validator({ schema: filterFormSchema }), reporter],
		onSubmit: async (data) => applySortAndFilter(data),
		initialValues: {
			status: [...statusOptions]
		}
	});
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
</script>

<main>
	<div class="flex justify-end gap-2 mb-2">
		<div class="dropdown">
			<div tabindex="0" role="button" class="btn">
				<ArrowDownUp size={16} />
				<p>Sort</p>
			</div>
			<!-- svelte-ignore a11y-no-noninteractive-tabindex -->
			<form
				method="post"
				tabindex={0}
				class="dropdown-content bg-base-100 z-10 p-2 shadow rounded-box"
				use:sortForm
			>
				<div class="form-control">
					<label class="label">
						<input type="radio" name="sortBy" value="title" class="radio" />
						<span class="label-text">Alphabetical</span>
					</label>
				</div>
				<div class="form-control">
					<label class="label">
						<input type="radio" name="sortBy" value="time_played_minutes" class="radio" />
						<span class="label-text">Time Played</span>
					</label>
				</div>
				<div class="form-control">
					<label class="label">
						<input type="radio" name="sortBy" value="started_on" class="radio" />
						<span class="label-text">Date Started</span>
					</label>
				</div>
				<div class="form-control">
					<label class="label">
						<input type="radio" name="sortBy" value="finished_on" class="radio mr-2" />
						<span class="label-text">Date Finished</span>
					</label>
				</div>
				<div class="form-control">
					<label class="label">
						<input type="radio" name="sortBy" value="created_at" class="radio" checked />
						<span class="label-text">Entry Added</span>
					</label>
				</div>
				<span class="divider" />
				<div class="form-control">
					<label class="label">
						<input type="radio" name="sortOrder" value="asc" class="radio" />
						<span class="label-text">Ascending</span>
					</label>
				</div>
				<div class="form-control">
					<label class="label">
						<input type="radio" name="sortOrder" value="desc" class="radio" checked />
						<span class="label-text">Descending</span>
					</label>
				</div>
				<div class="join mt-2">
					<button type="submit" disabled={!$isSortValid} class="btn w-full h-full join-item"
						>Apply</button
					>
					<button class="btn btn-error join-item" on:click={sortReset}>
						<X size={16} />
					</button>
				</div>
			</form>
		</div>
		<div class="dropdown dropdown-bottom dropdown-end">
			<div tabindex={0} role="button" class="btn">
				<Filter size={16} />
				<p>Filter</p>
			</div>
			<!-- svelte-ignore a11y-no-noninteractive-tabindex -->
			<div tabindex={0} class="dropdown-content bg-base-100 z-10 p-2 shadow rounded-box">
				<form method="post" use:filterForm>
					<Select
						items={statusOptions}
						bind:justValue={$filterFormData.status}
						bind:this={selectElement}
						placeholder="Statuses to filter..."
						closeListOnChange={false}
						multiple
					/>
					<div class="join mt-2 w-full">
						<button type="submit" disabled={!$isFilterValid} class="btn join-item w-full h-full"
							>Apply</button
						>
						<button
							class="btn btn-error join-item"
							on:click={() => {
								selectElement?.handleClear();
								filterReset();
								applySortAndFilter($filterFormData);
							}}
						>
							<X size={16} />
						</button>
					</div>
				</form>
			</div>
		</div>
	</div>
	<div class="grid grid-cols-3 gap-4">
		{#if $logsQueryResult.data && !$logsQueryResult.isLoading}
			{#each $logsQueryResult.data as log}
				<div class="card card-bordered">
					<div class="card-body">
						<h2 class="card-title line-clamp-2">{log.title}</h2>
						<GameStatusBadge status={log.statusId} />
						<p>{log.notes}</p>
					</div>
				</div>
			{/each}
		{/if}
	</div>
</main>
