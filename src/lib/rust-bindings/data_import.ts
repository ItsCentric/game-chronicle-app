import { invoke } from '@tauri-apps/api/core';
import type { z } from 'zod';
import { logDataSchema } from './database';

export async function getSteamData(steamId: string, steamKey: string) {
	const steamData = await invoke('get_steam_data', { steamId, steamKey });
	return logDataSchema.parse(steamData);
}

export async function importIgdbGames(data: z.infer<typeof logDataSchema>[]) {
	return (await invoke('import_igdb_games', { data })) as number;
}
