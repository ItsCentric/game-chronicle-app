<script>
	import Button from '$lib/components/ui/button/button.svelte';
	import logo from '$lib/assets/128x128-logo.png';
</script>

<main class="flex flex-col items-center justify-center w-full h-full container">
	<img src={logo} alt="Game Chronicle Logo" class="w-24 h-24" />
	<div class="text-center">
		<h1 class="font-heading text-3xl font-bold">Welcome to Game Chronicle</h1>
		<p class="text-lg mb-4">Glad to have you here. Let's get you settled in, shall we?</p>
		<div class="flex gap-2 justify-center">
			<Button href="onboarding/import/steam">Next</Button>
		</div>
	</div>
</main>
