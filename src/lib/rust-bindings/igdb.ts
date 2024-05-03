import { z } from 'zod';
import { invoke } from '@tauri-apps/api/core';

const accessTokenResponseSchema = z.object({
	access_token: z.string(),
	expires_in: z.number(),
	token_type: z.string()
});

const coverSchema = z.object({
	id: z.number(),
	image_id: z.string()
});

const igdbGameSchema = z.object({
	id: z.number(),
	name: z.string(),
	cover: coverSchema
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
	return games.map((game: unknown) => igdbGameSchema.parse(game));
}
