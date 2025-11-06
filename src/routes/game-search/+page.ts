import { type GameInfo } from '$lib/rust-bindings/igdb';
import { writable } from 'svelte/store';
import { z } from 'zod';

const cachedResponse = writable<GameInfo[] | null>(null);

export const load = async () => {
	if (typeof window === 'undefined') {
		return { popularGames: [] };
	}
	const { db, gameRowSchema } = await import('$lib/db/igdb');
	type GameRow = z.infer<typeof gameRowSchema>;
	let response: GameInfo[] | null = null;
	const unsubscribe = cachedResponse.subscribe((value) => (response = value));
	if (response) {
		unsubscribe();
		return { popularGames: response };
	}
	const popularGameIds = await db.select<{ gameId: string }[]>(
		`SELECT game_id as gameId FROM popularity_primitives ORDER BY value DESC LIMIT 72`
	);
	const gameIds = popularGameIds.map((g) => parseInt(g.gameId));
	const popularGames = await db.select<GameRow[]>(
		`SELECT g.*, c.image_id AS cover_image_id 
        FROM games g 
        LEFT JOIN covers c ON c.id = g.cover_id 
        LEFT JOIN game_platforms gp ON gp.game_id = g.id 
        LEFT JOIN platforms p ON p.id = gp.platform_id
        WHERE g.game_type IN (0, 4, 8, 9) 
        AND p.name NOT IN ('Android', 'iOS') 
        AND g.version_parent IS NULL 
        AND g.id IN (${gameIds.join(',')}) 
        GROUP BY g.id`
	);
	cachedResponse.set(popularGames);
	unsubscribe();
	return { popularGames };
};
