import { invoke } from '@tauri-apps/api/core';
import { z } from 'zod';
import { igdbGameSchema } from './igdb';
import { logDataSchema } from './database';

const logAndIgdbDataSchema = z.array(z.tuple([logDataSchema, igdbGameSchema]));

export async function getSteamData(steamId: string) {
	const steamData = await invoke('get_steam_data', { steamId });
	return logAndIgdbDataSchema.parse(steamData);
}

export async function importIgdbGames(data: z.infer<typeof logAndIgdbDataSchema>) {
	const newData = data.map(([logData, igdbData]) => {
		if (!igdbData.cover) {
			return [logData, igdbData];
		}
		const newObject = { ...igdbData } as Record<string, unknown>;
		// @ts-expect-error - we know that cover is defined
		newObject['cover']['image_id'] = newObject.cover.cover_id;
		// @ts-expect-error - we know that cover is defined
		delete newObject.cover.cover_id;
		newObject['name'] = newObject.title;
		delete newObject.title;
		return [logData, newObject];
	});
	await invoke('import_igdb_games', { data: newData });
}
