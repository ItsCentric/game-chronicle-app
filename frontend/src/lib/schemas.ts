import { z } from "zod";

export type SortFormSchema = typeof sortFormSchema;
export const sortFormSchema = z.object({
    sortBy: z.enum(['title', 'time_played_minutes', 'started_on', 'finished_on', 'created_at']).default('created_at'),
    sortOrder: z.enum(['asc', 'desc']).default('desc')
});

export const statusOptions = [
    'Wishlist',
    'Backlog',
    'Playing',
    'Completed',
    'Played',
    'Abandoned',
    'Retired'
] as const;
export type StatusOption = typeof statusOptions[number];

export const gameSearchSchema = z.object({
    gameTitle: z.string().min(1)
});
export type GameSearchFormSchema = typeof gameSearchSchema;

const tomorrow = new Date();
tomorrow.setDate(tomorrow.getDate() + 1);
export const logSchema = z.object({
    rating: z
        .number()
        .max(5, { message: 'Rating must be 5 or less' })
        .nonnegative({ message: 'Rating must be positive' }),
    logDate: z.date().max(tomorrow, { message: 'New log date must not be in the future' }).default(new Date()),
    status: z.enum(statusOptions),
    notes: z.string().max(1000, { message: 'Notes must be less than 1000 characters' }).optional(),
    finished: z.boolean().default(false),
    timePlayedHours: z.number({ invalid_type_error: 'Invalid value for hour' }).min(0).default('' as unknown as number),
    timePlayedMinutes: z.number({ invalid_type_error: 'Invalid value for minute' }).min(0).default('' as unknown as number)
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
});
export type SettingsFormSchema = typeof settingsSchema;
