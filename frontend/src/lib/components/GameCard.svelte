<script lang="ts">
	import type { main } from '$lib/wailsjs/go/models';
	import { Gamepad2 } from 'lucide-svelte';

	export let data: main.IgdbGame;
</script>

<div
	class="w-full rounded-3xl overflow-hidden relative aspect-[3/4] group"
	on:click
	on:keydown
	role="button"
	tabindex="0"
>
	{#if data.cover.image_id}
		<img
			src={'https://images.igdb.com/igdb/image/upload/t_cover_big/' + data.cover.image_id + '.jpg'}
			alt="cover"
			class="h-full"
		/>
	{:else}
		<span
			class="h-full from-primary to-secondary bg-gradient-to-br flex justify-center items-center"
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
		class="absolute font-heading z-50 left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 opacity-0 group-hover:opacity-100 transition-opacity text-center font-semibold"
	>
		{data.name}
	</p>
</div>
