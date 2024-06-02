import { check } from '@tauri-apps/plugin-updater';
import { getAll } from '@tauri-apps/api/window';

export const load = async () => {
	if (typeof window === 'undefined') {
		return {
			update: null
		};
	}
	let update: Awaited<ReturnType<typeof check>> = null;
	try {
		update = await check();
	} catch (error) {
		const windows = getAll();
		const mainWindow = windows.find((window) => window.label === 'main');
		const updaterWindow = windows.find((window) => window.label === 'updater');
		if (!mainWindow || !updaterWindow) {
			throw new Error('Main or updater window not found');
		}
		updaterWindow.close();
		mainWindow.show();
	}
	return {
		update
	};
};
