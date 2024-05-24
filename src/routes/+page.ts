import { goto } from '$app/navigation';
import { getDashboardStatistics, getLogs, getRecentLogs } from '$lib/rust-bindings/database';
import { getUserSettings } from '$lib/rust-bindings/helpers';
import { authenticateWithTwitch, getSimilarGames } from '$lib/rust-bindings/igdb';
import { statusOptions } from '$lib/schemas';

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
	try {
		const allButWishlistedOrBacklogged = statusOptions.filter(
			(status) => status != 'wishlist' && status != 'backlog'
		);
		const recentLogs = await getRecentLogs(6, allButWishlistedOrBacklogged);
		const logs = await getLogs('date', 'desc', allButWishlistedOrBacklogged);
		const accessTokenResponse = await authenticateWithTwitch();
		const gameIds = logs.map((log) => log.game.id);
		const similarGames = await getSimilarGames(accessTokenResponse.access_token, gameIds);
		const now = new Date();
		const endOfLastMonth = new Date(now.getFullYear(), now.getMonth(), 0);
		const startOfNextMonth = new Date(now.getFullYear(), now.getMonth() + 1, 1);
		const thisMonthStatistics = await getDashboardStatistics(endOfLastMonth, startOfNextMonth);
		const startOfLastMonth = new Date(
			endOfLastMonth.getFullYear(),
			endOfLastMonth.getMonth() - 1,
			1
		);
		const lastMonthStatistics = await getDashboardStatistics(
			new Date(startOfLastMonth.getFullYear(), startOfLastMonth.getMonth(), 0),
			new Date(startOfNextMonth.getFullYear(), startOfNextMonth.getMonth() - 1, 1)
		);
		const settings = await getUserSettings();

		return {
			username: settings.username,
			dashboardStatistics: [lastMonthStatistics, thisMonthStatistics],
			recentGames: recentLogs,
			similarGames
		};
	} catch (error) {
		if (error === 'Error: Twitch client ID not found') {
			goto('/settings');
		}
	}
};
