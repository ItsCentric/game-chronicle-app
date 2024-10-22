import { getLogById } from '$lib/rust-bindings/database';
import { getGamesById } from '$lib/rust-bindings/igdb';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params }) => {
	const log = await getLogById(parseInt(params.id));
	const game = (await getGamesById([log.game_id]))[0];

	return {
		log,
		game
	};
};
