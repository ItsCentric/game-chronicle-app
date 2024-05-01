import type { StatusOption } from '$lib/schemas';
import { invoke } from '@tauri-apps/api/tauri';
import { z } from 'zod';

const dashboardStatisticsSchema = z.object({
	total_minutes_played: z.number(),
	total_games_played: z.number(),
	total_games_completed: z.number()
});

const dashboardStatisticsResponseSchema = z.tuple([
	dashboardStatisticsSchema,
	dashboardStatisticsSchema
]);

const logSchema = z.object({
	id: z.number(),
	created_at: z.string(),
	updated_at: z.string(),
	title: z.string(),
	date: z.string(),
	rating: z.number(),
	notes: z.string(),
	status: z.string(),
	completed: z.boolean(),
	minutes_played: z.number(),
	igdb_id: z.number()
});

export async function getCurrentUsername() {
	const username = await invoke('get_current_username');
	return username as string;
}

export async function getDashboardStatistics() {
	const statistics = await invoke('get_dashboard_statistics');
	return dashboardStatisticsResponseSchema.parse(statistics);
}

export async function getRecentLogs(amount: number, filter: StatusOption[]) {
	const logs: object[] = await invoke('get_recent_logs', { amount, filter });
	console.log(logs);
	if (logs.length === 0) {
		return [];
	}
	return logs.map((log: unknown) => logSchema.parse(log));
}

export async function getLogs(sortBy: string, sortOrder: 'asc' | 'desc', filter: StatusOption[]) {
	console.log('getting logs...');
	const logs: object[] = await invoke('get_logs', {
		sortBy,
		sortOrder,
		filter: filter.map((option) => option.toLowerCase())
	});
	console.log('get logs called');
	return logs.map((log: unknown) => logSchema.parse(log));
}
