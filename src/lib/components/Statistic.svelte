<script lang="ts">
	import { ArrowDown, ArrowUp, CircleHelp, Equal } from 'lucide-svelte';

	export let lastMonthStat: number;
	export let currentStat: number;
	export let timeStat = false;

	$: statChangePercentage = Math.round(((currentStat - lastMonthStat) / lastMonthStat) * 100);
</script>

<div>
	<p class="text-slate-400"><slot /></p>
	{#if timeStat}
		<p class="text-3xl font-heading font-bold leading-tight">{currentStat}h</p>
	{:else}
		<p class="text-3xl font-heading font-bold leading-tight">{currentStat}</p>
	{/if}
	{#if lastMonthStat === 0}
		<div class="text-sm text-yellow-600 flex gap-1 items-center">
			<CircleHelp size="1em" />
			<p>No data from last month</p>
		</div>
	{:else if statChangePercentage > 0}
		<div class="text-sm text-green-600 flex gap-1 items-center">
			<ArrowUp size="1em" />
			<p>{statChangePercentage}% more than last month</p>
		</div>
	{:else if statChangePercentage < 0}
		<div class="text-sm text-red-600 flex gap-1 items-center">
			<ArrowDown size="1em" />
			<p>{-statChangePercentage}% less than last month</p>
		</div>
	{:else}
		<div class="text-sm flex gap-1 items-center text-slate-400">
			<Equal size="1em" />
			<p>No change from last month</p>
		</div>
	{/if}
</div>
