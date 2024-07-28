import { invoke } from '@tauri-apps/api/core';
import { z } from 'zod';

const dumpVersionsSchema = z.object({
	games: z.string(),
	covers: z.string(),
	websites: z.string(),
	platforms: z.string(),
	popularity_primitives: z.string()
});
export type DumpVersions = z.infer<typeof dumpVersionsSchema>;

const dumpInfoSchema = z.object({
	name: z.enum(['games', 'covers', 'websites', 'platforms', 'popularity_primitives']),
	url: z.string(),
	version: z.string()
});
type DumpInfo = z.infer<typeof dumpInfoSchema>;

export async function getLocalDumpVersions() {
	const dumpVersions = await invoke('get_local_dump_versions');
	return dumpVersionsSchema.parse(dumpVersions);
}

export async function saveLocalDumpVersions(dumpVersions: DumpVersions) {
	await invoke('save_local_dump_versions', { dumpVersions });
}

export async function getAllDumpInfo() {
	const dumpsInfo = await invoke('get_all_dump_info');
	return z.array(dumpInfoSchema).parse(dumpsInfo);
}

export async function downloadDumps(dumpInfo: DumpInfo[], toDirectory: string) {
	await invoke('download_dumps', { dumpInfo, toDirectory });
}

export async function importDumps(fromDirectory: string) {
	await invoke('import_dumps', { fromDirectory });
}
