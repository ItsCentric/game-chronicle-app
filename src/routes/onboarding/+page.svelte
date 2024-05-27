<script>
	import Button from '$lib/components/ui/button/button.svelte';
</script>

<main class="flex items-center justify-center w-full h-full container">
	<div class="text-center flex-1">
		<h1 class="font-heading text-3xl font-bold">Welcome to Game Chronicle</h1>
		<p class="text-lg mb-4">Coming from somewhere? Easily import your game data from anywhere.</p>
	</div>
	<div class="flex flex-col gap-4 flex-1">
		<Button class="w-3/4 mx-auto" size="lg" disabled>Steam</Button>
		<Button class="w-3/4 mx-auto" size="lg" disabled>Backloggd</Button>
		<Button class="w-3/4 mx-auto" size="lg" disabled>Tracker.gg</Button>
	</div>
</main>
