import { getLogById } from '$lib/rust-bindings/database';
import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import { authenticateWithTwitch, getGamesById, type IgdbGame } from '$lib/rust-bindings/igdb';
import type { z } from 'zod';
import { logSchema, type LogFormSchema, type StatusOption } from '$lib/schemas';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';

export const load: PageLoad = async ({ url }) => {
	if (typeof window === 'undefined') {
		return { igdbGame: { id: 0, name: '' } as IgdbGame, form: superValidate(zod(logSchema)) };
	}
	if (!url.searchParams.has('gameId')) {
		error(404, 'Game ID is required');
	}
	const tokenRes = await authenticateWithTwitch();
	const id = url.searchParams.get('id') as string;
	const gameId = url.searchParams.get('gameId') as string;
	const minutesPlayed = url.searchParams.get('minutesPlayed') as string;
	const games = await getGamesById(tokenRes.access_token, [parseInt(gameId)]);
	if (id) {
		const log = await getLogById(parseInt(id));
		const formData: z.infer<LogFormSchema> = {
			logDate: new Date(log.date),
			rating: log.rating,
			notes: log.notes,
			status: log.status as StatusOption,
			finished: log.completed,
			timePlayedMinutes: log.minutes_played % 60,
			timePlayedHours: Math.floor(log.minutes_played / 60)
		};
		const form = await superValidate(formData, zod(logSchema));
		return {
			igdbGame: games[0],
			form
		};
	} else {
		const form = await superValidate(zod(logSchema));
		return {
			igdbGame: games[0],
			form
		};
	}
};
