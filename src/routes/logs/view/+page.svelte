<script lang="ts">
	import { Button } from '$lib/components/ui/form';
	import { Calendar, Clock } from 'lucide-svelte';
	import type { PageData } from './$types';
	import { toTitleCase } from '$lib';
	import Separator from '$lib/components/ui/separator/separator.svelte';
	import { cn } from '$lib/utils';
	import { parseAbsoluteToLocal, toCalendarDate } from '@internationalized/date';

	export let data: PageData;
	const { game, log } = data;
	const parsedStartDate = toCalendarDate(parseAbsoluteToLocal(log.start_date));
	const parsedEndDate = toCalendarDate(parseAbsoluteToLocal(log.end_date));
</script>

<main class="min-h-full container py-8 px-16 relative">
	<div class="mb-4">
		<h1 class="text-3xl font-heading font-bold">Log Entry</h1>
		<p class="text-gray-500 text-lg font-heading">
			Here's your entry for {game.title}
		</p>
	</div>
	<div class="grid grid-cols-[25%_75%] gap-4">
		<div>
			<img
				src={'https://images.igdb.com/igdb/image/upload/t_cover_big/' +
					game.cover_image_id +
					'.jpg'}
				alt="cover"
				class="aspect-[3/4] rounded-3xl mb-4 w-full"
			/>
		</div>
		<div class="flex flex-col gap-2">
			<div>
				<div class="text-2xl flex items-center gap-4 font-heading">
					<p>{data.game.title}</p>
					<Separator orientation="vertical" decorative class="h-10" />
					<p
						class={cn(
							'text-base font-semibold font-sans decoration-2 underline-offset-4 underline rounded-xl',
							{
								'decoration-green-500':
									log.status === 'played' || log.status === 'playing' || log.status === 'completed',
								'decoration-gray-500': log.status === 'backlog',
								'decoration-blue-500': log.status === 'wishlist',
								'decoration-red-500': log.status === 'abandoned',
								'decoration-yellow-500': log.status === 'retired'
							}
						)}
					>
						{toTitleCase(log.status)}
					</p>
				</div>
				<div class="flex gap-1">
					{#each Array(5) as _, i}
						{#if log.rating >= i + 1}
							<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 26 26">
								<path
									fill="#eab308"
									d="M25.326 10.137a1.001 1.001 0 0 0-.807-.68l-7.34-1.066l-3.283-6.651c-.337-.683-1.456-.683-1.793 0L8.82 8.391L1.48 9.457a1 1 0 0 0-.554 1.705l5.312 5.178l-1.254 7.31a1.001 1.001 0 0 0 1.451 1.054L13 21.252l6.564 3.451a1 1 0 0 0 1.451-1.054l-1.254-7.31l5.312-5.178a.998.998 0 0 0 .253-1.024z"
								/>
							</svg>
						{:else}
							<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 26 26">
								<path
									fill="#454545"
									d="M25.326 10.137a1.001 1.001 0 0 0-.807-.68l-7.34-1.066l-3.283-6.651c-.337-.683-1.456-.683-1.793 0L8.82 8.391L1.48 9.457a1 1 0 0 0-.554 1.705l5.312 5.178l-1.254 7.31a1.001 1.001 0 0 0 1.451 1.054L13 21.252l6.564 3.451a1 1 0 0 0 1.451-1.054l-1.254-7.31l5.312-5.178a.998.998 0 0 0 .253-1.024z"
								/>
							</svg>
						{/if}
					{/each}
				</div>
			</div>
			<div class="flex gap-2">
				<Calendar />
				{#if parsedStartDate.compare(parsedEndDate) === 0}
					<p>
						{new Date(log.start_date).toLocaleDateString('en-US', {
							month: 'long',
							day: 'numeric',
							year: 'numeric'
						})}
					</p>
				{:else}
					<p>
						{new Date(log.start_date).toLocaleDateString('en-US', {
							month: 'long',
							day: 'numeric',
							year: 'numeric'
						})} - {new Date(log.end_date).toLocaleDateString('en-US', {
							month: 'long',
							day: 'numeric',
							year: 'numeric'
						})}
					</p>
				{/if}
			</div>
			<div>
				<div class="mb-2 flex gap-2 pointer-events-none">
					<Clock />
					<p>
						{Math.floor(log.minutes_played / 60)}h {log.minutes_played % 60}m
					</p>
				</div>
			</div>
			<div>
				<p>{log.notes}</p>
			</div>
		</div>
	</div>
	<div class="float-right">
		<Button variant="secondary" on:click={() => window.history.back()}>Back</Button>
		<Button href={`/logs/edit?id=${log.id}`}>Edit</Button>
	</div>
</main>
