import { getLocalTimeZone, today } from '@internationalized/date';
import { z } from 'zod';

export type SortFormSchema = typeof sortFormSchema;
export const sortFormSchema = z.object({
	sortBy: z
		.enum(['title', 'time_played_minutes', 'started_on', 'finished_on', 'created_at'])
		.default('created_at'),
	sortOrder: z.enum(['asc', 'desc']).default('desc')
});

export const statusOptions = [
	'wishlist',
	'backlog',
	'playing',
	'completed',
	'played',
	'abandoned',
	'retired'
] as const;
export type StatusOption = (typeof statusOptions)[number];

export const gameSearchSchema = z.object({
	gameTitle: z.string().min(1)
});
export type GameSearchFormSchema = typeof gameSearchSchema;

const timeZone = getLocalTimeZone();
const tomorrow = today(timeZone).add({ days: 1 }).toDate(timeZone);
export const logSchema = z.object({
	rating: z
		.number()
		.max(5, { message: 'Rating must be 5 or less' })
		.nonnegative({ message: 'Rating must be positive' }),
	logStartDate: z
		.date()
		.max(tomorrow, { message: 'New log start date must not be in the future' })
		.default(new Date()),
	logEndDate: z
		.date()
		.max(tomorrow, { message: 'New log end date must not be in the future' })
		.default(new Date()),
	status: z.enum(statusOptions),
	notes: z.string().max(1000, { message: 'Notes must be less than 1000 characters' }).optional(),
	timePlayedHours: z
		.number({ invalid_type_error: 'Invalid value for hour' })
		.min(0)
		.default('' as unknown as number),
	timePlayedMinutes: z
		.number({ invalid_type_error: 'Invalid value for minute' })
		.min(0)
		.default('' as unknown as number)
});
export type LogFormSchema = typeof logSchema;

export const filterFormSchema = z.object({
	status: z.array(z.enum(statusOptions))
});
export type FilterFormSchema = typeof filterFormSchema;

export const settingsSchema = z.object({
	username: z.string().min(1).max(50),
	executablePaths: z.array(z.string()),
	processMonitoringEnabled: z.boolean(),
	processMonitoringDirectoryDepth: z.number().min(0).max(99).default(3),
	autostart: z.boolean(),
	beta: z.boolean()
});
export type SettingsFormSchema = typeof settingsSchema;

export const steamImportFormSchema = z.object({
	steamKey: z.string().length(32, { message: 'Steam key must be 32 characters' }),
	steamId: z.string().min(1)
});

export type SteamImportFormSchema = typeof steamImportFormSchema;

export const gameDetectionSchema = settingsSchema.pick({
	processMonitoringEnabled: true,
	executablePaths: true
});
