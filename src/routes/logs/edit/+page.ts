import { getLogById } from '$lib/rust-bindings/database';
import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import type { z } from 'zod';
import { logSchema, type LogFormSchema, type StatusOption } from '$lib/schemas';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { getGamesById, type GameInfo } from '$lib/rust-bindings/igdb';

export const load: PageLoad = async ({ url }) => {
	if (typeof window === 'undefined') {
		return { igdbGame: { id: 0, title: '' } as GameInfo, form: superValidate(zod(logSchema)) };
	}
	const id = url.searchParams.get('id');
	const minutesPlayed = url.searchParams.has('minutesPlayed')
		? parseInt(url.searchParams.get('minutesPlayed') as string)
		: undefined;
	if (id) {
		const log = await getLogById(parseInt(id));
		const formData: z.infer<LogFormSchema> = {
			logStartDate: new Date(log.start_date),
			logEndDate: new Date(log.end_date),
			rating: log.rating,
			notes: log.notes,
			status: log.status as StatusOption,
			timePlayedMinutes: log.minutes_played % 60,
			timePlayedHours: Math.floor(log.minutes_played / 60)
		};
		const form = await superValidate(formData, zod(logSchema));
		const game = await getGamesById([log.game_id]);
		return {
			igdbGame: game[0],
			form
		};
	} else {
		const gameId = url.searchParams.get('gameId');
		if (!gameId) {
			throw error(404, 'Game is required');
		}
		const game = await getGamesById([parseInt(gameId)]);
		const form = await superValidate(zod(logSchema));
		if (minutesPlayed != undefined) {
			form.data.timePlayedHours = Math.floor(minutesPlayed / 60);
			form.data.timePlayedMinutes = minutesPlayed % 60;
			form.data.status = 'playing';
		} else {
			form.data.timePlayedHours = 0;
			form.data.timePlayedMinutes = 0;
		}
		return {
			igdbGame: game[0],
			form
		};
	}
};
