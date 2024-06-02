import { getLogs } from '$lib/rust-bindings/database';

export const load = async () => {
	if (typeof window === 'undefined') {
		return { logs: [] };
	}
	const logs = await getLogs('date', 'desc', []);
	return {
		logs
	};
};
