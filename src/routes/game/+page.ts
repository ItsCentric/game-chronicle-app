import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import { z } from 'zod';

export const load: PageLoad = async ({ url }) => {
	const { getGameById } = await import('$lib/db/igdb');
	const { db } = await import('$lib/db/logs');
	const { logRowSchema } = await import('$lib/db/logs');
	if (typeof window === 'undefined') {
		return { game: { id: 0, title: '' } as Awaited<ReturnType<typeof getGameById>> };
	}
	type LogRow = z.infer<typeof logRowSchema>;
	const gameId = url.searchParams.get('id');
	if (!gameId) {
		return error(404, 'Game not found');
	}
	const game = await getGameById(parseInt(gameId));
	const [logStats] = await db.select<
		{ count: number; totalTimePlayed: number; lastPlayed: string; averageRating: number }[]
	>(
		`SELECT COUNT(*) AS count, SUM(minutes_played) AS totalTimePlayed, MAX(start_date) AS lastPlayed, AVG(rating) as averageRating FROM logs WHERE game_id = ?`,
		[game.id]
	);
	const logs = await db.select<LogRow[]>(
		`SELECT * FROM logs WHERE game_id = ? ORDER BY end_date DESC LIMIT 10`,
		[game.id]
	);

	return { game, logStats, logs };
};
