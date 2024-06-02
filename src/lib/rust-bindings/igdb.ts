import { z } from 'zod';
import { invoke } from '@tauri-apps/api/core';

const accessTokenResponseSchema = z.object({
	access_token: z.string(),
	expires_in: z.number(),
	token_type: z.string()
});

const coverSchema = z.object({
	id: z.number(),
	cover_id: z.string()
});

export const igdbGameSchema = z.object({
	id: z.number(),
	title: z.string(),
	cover: coverSchema.optional().nullable()
});

const similarGamesSchema = z.object({
	similar_games: z.array(igdbGameSchema)
});

export type IgdbGame = z.infer<typeof igdbGameSchema>;

export async function authenticateWithTwitch() {
	const response = await invoke('authenticate_with_twitch');
	return accessTokenResponseSchema.parse(response);
}

export async function getGamesById(accessToken: string, gameIds: number[]) {
	const games: object[] = await invoke('get_games_by_id', { accessToken, gameIds });
	return games.map((game: unknown) => igdbGameSchema.parse(game));
}

export async function getSimilarGames(accessToken: string, gameIds: number[]) {
	const games: object[] = await invoke('get_similar_games', { accessToken, gameIds });
	const similarGames: IgdbGame[] = [];
	for (const game of games) {
		const similarGameRes = similarGamesSchema.parse(game);
		similarGames.push(...similarGameRes.similar_games);
	}
	return similarGames;
}

export async function getRandomTopGames(accessToken: string, amount: number) {
	const games: object[] = await invoke('get_random_top_games', { accessToken, amount });
	return games.map((game: unknown) => igdbGameSchema.parse(game));
}

export async function searchGame(accessToken: string, query: string) {
	const games: object[] = await invoke('search_game', { accessToken, searchQuery: query });
	return games.map((game: unknown) => igdbGameSchema.parse(game));
}
