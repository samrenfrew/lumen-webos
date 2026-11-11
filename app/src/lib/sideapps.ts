// Settings > Side apps (docs/plans/side-apps.md): which side apps are installed, the newest version of each, and installing, updating and
// removing them. The newest versions come from the Homebrew Channel's stable listing, which the release workflow writes; the root
// commands are in lib/root.ts.

import {isWebOS} from './luna';
import {loggedFetch} from './log';
import {installSideApp, sideAppVersions, uninstallSideApp, SIDE_APPS, type SideAppId} from './root';

export {SIDE_APPS, type SideAppId};

/** the Homebrew Channel's stable listing for this repo (.github/workflows/release.yml, tools/gen_hbc_repo.py) */
const LISTING = 'https://raw.githubusercontent.com/samrenfrew/lumen-webos/hbc-repo/stable/apps.json';
const LISTING_MS = 10000;

export const SIDE_APP_TITLE: Record<SideAppId, string> = {
	'com.samrenfrew.lumen.guide': 'Lumen Guide',
	'com.samrenfrew.lumen.quick': 'Lumen Quick Settings',
	'com.samrenfrew.lumen.saver': 'Lumen Screensaver'
};

export interface Release { version: string; url: string; sha256: string }

export interface SideApps {
	/** the version installed, for each side app on the TV */
	installed: Partial<Record<SideAppId, string>>;
	/** the newest release of each side app, or null while it is read or when it could not be */
	latest: Partial<Record<SideAppId, Release>> | null;
	/** the listing could not be read */
	offline: boolean;
}

/** -1, 0 or 1, for versions like 2.2.1 */
export function compareVersions (a: string, b: string): number {
	const x = a.split('.').map(Number), y = b.split('.').map(Number);
	for (let i = 0; i < Math.max(x.length, y.length); i++) {
		const d = (x[i] || 0) - (y[i] || 0);
		if (d) return d < 0 ? -1 : 1;
	}
	return 0;
}

/** The newest release of each side app, from the listing. */
function readListing (): Promise<Partial<Record<SideAppId, Release>>> {
	const late = new Promise<never>((_, reject) => window.setTimeout(() => reject(new Error('the listing took too long')), LISTING_MS));
	const read = loggedFetch(LISTING, {cache: 'no-store'}).then((r) => {
		if (!r.ok) throw new Error(`listing ${r.status}`);
		return r.json() as Promise<{packages?: {id?: string; manifest?: {version?: string; ipkUrl?: string; ipkHash?: {sha256?: string}}}[]}>;
	}).then((j) => {
		const out: Partial<Record<SideAppId, Release>> = {};
		(j.packages || []).forEach((p) => {
			const id = p.id as SideAppId;
			const m = p.manifest;
			if (SIDE_APPS.indexOf(id) === -1 || !m || !m.version || !m.ipkUrl || !m.ipkHash || !m.ipkHash.sha256) return;
			out[id] = {version: m.version, url: m.ipkUrl, sha256: m.ipkHash.sha256};
		});
		return out;
	});
	return Promise.race([read, late]);
}

/** What is installed (only on the TV) and what is newest. */
export function readSideApps (): Promise<SideApps> {
	const installed = isWebOS() ? sideAppVersions().catch(() => ({})) : Promise.resolve({});
	const latest = readListing().then((l) => ({l, offline: false}), () => ({l: null, offline: true}));
	return Promise.all([installed, latest]).then(([i, {l, offline}]) => ({installed: i, latest: l, offline}));
}

/** The side apps that are on and have a newer release. */
export function behind (s: SideApps | null): SideAppId[] {
	if (!s || !s.latest) return [];
	const latest = s.latest;
	return SIDE_APPS.filter((id) => {
		const have = s.installed[id], next = latest[id];
		return !!have && !!next && compareVersions(have, next.version) < 0;
	});
}

/** Installs the newest release of one side app. Resolves with a line for a toast. */
export function installSide (s: SideApps | null, id: SideAppId): Promise<string> {
	const next = s && s.latest && s.latest[id];
	if (!next) return Promise.resolve(s && s.offline ? 'Could not reach GitHub for the side apps' : `${SIDE_APP_TITLE[id]} is not in a release yet`);
	return installSideApp(id, next.url, next.sha256).then((r) => (r === 'installed' ? `${SIDE_APP_TITLE[id]} ${next.version} installed` : `${SIDE_APP_TITLE[id]}: ${r}`));
}

/** Removes one side app. Resolves with a line for a toast. */
export function removeSide (id: SideAppId): Promise<string> {
	return uninstallSideApp(id).then((gone) => (gone ? `${SIDE_APP_TITLE[id]} removed` : `Could not remove ${SIDE_APP_TITLE[id]}`));
}

/** Updates every side app that is on and behind, one at a time. Resolves with a line for a toast. */
export function updateSides (s: SideApps | null): Promise<string> {
	const list = behind(s);
	if (!list.length) return Promise.resolve('The side apps are up to date');
	const lines: string[] = [];
	return list.reduce((p, id) => p.then(() => installSide(s, id).then((l) => { lines.push(l); }, (e: Error) => { lines.push(`${SIDE_APP_TITLE[id]}: ${e.message}`); })), Promise.resolve())
		.then(() => {
			const failed = lines.filter((l) => !/ installed$/.test(l));
			return failed.length ? failed.join('. ') : list.length === 1 ? lines[0] : `${list.length} side apps updated`;
		});
}
