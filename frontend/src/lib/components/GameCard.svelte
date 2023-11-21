<script lang="ts">
	import type { main } from '$lib/wailsjs/go/models';

	export let data: main.IgdbGame;
</script>

<div class="w-full max-w-xs relative aspect-[3/4] group">
	<img
		src={'https://images.igdb.com/igdb/image/upload/t_cover_big/' + data.cover.image_id + '.jpg'}
		alt="cover"
		class="h-full hover:opacity-30 group-hover:opacity-30 transition-opacity"
	/>
	<p
		class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 opacity-0 group-hover:opacity-100 transition-opacity text-center"
	>
		{data.name}
	</p>
</div>
