import { gameDetectionSchema } from '$lib/schemas';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';

export const load = async () => {
	const form = superValidate(zod(gameDetectionSchema));
	return { form };
};
