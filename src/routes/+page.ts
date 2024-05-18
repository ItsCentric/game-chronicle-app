import { goto } from '$app/navigation';
import { getDashboardStatistics, getLogs, getRecentLogs } from '$lib/rust-bindings/database';
import { getCurrentUsername } from '$lib/rust-bindings/helpers';
import { authenticateWithTwitch, getSimilarGames } from '$lib/rust-bindings/igdb';
import { statusOptions, type StatusOption } from '$lib/schemas';

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
		const recentLogs = await getRecentLogs(
			6,
			statusOptions.filter((status) => status != 'wishlist')
		);
		const logs = await getLogs('date', 'desc', statusOptions as unknown as StatusOption[]);
		const accessTokenResponse = await authenticateWithTwitch();
		const gameIds = logs.map((log) => log.game.id);
		const similarGames = await getSimilarGames(accessTokenResponse.access_token, gameIds);

		return {
			username: await getCurrentUsername(),
			dashboardStatistics: await getDashboardStatistics(),
			recentGames: recentLogs,
			similarGames
		};
	} catch (error) {
		if (error === 'Error: Twitch client ID not found') {
			goto('/settings');
		}
	}
};
