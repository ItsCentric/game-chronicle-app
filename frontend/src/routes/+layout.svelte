<script>
	import '../app.css';
	import { Toaster } from 'svelte-french-toast';
</script>

<main class="h-full">
	<Toaster />
	<slot />
</main>
