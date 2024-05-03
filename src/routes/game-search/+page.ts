import { authenticateWithTwitch, getRandomTopGames } from '$lib/rust-bindings/igdb';

export const load = async () => {
	const authenticateRes = await authenticateWithTwitch();
	const randomGames = await getRandomTopGames(authenticateRes.access_token, 72);
	return { randomGames };
};
