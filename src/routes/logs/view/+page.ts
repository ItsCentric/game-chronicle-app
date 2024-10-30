import { getLogById } from '$lib/rust-bindings/database';
import { getGamesById } from '$lib/rust-bindings/igdb';
import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ url }) => {
	if (typeof window === 'undefined') {
		return {
			log: {
				id: 0,
				created_at: '',
				updated_at: '',
				start_date: '',
				end_date: '',
				rating: 0,
				notes: '',
				status: 'playing',
				minutes_played: 0,
				game_id: 0
			},
			game: {
				id: 0,
				title: '',
				category: '',
				cover_image_id: '',
				websites: [],
				similar_games: [],
				version_parent: 0,
				total_rating: 0
			}
		};
	}
	const idStr = url.searchParams.get('id');
	if (!idStr) {
		throw error(400, 'Missing id parameter');
	}
	const id = parseInt(idStr);
	const log = await getLogById(id);
	const game = (await getGamesById([log.game_id]))[0];

	return {
		log,
		game
	};
};
