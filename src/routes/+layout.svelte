<script>
	import { Toaster, toast } from 'svelte-sonner';
	import '../app.css';
	import { goto } from '$app/navigation';
	import { QueryClient, QueryClientProvider } from '@sveltestack/svelte-query';
	import { listen } from '@tauri-apps/api/event';

	const queryClient = new QueryClient();

	listen('game-stopped', (event) => {
		const data = event.payload;
		if (data.executableName !== '') {
			toast.info("Looks like you're playing a new title!", {
				description: 'Tell us what it is so we know for future reference.'
			});
			goto(
				`/game-search?executableName=${data.executableName}&minutesPlayed=${data.minutesPlayed}`
			);
		} else {
			goto(`/logs/edit?gameId=${data.gameId}&minutesPlayed=${data.minutesPlayed}`);
		}
	});
</script>

<main class="h-full">
	<Toaster />
	<QueryClientProvider client={queryClient}>
		<slot />
	</QueryClientProvider>
</main>
