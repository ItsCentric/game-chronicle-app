<script lang="ts">
	export let error: string | unknown;
</script>

<div
	class="absolute px-4 py-2 bg-red-800/80 shadow-lg rounded-xl text-center top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2"
>
	<p class="font-semibold font-heading text-lg">
		<slot />
	</p>
	<p>If the issue persists, try reaching out.</p>
	<p>{error}</p>
</div>
