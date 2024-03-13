<script>
	import { Toaster } from 'svelte-sonner';
import '../app.css';
	import { QueryClient, QueryClientProvider } from '@sveltestack/svelte-query';

	const queryClient = new QueryClient();
</script>

<main class="h-full">
    <Toaster />
	<QueryClientProvider client={queryClient}>
		<slot />
	</QueryClientProvider>
</main>
