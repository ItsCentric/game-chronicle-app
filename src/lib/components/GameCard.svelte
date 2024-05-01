<script lang="ts">
	import { Gamepad2 } from 'lucide-svelte';
	import type { IgdbGame } from '$lib/rust-bindings/igdb';

	export let data: IgdbGame;
</script>

<div
	class="w-full relative aspect-[3/4] group"
	data-testid="game-card"
	on:click
	on:keydown
	role="button"
	tabindex="0"
>
	{#if data.cover.image_id}
		<img
			src={'https://images.igdb.com/igdb/image/upload/t_cover_big/' + data.cover.image_id + '.jpg'}
			alt="cover"
			class="h-full rounded-3xl"
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
	<slot />
	<span
		class="opacity-0 z-10 group-hover:opacity-70 transition-opacity bg-black absolute inset-0 rounded-3xl"
	/>
	<p
		class="absolute font-heading z-20 left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 opacity-0 group-hover:opacity-100 transition-opacity text-center font-semibold"
	>
		{data.name}
	</p>
</div>
