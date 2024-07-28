import { z } from 'zod';
import { invoke } from '@tauri-apps/api/core';

export const gameInfoSchema = z.object({
	id: z.number(),
	title: z.string(),
	cover_image_id: z.string().optional().nullable(),
	websites: z.array(z.string()).optional().nullable(),
	similar_games: z.array(z.number()).optional().nullable(),
	category: z.number(),
	version_parent: z.number().optional().nullable(),
	total_rating: z.number().optional().nullable()
});

export type GameInfo = z.infer<typeof gameInfoSchema>;

export async function getGamesById(gameIds: number[]) {
	const games: object[] = await invoke('get_games_by_id', { gameIds });
	return games.map((game: unknown) => gameInfoSchema.parse(game));
}

export async function getPopularGames(amount: number) {
	const games: object[] = await invoke('get_popular_games', { amount });
	return games.map((game: unknown) => gameInfoSchema.parse(game));
}

export async function searchGame(query: string) {
	const games: object[] = await invoke('search_game', { searchQuery: query });
	return games.map((game: unknown) => gameInfoSchema.parse(game));
}
