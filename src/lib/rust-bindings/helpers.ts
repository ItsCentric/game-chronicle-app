import { invoke } from '@tauri-apps/api/core';
import { z } from 'zod';

const userSettingsSchema = z.object({
	username: z.string(),
	executable_paths: z.string().nullable(),
	process_monitoring: z.object({
		enabled: z.boolean(),
		directory_depth: z.number()
	}),
	autostart: z.boolean(),
	new: z.boolean()
});

export type UserSettings = z.infer<typeof userSettingsSchema>;

export async function getUserSettings() {
	const settings = await invoke('get_user_settings');
	return userSettingsSchema.parse(settings);
}

export async function saveUserSettings(settings: UserSettings) {
	const newUserSettings = await invoke('save_user_settings', { userSettings: settings });
	return userSettingsSchema.parse(newUserSettings);
}
