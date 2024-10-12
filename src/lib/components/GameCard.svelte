<script lang="ts">
	import { Gamepad2 } from 'lucide-svelte';
	import StarIcon from '$lib/components/icons/StarIcon.svelte';
	import { cn } from '$lib/utils';
	import { type StatusOption } from '$lib/schemas';
	import * as Tooltip from '$lib/components/ui/tooltip';

	export let title: string;
	export let cover: string | undefined | null;
	export let rating: number | undefined | null;
	export let status: StatusOption | undefined = undefined;

	const logStatusColorMap: Record<StatusOption, string> = {
		backlog: 'bg-gray-500',
		wishlist: 'bg-blue-500',
		playing: 'bg-green-500',
		played: 'bg-green-500',
		completed: 'bg-green-500',
		abandoned: 'bg-red-500',
		retired: 'bg-yellow-500'
	};
	function logStatusColor(status: StatusOption) {
		return logStatusColorMap[status];
	}
</script>

<div
	{...$$restProps}
	class={cn('px-4 py-2 border relative rounded-lg group flex gap-4', $$restProps['class'])}
	data-testid="game-card"
>
	<div class="relative aspect-[3/4] max-w-[10rem] flex-1 h-full group">
		{#if cover}
			<img
				src={'https://images.igdb.com/igdb/image/upload/t_cover_big/' + cover + '.jpg'}
				alt="cover"
				class="rounded-lg"
			/>
		{:else}
			<span
				class="h-full from-primary rounded-3xl to-secondary bg-gradient-to-br flex justify-center items-center"
			>
				<p class="select-none text-muted">
					<Gamepad2 size={64} />
				</p>
			</span>
		{/if}
	</div>
	<div class="flex-1">
		<div class="mb-4">
			<div class="flex gap-2 items-center">
				<h3 class="text-lg font-semibold line-clamp-2 md:line-clamp-none">{title}</h3>
				{#if status}
					<Tooltip.Root disableHoverableContent>
						<Tooltip.Trigger>
							<div class={`rounded-full h-2 w-2 ${logStatusColor(status)}`}></div>
						</Tooltip.Trigger>
						<Tooltip.Content class="capitalize">{status}</Tooltip.Content>
					</Tooltip.Root>
				{/if}
			</div>
			<slot name="sub-title" class="text-sm text-muted-foreground" />
		</div>
		<div class="text-foreground mb-4 flex gap-1 items-center">
			<StarIcon />
			<p>{rating === null || rating == undefined ? 'N/A' : rating.toFixed(1)}</p>
		</div>
		<slot name="description" />
	</div>
	<div
		class="opacity-0 -bottom-2 group-hover:opacity-100 duration-200 transition-opacity flex gap-1 -right-2 bg-background z-50 absolute border rounded-md px-2 py-1"
	>
		<slot name="actions" />
	</div>
</div>
