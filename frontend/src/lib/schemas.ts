import { z } from "zod";

export type SortFormData = z.infer<typeof sortFormSchema>;
export const sortFormSchema = z.object({
    sortBy: z.enum(['title', 'time_played_minutes', 'started_on', 'finished_on', 'created_at']),
    sortOrder: z.enum(['asc', 'desc'])
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
export type FilterFormData = z.infer<typeof filterFormSchema>;
export const filterFormSchema = z.object({
    status: z.array(z.enum(statusOptions))
});
