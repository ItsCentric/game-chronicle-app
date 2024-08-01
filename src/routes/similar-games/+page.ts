import { getLogs } from '$lib/rust-bindings/database';
import { getGamesById } from '$lib/rust-bindings/igdb';
import { statusOptions } from '$lib/schemas';

export const load = async () => {
	if (typeof window === 'undefined') {
		return { similarGames: [] };
	}
	const logs = await getLogs(
		'end_date',
		'desc',
		statusOptions.filter((status) => status != 'wishlist' && status != 'backlog')
	);
	const gameIds = logs.map((log) => log.game_id);
	const games = await getGamesById(gameIds);
	const similarGameIds = games
		.filter((game) => game.similar_games)
		.map((game) => game.similar_games as number[])
		.flat();
	const similarGames = await getGamesById(similarGameIds);
	return {
		similarGames
	};
};
