<script>
	import '../app.css';
	import { Toaster } from 'svelte-french-toast';
	import { QueryClient, QueryClientProvider } from '@sveltestack/svelte-query';

	const queryClient = new QueryClient();
</script>

<main class="h-full">
	<QueryClientProvider client={queryClient}>
		<Toaster />
		<slot />
	</QueryClientProvider>
</main>
