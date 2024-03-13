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

export const gameSearchSchema = z.object({
    gameTitle: z.string().min(1)
});
export type GameSearchFormSchema = typeof gameSearchSchema;

export const newLogSchema = z.object({
    rating: z
        .number()
        .max(5, { message: 'Rating must be 5 or less' })
        .nonnegative({ message: 'Rating must be positive' }),
    status: z.enum(['Playing', 'Completed', 'Abandoned']),
    notes: z.string().max(1000, { message: 'Notes must be less than 1000 characters' }).default(""),
    startedOn: z.date().max(new Date(), { message: 'Started on cannot be in the future' }),
    finishedOn: z.date().max(new Date(new Date().setDate(new Date().getDate() + 1)), {
        message: 'Finished on cannot be in the future'
    }),
    timePlayedHours: z.number({ invalid_type_error: 'Invalid value for hour' }).min(0).default('' as unknown as number),
    timePlayedMinutes: z.number({ invalid_type_error: 'Invalid value for minute' }).min(0).default('' as unknown as number)
});
export type NewLogFormSchema = typeof newLogSchema;

export const filterFormSchema = z.object({
    status: z.array(z.enum(statusOptions))
});
export type FilterFormSchema = typeof filterFormSchema;

export const settingsSchema = z.object({
    executablePaths: z.array(z.string()),
    processMonitoringEnabled: z.boolean(),
});
export type SettingsFormSchema = typeof settingsSchema;
