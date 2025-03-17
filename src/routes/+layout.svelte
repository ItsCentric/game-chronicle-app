<script lang="ts">
	import { Toaster, toast } from 'svelte-sonner';
	import '../app.css';
	import { goto } from '$app/navigation';
	import { QueryClient, QueryClientProvider } from '@sveltestack/svelte-query';
	import { listen } from '@tauri-apps/api/event';
	import { navigating } from '$app/stores';
	import { fade } from 'svelte/transition';
	import { tweened } from 'svelte/motion';
	import { cubicOut } from 'svelte/easing';
	import { getRecentLogs, type Log, updateLog } from '$lib/rust-bindings/database';
	import { statusOptions } from '$lib/schemas';
	import * as AlertDialog from '$lib/components/ui/alert-dialog';

	const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
	const loadProgress = tweened(0, { duration: 2500, easing: cubicOut });
	let showProgress = false;
	let updatedLog: Log;

	listen('game-stopped', async (event) => {
		const data = event.payload;
		if (data.executable_name?.length > 0) {
			toast.info("Looks like you're playing a new title!", {
				description: 'Tell us what it is so we know for future reference.'
			});
			goto(
				`/game-search?executableName=${data.executable_name}&minutesPlayed=${data.minutes_played}&startTime=${data.start_time}`
			);
		} else {
			const recentLogs = await getRecentLogs(1, statusOptions);
			const previousGame = recentLogs[0]?.game_id;
			if (previousGame === data.game_id) {
				open = true;
				updatedLog = recentLogs[0];
				updatedLog.minutes_played += data.minutes_played;
				return;
			}
			goto(
				`/logs/edit?gameId=${data.game_id}&minutesPlayed=${data.minutes_played}&startTime=${data.start_time}`
			);
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
	let open = false;
</script>

<main
	class="h-full relative"
	on:contextmenu={(e) => {
		if (process.env.NODE_ENV === 'production') e.preventDefault();
	}}
>
	{#if showProgress}
		<div
			in:fade={{ duration: 0, delay: 500 }}
			out:fade={{ duration: 300 }}
			class="absolute top-0 z-50 w-full"
		>
			<span class="bg-accent float-left h-1" style={`width: ${$loadProgress * 100}%`} />
		</div>
	{/if}
	<AlertDialog.Root {open} onOpenChange={(newVal) => (open = newVal)}>
		<AlertDialog.Content>
			<AlertDialog.Header>
				<AlertDialog.Title>You just closed the same game</AlertDialog.Title>
				<AlertDialog.Description
					>Crashed last time? Needed to restart? It happens. We can add your playtime to your
					previous log if that's the case.</AlertDialog.Description
				>
			</AlertDialog.Header>
			<AlertDialog.Footer>
				<AlertDialog.Cancel>Don't add playtime</AlertDialog.Cancel>
				<AlertDialog.Action
					on:click={async () =>
						toast.promise(updateLog(updatedLog), {
							loading: 'Updating log...',
							success: 'Log updated successfully!',
							error: 'Failed to update log'
						})}>Yes, add playtime</AlertDialog.Action
				>
			</AlertDialog.Footer>
		</AlertDialog.Content>
	</AlertDialog.Root>
	<Toaster />
	<QueryClientProvider client={queryClient}>
		<slot />
	</QueryClientProvider>
</main>
