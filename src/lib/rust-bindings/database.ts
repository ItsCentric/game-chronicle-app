import { statusOptions, type StatusOption } from '$lib/schemas';
import { invoke } from '@tauri-apps/api/core';
import { z } from 'zod';

const dashboardStatisticsSchema = z.object({
	total_minutes_played: z.number(),
	total_games_played: z.number(),
	total_games_completed: z.number()
});

const gameSchema = z.object({
	id: z.number(),
	title: z.string(),
	cover_id: z.string().optional()
});

const logSchema = z.object({
	id: z.number(),
	created_at: z.string(),
	updated_at: z.string(),
	date: z.string(),
	rating: z.number(),
	notes: z.string(),
	status: z.enum([...statusOptions]),
	minutes_played: z.number(),
	game: gameSchema
});

const logDataSchema = logSchema.omit({ id: true, created_at: true, updated_at: true });

const logUpdateSchema = logSchema.omit({ created_at: true, updated_at: true, game: true });

const executableDetailsSchema = z.object({
	name: z.string(),
	game_id: z.number(),
	minutes_played: z.number()
});

export type Log = z.infer<typeof logSchema>;
export type LogData = z.infer<typeof logDataSchema>;
export type ExecutableDetails = z.infer<typeof executableDetailsSchema>;

export async function getCurrentUsername() {
	const username = await invoke('get_current_username');
	return username as string;
}

export async function getDashboardStatistics(startDate: Date, endDate: Date) {
	const statistics = await invoke('get_dashboard_statistics', {
		startDate: startDate.toISOString(),
		endDate: endDate.toISOString()
	});
	return dashboardStatisticsSchema.parse(statistics);
}

export async function getRecentLogs(amount: number, filter: StatusOption[]) {
	const logs: object[] = await invoke('get_recent_logs', { amount, filter });
	if (logs.length === 0) {
		return [];
	}
	return logs.map((log: unknown) => logSchema.parse(log));
}

export async function getLogs(sortBy: string, sortOrder: 'asc' | 'desc', filter: StatusOption[]) {
	if (filter.length === 0) {
		filter = [...statusOptions];
	}
	const logs: object[] = await invoke('get_logs', {
		sortBy,
		sortOrder,
		filter: filter.map((option) => option.toLowerCase())
	});
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

export async function updateLog(log: z.infer<typeof logUpdateSchema>) {
	const updatedLogId = await invoke('update_log', { logData: log });
	return updatedLogId as number;
}

export async function addExecutableDetails(executableDetails: ExecutableDetails) {
	const addedExecutableDetailsId = await invoke('add_executable_details', { executableDetails });
	return addedExecutableDetailsId as number;
}

export async function addLog(log: LogData) {
	const addedLogId = await invoke('add_log', { logData: log });
	return addedLogId as number;
}

export async function getLoggedGame(id: number) {
	const game = await invoke('get_logged_game', { id });
	return gameSchema.parse(game);
}