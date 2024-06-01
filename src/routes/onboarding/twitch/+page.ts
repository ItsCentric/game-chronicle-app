import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { twitchCredentialsSchema } from '$lib/schemas';

export const load = async () => {
	const form = superValidate(zod(twitchCredentialsSchema));
	return { form };
};
