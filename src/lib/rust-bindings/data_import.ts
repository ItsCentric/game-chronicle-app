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
		const { cover, title, ...restOfIgdbData } = igdbData;
		const newIgdbData = {
			...restOfIgdbData,
			cover: { image_id: cover.cover_id, id: cover.id },
			name: title
		};
		return [logData, newIgdbData];
	});
	return (await invoke('import_igdb_games', { data: newData })) as number;
}
