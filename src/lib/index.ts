import type { z } from 'zod';
import type { LogFormSchema } from './schemas';
import type { IgdbGame } from './rust-bindings/igdb';
import type { LogData } from './rust-bindings/database';

export function logDataFromForm(igdbGame: IgdbGame, formData: z.infer<LogFormSchema>): LogData {
	return {
		title: igdbGame.name,
		status: formData.status.toLowerCase(),
		rating: formData.rating,
		date: formData.logDate.toISOString(),
		notes: formData.notes ?? '',
		minutes_played: formData.timePlayedHours * 60 + formData.timePlayedMinutes,
		igdb_id: igdbGame.id
	};
}
