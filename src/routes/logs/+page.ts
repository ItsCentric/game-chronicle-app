import { getLogs } from '$lib/rust-bindings/database';
import { getGamesById } from '$lib/rust-bindings/igdb';

export const load = async () => {
	if (typeof window === 'undefined') {
		return { logsAndGames: [] };
	}
	const logs = await getLogs('end_date', 'desc', []);
	const games = await getGamesById(logs.map((log) => log.game_id));
	const logsAndGames = logs.map((log) => {
		const associatedGame = games.find((game) => game.id === log.game_id);
		if (!associatedGame) {
			throw new Error(`Could not find game with id ${log.game_id}`);
		}
		return { ...log, game: associatedGame };
	});

	return {
		logsAndGames
	};
};
