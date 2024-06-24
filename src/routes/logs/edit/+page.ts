import { getLogById } from '$lib/rust-bindings/database';
import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import type { z } from 'zod';
import { logSchema, type LogFormSchema, type StatusOption } from '$lib/schemas';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import type { IgdbGame } from '$lib/rust-bindings/igdb';

export const load: PageLoad = async ({ url }) => {
	if (typeof window === 'undefined') {
		return { igdbGame: { id: 0, title: '' } as IgdbGame, form: superValidate(zod(logSchema)) };
	}
	const id = url.searchParams.get('id');
	const minutesPlayed = url.searchParams.has('minutesPlayed')
		? parseInt(url.searchParams.get('minutesPlayed') as string)
		: undefined;
	if (id) {
		const log = await getLogById(parseInt(id));
		const formData: z.infer<LogFormSchema> = {
			logDate: new Date(log.date),
			rating: log.rating,
			notes: log.notes,
			status: log.status as StatusOption,
			timePlayedMinutes: log.minutes_played % 60,
			timePlayedHours: Math.floor(log.minutes_played / 60)
		};
		const form = await superValidate(formData, zod(logSchema));
		const { cover_id, ...game } = log.game;
		return {
			igdbGame: cover_id ? { ...game, cover: { id: 0, cover_id } } : { ...game },
			form
		};
	} else {
		if (!url.searchParams.has('game')) {
			error(404, 'Game is required');
		}
		const gameString = url.searchParams.get('game') as string;
		const game = JSON.parse(gameString) as IgdbGame;
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
			igdbGame: game,
			form
		};
	}
};
