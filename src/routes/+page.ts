import { getDashboardStatistics, getLogs, getRecentLogs } from '$lib/rust-bindings/database';
import { getUserSettings } from '$lib/rust-bindings/helpers';
import { getGamesById } from '$lib/rust-bindings/igdb';
import { statusOptions } from '$lib/schemas';
import { redirect } from '@sveltejs/kit';
import { check } from '@tauri-apps/plugin-updater';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import { getAllWindows } from '@tauri-apps/api/window';
import { load as loadStore } from '@tauri-apps/plugin-store';

export const load = async () => {
	if (typeof window === 'undefined') {
		return {
			settings: {
				username: '',
				new: false,
				process_monitoring: { enabled: false, directoryDepth: 0 },
				executable_paths: '',
				autostart: false
			},
			dashboardStatistics: [
				{ total_games_played: 0, total_minutes_played: 0, total_games_completed: 0 },
				{ total_games_played: 0, total_minutes_played: 0, total_games_completed: 0 }
			],
			recentLogs: [],
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
		await getCurrentWebview().window.hide();
		const windows = await getAllWindows();
		await windows.find((window) => window.label === 'updater')?.show();
		return {
			settings: {
				username: '',
				new: false,
				process_monitoring: { enabled: false, directoryDepth: 0 },
				executable_paths: '',
				autostart: false
			},
			dashboardStatistics: [
				{ total_games_played: 0, total_minutes_played: 0, total_games_completed: 0 },
				{ total_games_played: 0, total_minutes_played: 0, total_games_completed: 0 }
			],
			recentLogs: [],
			similarGames: []
		};
	}
	let path: string;
	if (process.env.NODE_ENV === 'development') {
		path = 'dev/persistent.json';
	} else {
		path = 'persistent.json';
	}
	const store = await loadStore(path);
	const DAY_IN_MS = 24 * 60 * 60 * 1000;
	const lastDumpUpdate = (await store.get<number>('lastDumpUpdate')) ?? Date.now() - DAY_IN_MS;
	if (Date.now() - lastDumpUpdate >= DAY_IN_MS) {
		throw redirect(301, '/dumps');
	}
	const settings = await getUserSettings();
	if (settings.new) {
		throw redirect(301, '/onboarding');
	}
	const allButWishlistedOrBacklogged = statusOptions.filter(
		(status) => status != 'wishlist' && status != 'backlog'
	);
	const recentLogs = await getRecentLogs(3, allButWishlistedOrBacklogged);
	console.log('recentLogs', recentLogs);
	let gameIds = recentLogs.map((log) => log.game_id);
	let games = await getGamesById(gameIds);
	const gameAndRecentLogs = recentLogs.map((log) => {
		const game = games.find((game) => game.id === log.game_id);
		if (!game) throw new Error('Game not found');
		return { ...log, game };
	});
	const logs = await getLogs('end_date', 'desc', allButWishlistedOrBacklogged);
	gameIds = logs.map((log) => log.game_id);
	games = await getGamesById(gameIds);
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
	console.log('Loaded dashboard data');

	return {
		settings,
		dashboardStatistics: [lastMonthStatistics, thisMonthStatistics],
		recentLogs: gameAndRecentLogs,
		similarGames
	};
};
