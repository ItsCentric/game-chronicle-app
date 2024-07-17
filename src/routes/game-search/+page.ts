import { getRandomTopGames, type GameInfo } from '$lib/rust-bindings/igdb';
import { writable } from 'svelte/store';

const cachedResponse = writable<GameInfo[] | null>(null);

export const load = async () => {
	if (typeof window === 'undefined') {
		return { randomGames: [] };
	}
	let response: GameInfo[] | null = null;
	const unsubscribe = cachedResponse.subscribe((value) => (response = value));
	if (response) {
		unsubscribe();
		return { randomGames: response };
	}
	const randomGames = await getRandomTopGames(72);
	cachedResponse.set(randomGames);
	unsubscribe();
	return { randomGames };
};
