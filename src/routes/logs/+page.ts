import { getLogs } from '$lib/rust-bindings/database';
import { authenticateWithTwitch, getGamesById } from '$lib/rust-bindings/igdb';
import type { StatusOption } from '$lib/schemas';

export const load = async () => {
	const logs = await getLogs('date', 'desc', []);
	const accessTokenResponse = await authenticateWithTwitch();
	const gameIds = logs.map((log) => log.igdb_id);
	const games = await getGamesById(accessTokenResponse.access_token, gameIds);
	return {
		logs: logs.map((log) => {
			const game = games.find((game) => game.id === log.igdb_id);
			if (!game) {
				throw new Error('Game not found');
			}
			return { ...log, status: log.status as StatusOption, game };
		})
	};
};
