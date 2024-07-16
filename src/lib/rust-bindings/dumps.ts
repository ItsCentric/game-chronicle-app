import { invoke } from '@tauri-apps/api/core';
import { z } from 'zod';

const dumpVersionsSchema = z.object({
	games: z.string(),
	covers: z.string(),
	websites: z.string(),
	platforms: z.string()
});
export type DumpVersions = z.infer<typeof dumpVersionsSchema>;

const dumpInfoSchema = z.object({
	name: z.enum(['games', 'covers', 'websites', 'platforms']),
	url: z.string(),
	version: z.string()
});
type DumpInfo = z.infer<typeof dumpInfoSchema>;

export async function getLocalDumpVersions() {
	console.log('getLocalDumpVersions');
	const dumpVersions = await invoke('get_local_dump_versions');
	console.log('dumpVersions', dumpVersions);
	return dumpVersionsSchema.parse(dumpVersions);
}

export async function saveLocalDumpVersions(dumpVersions: DumpVersions) {
	console.log('saveLocalDumpVersions');
	await invoke('save_local_dump_versions', { dumpVersions });
	console.log('saveLocalDumpVersions done');
}

export async function getAllDumpInfo() {
	console.log('getAllDumpInfo');
	const dumpsInfo = await invoke('get_all_dump_info');
	console.log('dumpsInfo', dumpsInfo);
	return z.array(dumpInfoSchema).parse(dumpsInfo);
}

export async function downloadDumps(dumpInfo: DumpInfo[], toDirectory: string) {
	console.log('downloadDumps');
	await invoke('download_dumps', { dumpInfo, toDirectory });
	console.log('downloadDumps done');
}

export async function importDumps(fromDirectory: string) {
	console.log('importDumps');
	await invoke('import_dumps', { fromDirectory });
	console.log('importDumps done');
}
