import { statusOptions } from '$lib/schemas';
import { appDataDir } from '@tauri-apps/api/path';
import Database from '@tauri-apps/plugin-sql';
import { z } from 'zod';

const dataDir = await appDataDir();
export const db = await Database.load(`sqlite:${dataDir}/logs.db`);

export const logRowSchema = z.object({
	id: z.number(),
	gameId: z.number(),
	createdAt: z.string(),
	updatedAt: z.string().optional(),
	startDate: z.string().transform((val) => new Date(val)),
	endDate: z.string().transform((val) => new Date(val)),
	rating: z.number(),
	notes: z.string().optional().nullable(),
	status: z.enum(statusOptions),
	minutesPlayed: z.number()
});
export type LogRow = z.infer<typeof logRowSchema>;
