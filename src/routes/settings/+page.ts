import { getUserSettings } from '$lib/rust-bindings/helpers';
import { settingsSchema } from '$lib/schemas';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';

export const load = async () => {
	if (typeof window === 'undefined') {
		return { form: superValidate(zod(settingsSchema)) };
	}
	const userSettings = await getUserSettings();
	const formData = {
		username: userSettings.username,
		executablePaths: userSettings.executable_paths ? userSettings.executable_paths.split(';') : [],
		processMonitoringEnabled: userSettings.process_monitoring.enabled,
		processMonitoringDirectoryDepth: userSettings.process_monitoring.directory_depth,
		autostart: userSettings.autostart
	};
	const form = await superValidate(formData, zod(settingsSchema));

	return { form };
};
