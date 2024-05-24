import { authenticateWithTwitch, getRandomTopGames, type IgdbGame } from '$lib/rust-bindings/igdb';
import { writable } from 'svelte/store';

const cachedResponse = writable<IgdbGame[] | null>(null);

export const load = async () => {
	if (typeof window === 'undefined') {
		return { randomGames: [] };
	}
	let response: IgdbGame[] | null = null;
	const unsubscribe = cachedResponse.subscribe((value) => (response = value));
	if (response) {
		unsubscribe();
		return { randomGames: response };
	}
	const authenticateRes = await authenticateWithTwitch();
	const randomGames = await getRandomTopGames(authenticateRes.access_token, 72);
	cachedResponse.set(randomGames);
	unsubscribe();
	return { randomGames };
};
