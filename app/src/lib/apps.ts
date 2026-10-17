// Available apps. Two providers:
//
//  1. Catalogue (default, no root): config/catalog.ts filtered by getAppLoadStatus.
//  2. Homebrew Channel (opt-in, rooted TVs only): the Homebrew Channel's service runs
//     shell commands as root for any app that asks. With the user's consent (an option
//     in Settings, off by default) we use it to run `luna-send listLaunchPoints`, which
//     gives the live list with LG's own titles, and to read each icon as base64. If the
//     service isn't there, we fall back to the catalogue.

import {APP_CATALOG, type CatalogApp} from '../config/catalog';
import {APP_ID} from '../config/constants';
import {isWebOS} from './luna';
import {listLaunchPoints, readIcons} from './root';
import {appExists} from './launch';
import {ICON_PACK} from '../config/iconpack';
import {SYSTEM_APPS} from '../config/systemApps';
import type {GlyphName} from '../components/Glyph';
import {readCache, writeCache} from './boot';
import {bootMark} from './log';

export interface AppEntry {
	id: string;
	title: string;
	/** image URL (relative asset path or data: URI); undefined → monogram */
	iconUrl?: string;
	color?: string;
	/** a small square icon (not full tile artwork): drawn at a sharp size on a tile coloured like it */
	plain?: boolean;
	/** a built-in vector icon on the colour (the built-in tiles, config/builtinApps.ts) */
	glyph?: GlyphName;
}

export type AppSource = 'catalog' | 'homebrew';

export interface AppList {
	apps: AppEntry[];
	/** where the list actually came from (homebrew falls back to catalog) */
	source: AppSource;
}

const ICON_DIR = 'icons/apps/';

const slug = (s: string) => s.toLowerCase().replace(/[^a-z0-9]+/g, '');

/** An icon from the drop-in icon pack (public/iconpack), matched by the app's id or title. It wins over every other icon. */
export function packIcon (id: string, title: string): string | undefined {
	const file = ICON_PACK[slug(id)] || ICON_PACK[slug(title)];
	return file ? 'iconpack/' + file : undefined;
}

export function listApps (useHomebrew: boolean, showSystem = false): Promise<AppList> {
	return listAppsBase(useHomebrew).then((list) => (showSystem ? addSystemApps(list) : list));
}

// ---------- the list the last launch showed (lib/boot.ts) ----------

const LIST_CACHE_KEY = 'lumen-boot-apps-v1';
/** marks an icon kept in the icon cache, so the data: URI is not stored twice */
const ICON_REF = '@icon:';
/** where each icon read through the Homebrew Channel came from, by app id */
const iconPaths: Record<string, string> = {};

/** The list the last launch showed with these options, or null. Icons read from the TV come back from the icon cache. */
export function cachedApps (useHomebrew: boolean, showSystem: boolean): AppEntry[] | null {
	const saved = readCache<{key: string; apps: AppEntry[]}>(LIST_CACHE_KEY);
	if (!saved || saved.key !== listKey(useHomebrew, showSystem) || !Array.isArray(saved.apps)) return null;
	const icons = saved.apps.some((a) => !!a.iconUrl && a.iconUrl.indexOf(ICON_REF) === 0) ? readIconCache() : {};
	return saved.apps.map((a) => {
		if (!a.iconUrl || a.iconUrl.indexOf(ICON_REF) !== 0) return a;
		const path = a.iconUrl.slice(ICON_REF.length);
		iconPaths[a.id] = path;
		return {...a, iconUrl: icons[path]};
	});
}

export function saveAppsCache (useHomebrew: boolean, showSystem: boolean, apps: AppEntry[]): void {
	const slim = apps.map((a) => (a.iconUrl && a.iconUrl.indexOf('data:') === 0 && iconPaths[a.id] ? {...a, iconUrl: ICON_REF + iconPaths[a.id]} : a));
	writeCache(LIST_CACHE_KEY, {key: listKey(useHomebrew, showSystem), apps: slim});
}

/** True when two lists would draw the same tiles (the icons compared by length and start, not in full). */
export function sameApps (a: AppEntry[], b: AppEntry[]): boolean {
	const sig = (list: AppEntry[]) => JSON.stringify(list.map((x) => [x.id, x.title, x.color, !!x.plain, x.iconUrl ? x.iconUrl.length + x.iconUrl.slice(0, 80) : '']));
	return sig(a) === sig(b);
}

function listKey (useHomebrew: boolean, showSystem: boolean): string {
	return `${useHomebrew ? 'hb' : 'cat'}:${showSystem ? 'sys' : ''}`;
}

function addSystemApps (list: AppList): Promise<AppList> {
	const have = list.apps.map((a) => a.id);
	const extra = SYSTEM_APPS.filter((a) => have.indexOf(a.id) === -1);
	const check = isWebOS() ? Promise.all(extra.map((a) => appExists(a.id))) : Promise.resolve(extra.map(() => true));
	return check.then((present) => ({...list, apps: list.apps.concat(extra.filter((_, i) => present[i]).map((a) => ({id: a.id, title: a.title, iconUrl: packIcon(a.id, a.title), color: a.color})))}));
}

function listAppsBase (useHomebrew: boolean): Promise<AppList> {
	if (!isWebOS()) return Promise.all(catalogEntries(APP_CATALOG).map(withIconColor)).then((apps): AppList => ({apps, source: 'catalog'}));
	const catalog = () => listViaCatalog().then((apps): AppList => ({apps, source: 'catalog'}));
	if (!useHomebrew) return catalog();
	return listViaHomebrew()
		.then((apps): AppList => ({apps, source: 'homebrew'}))
		.catch(catalog);
}

// ---------- provider 1: catalogue ----------

/** dev preview only: ?plain shows the catalogue's small icons the way unknown apps are drawn */
const DEV_PLAIN = import.meta.env.DEV && /[?&]plain/.test(location.search);

function catalogEntries (apps: CatalogApp[]): AppEntry[] {
	return apps.map((a) => ({
		id: a.id,
		title: a.title,
		iconUrl: (DEV_PLAIN ? undefined : packIcon(a.id, a.title)) || (a.icon ? ICON_DIR + a.icon : undefined),
		color: a.color,
		plain: DEV_PLAIN ? !!a.icon : (!packIcon(a.id, a.title) && !!a.icon)
	}));
}

function listViaCatalog (): Promise<AppEntry[]> {
	return Promise.all(APP_CATALOG.map((a) => appExists(a.id)))
		.then((present) => Promise.all(catalogEntries(APP_CATALOG.filter((_, i) => present[i])).map(withIconColor)));
}

// ---------- provider 2: Homebrew Channel (opt-in) ----------

interface LaunchPoint { id: string; title: string; largeIcon?: string; icon?: string; }

/** Apps LG lists but that make no sense on a launcher (ourselves, promos). */
const HIDDEN_IDS = [APP_ID, 'google.assistant.promo', 'com.webos.app.discovery'];

const ICON_CACHE_KEY = 'launcher-icon-cache-v1';

function listViaHomebrew (): Promise<AppEntry[]> {
	return listLaunchPoints()
		.then((out) => {
			const parsed = JSON.parse(out) as {launchPoints?: LaunchPoint[]};
			const points = (parsed.launchPoints || []).filter((p) => p.id && p.title && HIDDEN_IDS.indexOf(p.id) === -1);
			if (!points.length) throw new Error('no launch points');
			bootMark('apps-parsed', `${points.length}`);
			return loadIcons(points).then((icons) => Promise.all(points.map((p) => {
				const path = p.largeIcon || p.icon;
				if (path && icons[p.id]) iconPaths[p.id] = path;
				const known = APP_CATALOG.find((c) => c.id === p.id);
				return {
					id: p.id,
					// the catalogue's titles are shorter than LG's ("IPTV Player" vs "webOS IPTV Player")
					title: known ? known.title : p.title,
					// the catalogue's own icon before the TV's: they are full squares picked to fill a tile, the TV's often have a margin
					iconUrl: packIcon(p.id, known ? known.title : p.title) || (known && known.icon ? ICON_DIR + known.icon : undefined) || icons[p.id],
					color: known && known.color,
					plain: !packIcon(p.id, known ? known.title : p.title) && !!(icons[p.id] || (known && known.icon))
				} as AppEntry;
			}).map(withIconColor)).then((list) => { bootMark('apps-colours'); return list; }));
		});
}

/** Read the icons as data URIs in one shell round-trip, cached in localStorage by path.
 *  Icons are requested by *index*, so app IDs never reach the shell. */
function loadIcons (points: LaunchPoint[]): Promise<Record<string, string>> {
	const cache = readIconCache();
	const result: Record<string, string> = {};
	const missing: {id: string; path: string}[] = [];
	for (const p of points) {
		const path = p.largeIcon || p.icon;
		if (!path) continue;
		if (cache[path]) result[p.id] = cache[path];
		else missing.push({id: p.id, path});
	}
	if (!missing.length) return Promise.resolve(result);

	// one line per icon, "<index> <base64>" (lib/root.ts skips any path that is not a plain absolute path); app IDs never reach the shell
	return readIcons(missing.map((m) => m.path)).then((out) => {
		for (const line of out.split('\n')) {
			const match = /^(\d+) ([A-Za-z0-9+\/=]+)$/.exec(line);
			const entry = match && missing[Number(match[1])];
			if (!match || !entry) continue;
			const url = `data:image/png;base64,${match[2]}`;
			result[entry.id] = url;
			cache[entry.path] = url;
		}
		writeIconCache(cache);
		return result;
	}).catch(() => result);
}

function readIconCache (): Record<string, string> {
	try {
		const t = performance.now();
		const raw = localStorage.getItem(ICON_CACHE_KEY) || '{}';
		const out = JSON.parse(raw) as Record<string, string>;
		bootMark('iconcache', `${Math.round(raw.length / 1024)}KB ${Math.round(performance.now() - t)}ms`);
		return out;
	} catch { return {}; }
}

function writeIconCache (cache: Record<string, string>): void {
	try { localStorage.setItem(ICON_CACHE_KEY, JSON.stringify(cache)); }
	catch { /* quota exceeded: icons will simply be re-fetched next time */ }
}

/** The colour of an icon's own background (a pixel at its left edge, or its top), so a small icon can sit on a tile of the same colour. Icons read from the TV
 *  are data: URIs, so the canvas can be read; a file the app ships may not allow it, then the catalogue colour (or a dark default) is used. */
const COLOR_CACHE_KEY = 'launcher-icon-colors-v2';

function readColorCache (): Record<string, string> {
	try { return JSON.parse(localStorage.getItem(COLOR_CACHE_KEY) || '{}') as Record<string, string>; } catch { return {}; }
}

/** what one icon cost (boot timeline): how it ended, its size, and the time spent drawing and reading it */
function sampled (id: string, img: HTMLImageElement, outcome: string, t: number): void {
	bootMark('sampled', `${id} ${outcome} ${img.naturalWidth}px ${Math.round(performance.now() - t)}ms`);
}

function sampleColor (url: string, id: string): Promise<string | undefined> {
	return new Promise((resolve) => {
		const img = new Image();
		img.onload = () => {
			const t = performance.now();
			try {
				// a 16 x 16 copy: reading back a handful of pixels from a small canvas is cheap, from a full-size icon it freezes the TV
				const c = document.createElement('canvas');
				c.width = 16; c.height = 16;
				const g = c.getContext('2d');
				if (!g) { sampled(id, img, 'nocanvas', t); resolve(undefined); return; }
				g.drawImage(img, 0, 0, 16, 16);
				// the four corners and the middle of each edge; the colour most of the opaque ones share is the icon's background. One spot
				// alone can land on the logo (YouTube's red play button sits at the left edge of a white icon).
				const spots = [[1, 1], [14, 1], [1, 14], [14, 14], [8, 1], [1, 8], [14, 8], [8, 14]];
				const votes: Record<string, {n: number; rgb: string}> = {};
				let best: {n: number; rgb: string} | undefined;
				for (const [x, y] of spots) {
					const d = g.getImageData(x, y, 1, 1).data;
					if (d[3] <= 200) continue;
					const bucket = `${d[0] >> 4},${d[1] >> 4},${d[2] >> 4}`;
					const v = votes[bucket] || (votes[bucket] = {n: 0, rgb: `rgb(${d[0]}, ${d[1]}, ${d[2]})`});
					v.n++;
					if (!best || v.n > best.n) best = v;
				}
				sampled(id, img, best ? 'ok' : 'clear', t);
				resolve(best && best.rgb);
			} catch { sampled(id, img, 'taint', t); resolve(undefined); }
		};
		img.onerror = () => { sampled(id, img, 'error', performance.now()); resolve(undefined); };
		img.src = url;
	});
}

/** stored for an icon that gave no colour, so it is not tried again on every launch */
const NO_COLOR = 'none';

/** colours already found are kept by app id and icon size, so launching again does no image work at all */
function withIconColor (entry: AppEntry): Promise<AppEntry> {
	if (!entry.plain || !entry.iconUrl) return Promise.resolve(entry);
	const key = `${entry.id}:${entry.iconUrl.length}`;
	const cache = readColorCache();
	if (cache[key]) return Promise.resolve(cache[key] === NO_COLOR ? entry : {...entry, color: cache[key]});
	bootMark('sample', entry.id);
	return sampleColor(entry.iconUrl, entry.id).then((color) => {
		const next = readColorCache();
		next[key] = color || NO_COLOR;
		try { localStorage.setItem(COLOR_CACHE_KEY, JSON.stringify(next)); } catch { /* full: sampled again next time */ }
		return {...entry, color: color || entry.color};
	});
}

/** What a fresh install shows on the home screen: the catalogue's favourites first, then the other installed apps, without the TV's own system apps. */
export function seedAppIds (apps: AppEntry[], preferred: string[]): string[] {
	const system = /^(com\.webos\.|com\.palm\.|com\.lge\.|airplay$|google\.assistant|amazon\.alexa|yandex\.alice)/;
	const rest = apps.filter((a) => preferred.indexOf(a.id) === -1 && !system.test(a.id)).map((a) => a.id);
	const first = preferred.filter((id) => apps.some((a) => a.id === id));
	return first.concat(rest).slice(0, 24);
}
