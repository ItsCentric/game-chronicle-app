<script>
	import { Toaster, toast } from 'svelte-sonner';
	import '../app.css';
	import { goto } from '$app/navigation';
	import { QueryClient, QueryClientProvider } from '@sveltestack/svelte-query';
	import { listen } from '@tauri-apps/api/event';
	import { navigating } from '$app/stores';
	import { fade } from 'svelte/transition';
	import { tweened } from 'svelte/motion';
	import { cubicOut } from 'svelte/easing';

	const queryClient = new QueryClient();
	const loadProgress = tweened(0, { duration: 2500, easing: cubicOut });
	let showProgress = false;

	listen('game-stopped', (event) => {
		const data = event.payload;
		if (data.executable_name?.length > 0) {
			toast.info("Looks like you're playing a new title!", {
				description: 'Tell us what it is so we know for future reference.'
			});
			goto(
				`/game-search?executableName=${data.executable_name}&minutesPlayed=${data.minutes_played}`
			);
		} else {
			goto(`/logs/edit?gameId=${data.game_id}&minutesPlayed=${data.minutes_played}`);
		}
	});

	$: if ($navigating) {
		loadProgress.set(0.7, { duration: 2500 });
		showProgress = true;
		$navigating.complete.then(() => {
			loadProgress.set(1, { duration: 400 }).then(() => {
				showProgress = false;
				loadProgress.set(0, { duration: 0 });
			});
		});
	}
</script>

<main class="h-full relative">
	{#if showProgress}
		<div
			in:fade={{ duration: 0, delay: 500 }}
			out:fade={{ duration: 300 }}
			class="absolute top-0 z-50 w-full"
		>
			<span class="bg-accent float-left h-1" style={`width: ${$loadProgress * 100}%`} />
		</div>
	{/if}
	<Toaster />
	<QueryClientProvider client={queryClient}>
		<slot />
	</QueryClientProvider>
</main>
