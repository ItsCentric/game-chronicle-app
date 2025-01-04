import type { z } from 'zod';
import type { LogFormSchema, StatusOption } from './schemas';
import type { GameInfo } from './rust-bindings/igdb';
import type { LogData } from './rust-bindings/database';
import { writeTextFile, BaseDirectory, exists, mkdir } from '@tauri-apps/plugin-fs';
import { getLocalTimeZone, now } from '@internationalized/date';

export function logDataFromForm(igdbGame: GameInfo, formData: z.infer<LogFormSchema>): LogData {
	return {
		status: formData.status.toLowerCase() as StatusOption,
		rating: formData.rating,
		start_date: formData.logStartDate.toISOString(),
		end_date: formData.logEndDate.toISOString(),
		notes: formData.notes ?? '',
		minutes_played: formData.timePlayedHours * 60 + formData.timePlayedMinutes,
		game_id: igdbGame.id
	};
}

export function toTitleCase(str: string) {
	return str.replace(/\b\w/g, (char) => char.toUpperCase());
}

export async function writeErrorLog(errorMessage: string) {
	const currentTime = now(getLocalTimeZone());
	const logsDirExists = await exists('', { baseDir: BaseDirectory.AppLog });
	if (!logsDirExists) await mkdir('', { baseDir: BaseDirectory.AppLog, recursive: true });
	await writeTextFile(
		'errors.log',
		`[${currentTime.hour}:${currentTime.minute}:${currentTime.second}] ${errorMessage}\n`,
		{ baseDir: BaseDirectory.AppLog, append: true }
	);
}
