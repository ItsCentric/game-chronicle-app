import type { HandleClientError } from '@sveltejs/kit';
import { writeErrorLog } from '$lib';

export const handleError: HandleClientError = async ({ error }) => {
	writeErrorLog(error);
};
