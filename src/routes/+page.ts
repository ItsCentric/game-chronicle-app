import { getDashboardStatistics, getLogs, getRecentLogs } from '$lib/rust-bindings/database';
import { getUserSettings } from '$lib/rust-bindings/helpers';
import { getGamesById } from '$lib/rust-bindings/igdb';
import { statusOptions } from '$lib/schemas';
import { redirect } from '@sveltejs/kit';
import { check } from '@tauri-apps/plugin-updater';
import { getCurrent } from '@tauri-apps/api/webview';
import { getAll } from '@tauri-apps/api/window';
import { checkedForDumpUpdate as checkedForDumpUpdateStore } from '$lib/stores';

export const load = async () => {
	if (typeof window === 'undefined') {
		return {
			username: '',
			dashboardStatistics: [
				{ total_games_played: 0, total_minutes_played: 0, total_games_completed: 0 },
				{ total_games_played: 0, total_minutes_played: 0, total_games_completed: 0 }
			],
			recentGames: [],
			similarGames: []
		};
	}
	let update: Awaited<ReturnType<typeof check>> = null;
	try {
		update = await check();
	} catch (error) {
		console.error(error);
	}
	if (update?.available) {
		await getCurrent().window.hide();
		const windows = getAll();
		await windows.find((window) => window.label === 'updater')?.show();
		return;
	}
	let checkedForDumpUpdate = false;
	const unsubscribe = checkedForDumpUpdateStore.subscribe((value) => {
		checkedForDumpUpdate = value;
	});
	if (!checkedForDumpUpdate) {
		throw redirect(301, '/dumps');
	}
	unsubscribe();
	const settings = await getUserSettings();
	if (settings.new) {
		throw redirect(301, '/onboarding');
	}
	const allButWishlistedOrBacklogged = statusOptions.filter(
		(status) => status != 'wishlist' && status != 'backlog'
	);
	const recentLogs = await getRecentLogs(6, allButWishlistedOrBacklogged);
	const logs = await getLogs('end_date', 'desc', allButWishlistedOrBacklogged);
	const gameIds = logs.map((log) => log.game_id);
	const games = await getGamesById(gameIds);
	const similarGameIds = games
		.filter((game) => (game.similar_games?.length ?? 0) > 0)
		.map((game) => game.similar_games as number[])
		.flat();
	const similarGames = await getGamesById(similarGameIds);
	const now = new Date();
	const endOfLastMonth = new Date(now.getFullYear(), now.getMonth(), 0);
	const startOfNextMonth = new Date(now.getFullYear(), now.getMonth() + 1, 1);
	const thisMonthStatistics = await getDashboardStatistics(endOfLastMonth, startOfNextMonth);
	const startOfLastMonth = new Date(endOfLastMonth.getFullYear(), endOfLastMonth.getMonth() - 1, 1);
	const lastMonthStatistics = await getDashboardStatistics(
		new Date(startOfLastMonth.getFullYear(), startOfLastMonth.getMonth(), 0),
		new Date(startOfNextMonth.getFullYear(), startOfNextMonth.getMonth() - 1, 1)
	);

	return {
		username: settings.username,
		dashboardStatistics: [lastMonthStatistics, thisMonthStatistics],
		recentGames: recentLogs,
		similarGames
	};
};
