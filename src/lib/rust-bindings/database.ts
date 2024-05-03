import type { StatusOption } from '$lib/schemas';
import { invoke } from '@tauri-apps/api/core';
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

const executableDetailsSchema = z.object({
	name: z.string(),
	igdb_id: z.number(),
	minutes_played: z.number()
});

export type Log = z.infer<typeof logSchema>;
export type LogData = Omit<Log, 'id' | 'created_at' | 'updated_at'>;
export type ExecutableDetails = z.infer<typeof executableDetailsSchema>;

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

export async function deleteLog(id: number) {
	const deletedLogId = await invoke('delete_log', { id });
	return deletedLogId as number;
}

export async function getLogById(id: number) {
	const log = await invoke('get_log_by_id', { id });
	return logSchema.parse(log);
}

export async function updateLog(log: Omit<Log, 'created_at' | 'updated_at'>) {
	const updatedLogId = await invoke('update_log', { log });
	return updatedLogId as number;
}

export async function addExecutableDetails(executableDetails: ExecutableDetails) {
	const addedExecutableDetailsId = await invoke('add_executable_details', { executableDetails });
	return addedExecutableDetailsId as number;
}

export async function addLog(log: LogData) {
	const addedLogId = await invoke('add_log', { log });
	return addedLogId as number;
}
