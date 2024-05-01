import { getUserSettings } from '$lib/rust-bindings/main';
import { settingsSchema } from '$lib/schemas';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';

export const load = async () => {
	const userSettings = await getUserSettings();
	const formData = {
		username: userSettings.username,
		executablePaths: userSettings.executable_paths.split(';'),
		processMonitoringEnabled: userSettings.process_monitoring.enabled,
		processMonitoringDirectoryDepth: userSettings.process_monitoring.directory_depth
	};
	const form = await superValidate(formData, zod(settingsSchema));

	return { form };
};
