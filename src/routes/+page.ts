import { goto } from '$app/navigation';
import {
	getCurrentUsername,
	getDashboardStatistics,
	getLogs,
	getRecentLogs
} from '$lib/rust-bindings/database';
import { authenticateWithTwitch, getGamesById, getSimilarGames } from '$lib/rust-bindings/igdb';
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
		const accessTokenResponse = await authenticateWithTwitch();
		const recentLogs = await getRecentLogs(
			6,
			statusOptions.filter((status) => status != 'Wishlist')
		);
		const recentGameIds = recentLogs.map((log) => log.igdb_id);
		const games = await getGamesById(accessTokenResponse.access_token, recentGameIds);
		const sortedGames = [];
		for (let i = 0; i < recentGameIds.length; i++) {
			const game = games.find((game) => game.id === recentGameIds[i]);
			if (game) {
				sortedGames.push(game);
			}
		}
		const logs = await getLogs('date', 'desc', statusOptions as unknown as StatusOption[]);
		const gameIds = logs.map((log) => log.igdb_id);
		const similarGames = await getSimilarGames(accessTokenResponse.access_token, gameIds);

		return {
			username: await getCurrentUsername(),
			dashboardStatistics: await getDashboardStatistics(),
			recentGames: sortedGames,
			similarGames
		};
	} catch (error) {
		if (error === 'Error: Twitch client ID not found') {
			goto('/settings');
		}
	}
};
