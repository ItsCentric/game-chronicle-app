import Database from '@tauri-apps/plugin-sql';
import { appDataDir } from '@tauri-apps/api/path';
import { z } from 'zod';

const dataDir = await appDataDir();
let path: string;

if (process.env.NODE_ENV === 'development') path = dataDir.concat('/dev/igdb.db');
else path = dataDir.concat('/igdb.db');

export const db = await Database.load(`sqlite:${path}`);

export const gameRowSchema = z.object({
	id: z.number(),
	title: z.string(),
	description: z.string().optional().nullable(),
	cover_id: z.string().optional().nullable(),
	release_date: z.string().optional().nullable(),
	category: z.number().optional().nullable(),
	version_parent: z.number().optional().nullable(),
	total_rating: z.number().optional().nullable()
});
type GameRow = z.infer<typeof gameRowSchema>;

const coverRowSchema = z.object({
	id: z.number(),
	image_id: z.string()
});
type CoverRow = z.infer<typeof coverRowSchema>;

const websiteRowSchema = z.object({
	id: z.number(),
	url: z.string()
});
type WebsiteRow = z.infer<typeof websiteRowSchema>;

const platformRowSchema = z.object({
	id: z.number(),
	name: z.string(),
	category: z.number()
});
type PlatformRow = z.infer<typeof platformRowSchema>;

const companyRowSchema = z.object({
	id: z.number(),
	name: z.string()
});
type CompanyRow = z.infer<typeof companyRowSchema>;

const genreRowSchema = z.object({
	id: z.number(),
	name: z.string()
});
type GenreRow = z.infer<typeof genreRowSchema>;

const gameTimeToBeatRowSchema = z.object({
	id: z.number(),
	game_id: z.number(),
	completely: z.number().optional().nullable(),
	normally: z.number().optional().nullable(),
	hastily: z.number().optional().nullable(),
	count: z.number()
});
type GameTimeToBeatRow = z.infer<typeof gameTimeToBeatRowSchema>;

const gameWebsiteRowSchema = z.object({
	game_id: z.number(),
	website_id: z.number()
});
type GameWebsiteRow = z.infer<typeof gameWebsiteRowSchema>;

const similarGameRowSchema = z.object({
	game_id: z.number(),
	similar_game_id: z.number()
});
type SimilarGameRow = z.infer<typeof similarGameRowSchema>;

const gamePlatformRowSchema = z.object({
	game_id: z.number(),
	platform_id: z.number()
});
type GamePlatformRow = z.infer<typeof gamePlatformRowSchema>;

const involvedCompanyRowSchema = z.object({
	id: z.number(),
	game_id: z.number(),
	company_id: z.number(),
	developer: z.boolean(),
	publisher: z.boolean()
});
type InvolvedCompanyRow = z.infer<typeof involvedCompanyRowSchema>;

const artworkRowSchema = z.object({
	id: z.number(),
	game_id: z.number(),
	image_id: z.string()
});
type ArtworkRow = z.infer<typeof artworkRowSchema>;

const screenshotRowSchema = z.object({
	id: z.number(),
	game_id: z.number(),
	image_id: z.string()
});
type ScreenshotRow = z.infer<typeof screenshotRowSchema>;

const videoRowSchema = z.object({
	id: z.number(),
	game_id: z.number(),
	video_id: z.string()
});
type VideoRow = z.infer<typeof videoRowSchema>;

const gameGenreRowSchema = z.object({
	id: z.number(),
	game_id: z.number(),
	genre_id: z.number()
});
type GameGenreRow = z.infer<typeof gameGenreRowSchema>;

const gameSchema = z.object({
	id: z.number(),
	title: z.string(),
	description: z.string().optional().nullable(),
	cover_image_id: z.string().optional().nullable(),
	release_date: z
		.string()
		.transform((val) => new Date(val))
		.optional()
		.nullable(),
	websites: z.array(z.string()),
	developer: z.string().optional().nullable(),
	genres: z.array(z.string()),
	category: z.number().optional().nullable(),
	version_parent: z.number().optional().nullable(),
	total_rating: z.number().optional().nullable(),
	artworks: z.array(z.string()),
	screenshots: z.array(z.string()),
	videos: z.array(z.string()),
	platforms: z.array(z.string()),
	publisher: z.string().optional().nullable(),
	time_to_beat: gameTimeToBeatRowSchema.omit({ id: true, game_id: true }).optional().nullable()
});
type Game = z.infer<typeof gameSchema>;

export async function getGameById(id: number): Promise<Game> {
	const [game] = await db.select<GameRow[]>(`SELECT * FROM games WHERE id = ?`, [id]);
	let cover: CoverRow | undefined;
	if (game.cover_id) {
		const [res] = await db.select<CoverRow[]>(`SELECT * FROM covers WHERE id = ?`, [game.cover_id]);
		cover = res;
	}
	const websites = await db.select<WebsiteRow[]>(
		`SELECT w.* FROM game_websites gw JOIN websites w ON gw.website_id = w.id WHERE gw.game_id = ?`,
		[id]
	);
	const [developer] = await db.select<CompanyRow[]>(
		`SELECT co.* FROM involved_companies ic JOIN companies co ON ic.company_id = co.id WHERE ic.game_id = ? AND ic.developer = "t"`,
		[id]
	);
	const [publisher] = await db.select<CompanyRow[]>(
		`SELECT co.* FROM involved_companies ic JOIN companies co ON ic.company_id = co.id WHERE ic.game_id = ? AND ic.publisher = "t"`,
		[id]
	);
	const genres = await db.select<GenreRow[]>(
		`SELECT g.* FROM game_genres gg JOIN genres g ON gg.genre_id = g.id WHERE gg.game_id = ?`,
		[id]
	);
	const artworks = await db.select<ArtworkRow[]>(`SELECT * FROM artworks WHERE game_id = ?`, [id]);
	const screenshots = await db.select<ScreenshotRow[]>(
		`SELECT * FROM screenshots WHERE game_id = ?`,
		[id]
	);
	const videos = await db.select<VideoRow[]>(`SELECT * FROM videos WHERE game_id = ?`, [id]);
	const platforms = await db.select<PlatformRow[]>(
		`SELECT p.* FROM game_platforms gp JOIN platforms p ON gp.platform_id = p.id WHERE gp.game_id = ?`,
		[id]
	);
	const [timeToBeat] = await db.select<GameTimeToBeatRow[]>(
		`SELECT count, completely, normally, hastily FROM game_time_to_beats WHERE game_id = ?`,
		[id]
	);

	return {
		id: game.id,
		title: game.title,
		description: game.description ?? null,
		cover_image_id: cover?.image_id ?? null,
		release_date: game.release_date ? new Date(game.release_date) : null,
		websites: websites.map((w) => w.url),
		developer: developer?.name ?? null,
		publisher: publisher?.name ?? null,
		genres: genres.map((g) => g.name),
		category: game.category ?? 0,
		version_parent: game.version_parent ?? null,
		total_rating: game.total_rating ?? null,
		artworks: artworks.map((a) => a.image_id),
		screenshots: screenshots.map((s) => s.image_id),
		videos: videos.map((v) => v.video_id),
		platforms: platforms.map((p) => p.name),
		time_to_beat: timeToBeat
	};
}
