import type { z } from 'zod';
import type { LogFormSchema, StatusOption } from './schemas';
import type { GameInfo } from './rust-bindings/igdb';
import type { LogData } from './rust-bindings/database';

export function logDataFromForm(igdbGame: GameInfo, formData: z.infer<LogFormSchema>): LogData {
	return {
		status: formData.status.toLowerCase() as StatusOption,
		rating: formData.rating,
		date: formData.logDate.toISOString(),
		notes: formData.notes ?? '',
		minutes_played: formData.timePlayedHours * 60 + formData.timePlayedMinutes,
		game_id: igdbGame.id
	};
}

export function toTitleCase(str: string) {
	return str.replace(/\b\w/g, (char) => char.toUpperCase());
}
