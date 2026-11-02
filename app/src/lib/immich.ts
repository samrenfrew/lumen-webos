// Your own photos from an Immich server on the home network, as a wallpaper source. The server's address and a read-only API key live in
// /var/lib/homehook/immich.json on the TV: typed in Settings > Immich, or written over ssh (see docs/immich.md).
//
// Immich sends no CORS headers outside its dev mode, so every API call goes through curl on the TV (lib/root.ts). The picture itself is an
// ordinary <img>/background URL with the key as a query parameter, which needs no CORS. The blurred copies the glass and the lower rows show
// are made here, once per picture, from Immich's small thumbnail (fetched as base64, so the canvas is never tainted).

import {readFile, writeFile, immichRequest, immichServerOk, immichStackIds} from './root';
import {readCache, writeCache} from './boot';
import {loadConfig} from './storage';

export interface ImmichServer { url: string; key: string }

/** The choices offered in Settings > Wallpaper > Choose wallpaper > Immich, and Settings > Screensaver > Immich photos. */
export const IMMICH_SOURCES: {key: string; title: string}[] = [
	{key: 'immich:recent', title: 'Recent'},
	{key: 'immich:recent-people', title: 'Recent, with people'},
	{key: 'immich:memories', title: 'On this day'},
	{key: 'immich:favorites', title: 'Favourites'},
	{key: 'immich:all', title: 'All photos'}
];

/** The start of a window that many days back. */
function since (days: number): string { return new Date(Date.now() - days * 864e5).toISOString(); }

/** Each batch is half from the whole window, a quarter from its last third and a quarter from its last twelfth. Over a year, about a third
 *  of the photos shown are from the last month and a third are over 4 months old. */
const TIERS = [{part: 1, share: 0.5}, {part: 1 / 3, share: 0.25}, {part: 1 / 12, share: 0.25}];

const SERVER_KEY = 'lumen-immich-server-v1';
const POOL_KEY = 'lumen-immich-pool-v1';
const SHOWN_MAX = 40;      // pictures remembered as recently shown, so a shuffle does not repeat them

/** The server as last read from the TV (kept in localStorage so a cold start needs no root call to build the picture URL). */
export function cachedServer (): ImmichServer | null { return readCache<ImmichServer>(SERVER_KEY); }

/** Reads /var/lib/homehook/immich.json ({"url": "http://192.168.1.10:2283", "key": "..."}). Resolves null when it is missing or wrong. */
export function loadServer (): Promise<ImmichServer | null> {
	return readFile('immich.json').then((text) => {
		if (!text.trim()) { writeCache(SERVER_KEY, null); return null; }
		const j = JSON.parse(text) as {url?: string; key?: string};
		const url = String(j.url || '').trim().replace(/\/+$/, '').replace(/\/api$/, '');
		const server = j.key ? {url, key: String(j.key).trim()} : null;
		writeCache(SERVER_KEY, server);
		return server;
	}).catch(() => cachedServer());
}

/** A typed address as Lumen stores it: no trailing slash or /api, and http:// when no scheme was typed. */
function tidyUrl (url: string): string {
	const u = url.trim().replace(/\/+$/, '').replace(/\/api$/, '');
	return u && !/^https?:\/\//.test(u) ? 'http://' + u : u;
}

/** Settings > Immich > Test connection: the server answers, then the key opens its albums. The tidied server and its albums, or what went wrong. */
export function testServer (typed: ImmichServer): Promise<{server: ImmichServer; albums: {id: string; title: string}[]} | string> {
	const server = {url: tidyUrl(typed.url), key: typed.key.trim()};
	const ok = immichServerOk(server.url, server.key);
	if (!server.url) return Promise.resolve('Type the server address first');
	if (!ok.base) return Promise.resolve('The address should look like http://192.168.1.10:2283, with no path');
	if (!server.key) return Promise.resolve('Type the API key first');
	if (!ok.key) return Promise.resolve('That API key has characters an Immich key never has');
	return immichRequest(server.url, server.key, '/api/server/version').then(
		() => albums(server).then((list) => ({server, albums: list}), () => 'The server answered, but it refused the API key'),
		() => `No answer from ${server.url}`
	);
}

/** Writes the server to /var/lib/homehook/immich.json, where loadServer reads it, and to the cache. */
export function saveServer (server: ImmichServer): Promise<void> {
	return writeFile('immich.json', JSON.stringify({url: server.url, key: server.key})).then(() => { writeCache(SERVER_KEY, server); });
}

export function albums (server: ImmichServer): Promise<{id: string; title: string}[]> {
	return immichRequest(server.url, server.key, '/api/albums').then((text) => {
		const list = JSON.parse(text) as {id: string; albumName: string; assetCount?: number}[];
		return list.filter((a) => (a.assetCount || 0) > 0).map((a) => ({id: a.id, title: a.albumName}));
	});
}

interface Asset {
	id: string;
	type?: string;
	width?: number | null;
	height?: number | null;
	/** the people Immich found in the picture, named or not, with where their faces are; present only when asked for with withPeople */
	people?: {faces?: Face[] | null}[] | null;
	/** faces Immich found but has not put with a person */
	unassignedFaces?: Face[] | null;
	/** when it was taken, in the place it was taken */
	localDateTime?: string | null;
	fileCreatedAt?: string | null;
	exifInfo?: {exifImageWidth?: number | null; exifImageHeight?: number | null; orientation?: string | null; make?: string | null} | null;
}

/** A face box, in the pixels of the picture Immich measured it on (imageWidth x imageHeight). */
interface Face { boundingBoxX1?: number; boundingBoxY1?: number; boundingBoxX2?: number; boundingBoxY2?: number; imageWidth?: number; imageHeight?: number }

/** Width over height as the picture is seen, or 0 when Immich does not know. */
function ratio (a: Asset): number {
	let w = a.width || (a.exifInfo && a.exifInfo.exifImageWidth) || 0;
	let h = a.height || (a.exifInfo && a.exifInfo.exifImageHeight) || 0;
	const o = a.exifInfo && a.exifInfo.orientation;
	if (!a.width && (o === '6' || o === '8' || o === '5' || o === '7')) { const t = w; w = h; h = t; }   // rotated by EXIF
	return w && h ? w / h : 0;
}

/** The middle of the box round all of a photo's faces, as fractions of its width and height, or null when there are none. */
function faceCentre (faces: Face[]): [number, number] | null {
	let x1 = 1, y1 = 1, x2 = 0, y2 = 0;
	faces.forEach((f) => {
		const w = f.imageWidth || 0, h = f.imageHeight || 0;
		if (!w || !h || f.boundingBoxX1 == null || f.boundingBoxY1 == null || f.boundingBoxX2 == null || f.boundingBoxY2 == null) return;
		x1 = Math.min(x1, f.boundingBoxX1 / w); y1 = Math.min(y1, f.boundingBoxY1 / h);
		x2 = Math.max(x2, f.boundingBoxX2 / w); y2 = Math.max(y2, f.boundingBoxY2 / h);
	});
	return x2 > x1 && y2 > y1 ? [(x1 + x2) / 2, (y1 + y2) / 2] : null;
}

/** Each photo's shape and face centre ([ratio, x, y]; x and y are -1 with no faces, -2 when it has people whose faces are not read yet), by id. Kept apart from the pool so the photo the
 *  next launch opens on keeps its entry whatever the screensaver picks meanwhile; only the newest FRAMES_MAX are kept. */
const FRAMES_KEY = 'lumen-immich-frames-v1';
const FRAMES_MAX = 300;
type Frame = [number, number, number];

function readFrames (): Record<string, Frame> { return readCache<Record<string, Frame>>(FRAMES_KEY) || {}; }

function writeFrames (frames: Record<string, Frame>): void {
	const ids = Object.keys(frames);
	ids.slice(0, Math.max(0, ids.length - FRAMES_MAX)).forEach((id) => { delete frames[id]; });
	writeCache(FRAMES_KEY, frames);
}

function saveFrames (list: Asset[]): void {
	const frames = readFrames();
	list.forEach((a) => {
		const known = frames[a.id];
		delete frames[a.id];
		// faces read on an earlier pick are kept; the search itself only says whether there are people
		frames[a.id] = known && known[1] >= 0 ? known : [Math.round(ratio(a) * 1000) / 1000, a.people && a.people.length ? -2 : -1, -1];
	});
	writeFrames(frames);
}

/** Reads where the faces are in a picked photo that has people, one small request each. It needs the API key to have face.read; without it
 *  (or on any failure) the photo is simply placed as one with no faces. */
function readFaces (server: ImmichServer, pic: string): Promise<void> {
	const frames = readFrames();
	const todo = pictureIds(pic).filter((id) => frames[id] && frames[id][1] === -2);
	return Promise.all(todo.map((id) => immichRequest(server.url, server.key, `/api/faces?id=${id}`).then((text) => {
		const list = JSON.parse(text) as Face[];
		return Array.isArray(list) ? faceCentre(list) : null;
	}).catch(() => null))).then((found) => {
		const now = readFrames();
		todo.forEach((id, i) => {
			const c = found[i];
			if (now[id]) now[id] = [now[id][0], c ? Math.round(c[0] * 1000) / 1000 : -1, c ? Math.round(c[1] * 1000) / 1000 : -1];
		});
		if (todo.length) writeFrames(now);
	});
}

/** Where to place one photo in a box `box` wide and 1 high, scaled to cover it, as background-position fractions: faces sit a little
 *  above the middle; with none, more of the top is kept than the bottom, since heads and subjects tend to sit high. */
function place (f: Frame | undefined, box: number): [number, number] {
	if (!f || !f[0]) return [0.5, 0.4];
	const k = Math.max(box / f[0], 1);
	const w = f[0] * k, h = k;
	const clamp = (n: number) => Math.min(1, Math.max(0, n));
	const x = f[1] >= 0 && w > box + 0.001 ? clamp((box * 0.5 - f[1] * w) / (box - w)) : 0.5;
	const y = h <= 1.001 ? 0.5 : f[2] >= 0 ? clamp((0.42 - f[2] * h) / (1 - h)) : 0.3;
	return [x, y];
}

const pct = (n: number) => `${Math.round(n * 1000) / 10}%`;

/** The background-position for an Immich picture on a 16:9 screen: one photo, or a portrait pair side by side (each half the width less
 *  the 4px seam, as theme/base.css lays them out). */
export function picturePosition (pic: string): string {
	const frames = readFrames();
	const ids = pictureIds(pic);
	if (ids.length > 1) return ids.map((id, i) => `${i ? 'right' : 'left'} ${pct(place(frames[id], (960 - 4) / 1080)[1])}`).join(', ');
	const p = place(frames[ids[0]], 16 / 9);
	return `${pct(p[0])} ${pct(p[1])}`;
}

/** Joins the two portrait photos shown side by side as one picture ('<id>+<id>'). Immich ids are UUIDs, so never hold it. */
const PAIR = '+';

/** The Immich photos in a picture: one, or two for a portrait pair. */
export function pictureIds (pic: string): string[] { return pic.split(PAIR); }

function day (a: Asset): string { return (a.localDateTime || a.fileCreatedAt || '').slice(0, 10); }

/** Candidate pictures, and the day each photo in them was taken (by photo id, for the photos Immich gave a date). */
interface Batch { ids: string[]; days: Record<string, string> }

/** Landscape photos as they are (unknown sizes too), portrait ones in pairs taken on different days so a pair is never two shots of one
 *  moment. Square-ish ones are left out: cropped to the screen or to half of it, they lose too much. */
function arrange (list: Asset[]): Batch {
	saveFrames(list);
	const out: string[] = [];
	const days: Record<string, string> = {};
	const tall: Asset[] = [];
	list.forEach((a) => {
		const r = ratio(a);
		if (day(a)) days[a.id] = day(a);
		if (!r || r >= 1.2) out.push(a.id);
		else if (r <= 1 / 1.2) tall.push(a);
	});
	while (tall.length > 1) {
		const a = tall.shift() as Asset;
		const d = day(a);
		const i = tall.findIndex((b) => !d || day(b) !== d);
		if (i !== -1) out.push(a.id + PAIR + tall.splice(i, 1)[0].id);
	}
	for (let i = out.length - 1; i > 0; i--) { const j = Math.floor(Math.random() * (i + 1)); const t = out[i]; out[i] = out[j]; out[j] = t; }
	return {ids: out, days};
}

/** Looks like a camera photo: screenshots and saved images carry no camera make. Only applied to the broad sources (Recent, All). */
function camera (a: Asset): boolean { return !!(a.exifInfo && a.exifInfo.make); }

function today (): string {
	const d = new Date();
	return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
}

/** The photos in a stack other than its cover, which a search leaves out so a stack shows as one photo. Read once every few hours. */
const STACKS_KEY = 'lumen-immich-stacks-v1';
const STACKS_EVERY = 6 * 3600e3;
let stacksRead: Promise<Record<string, true>> | null = null;

function stackedAway (server: ImmichServer): Promise<Record<string, true>> {
	const saved = readCache<{at: number; ids: string[]}>(STACKS_KEY);
	const asSet = (ids: string[]) => { const set: Record<string, true> = {}; ids.forEach((x) => { set[x] = true; }); return set; };
	if (saved && Date.now() - saved.at < STACKS_EVERY) return Promise.resolve(asSet(saved.ids));
	if (stacksRead) return stacksRead;
	stacksRead = immichStackIds(server.url, server.key).then((text) => {
		const covers: Record<string, true> = {};
		const all: string[] = [];
		text.split('\n').forEach((line) => { if (line.charAt(0) === '=') all.push(line.slice(1)); else if (line) covers[line] = true; });
		const ids = all.filter((x) => !covers[x]);
		writeCache(STACKS_KEY, {at: Date.now(), ids});
		return asSet(ids);
	// without stack.read (or with no stacks) nothing is left out; asked again after STACKS_EVERY
	}).catch(() => { writeCache(STACKS_KEY, {at: Date.now(), ids: []}); return {} as Record<string, true>; });
	const done = () => { stacksRead = null; };
	stacksRead.then(done, done);
	return stacksRead;
}

/** A batch of candidate pictures for a source, already filtered. */
function search (server: ImmichServer, sel: string): Promise<Batch> {
	return stackedAway(server).then((away) => searchIn(server, sel, away));
}

function searchIn (server: ImmichServer, sel: string, away: Record<string, true>): Promise<Batch> {
	// several sources picked (immich:recent|immich:album:<id>): each search draws from one of them
	const parts = sel.split('|');
	if (parts.length > 1) return searchIn(server, parts[Math.floor(Math.random() * parts.length)], away);
	const keep = (list: Asset[], broad: boolean) => arrange(list.filter((a) => (!a.type || a.type === 'IMAGE') && (!broad || camera(a)) && !away[a.id]));
	const ask = (body: object, size: number) =>
		immichRequest(server.url, server.key, '/api/search/random', {body: {type: 'IMAGE', withExif: true, withPeople: true, size, ...body}})
			.then((text) => JSON.parse(text) as Asset[]);
	const random = (body: object, broad: boolean) => ask(body, 40).then((list) => keep(list, broad));
	// recent photos weighted up (TIERS); days is the window, and the first tier has no limit when all is set
	const tiered = (days: number, size: number, body: object, all = false) => Promise.all(TIERS.map((t) =>
		ask(t.part === 1 && all ? body : {...body, takenAfter: since(days * t.part)}, Math.round(size * t.share)).catch(() => [] as Asset[])
	)).then((lists) => {
		const seen: Record<string, true> = {};
		return lists.reduce((out, list) => out.concat(list), [] as Asset[]).filter((a) => !seen[a.id] && (seen[a.id] = true));
	});
	const recent = (size: number) => tiered(loadConfig().immichRecentDays, size, {});
	if (sel === 'immich:memories') {
		return immichRequest(server.url, server.key, `/api/memories?for=${today()}`).then((text) => {
			const memories = JSON.parse(text) as {assets?: Asset[]}[];
			const batch = keep(memories.reduce((all, m) => all.concat(m.assets || []), [] as Asset[]), false);
			return batch.ids.length > 0 ? batch : recent(40).then((list) => keep(list, true));   // no memories today
		});
	}
	// Immich keeps no date for when a photo was made a favourite, so the date it was taken stands in, over the last year
	if (sel === 'immich:favorites') return tiered(365, 40, {isFavorite: true}, true).then((list) => keep(list, false));
	if (sel === 'immich:recent') return recent(40).then((list) => keep(list, true));
	if (sel === 'immich:recent-people') {
		// a bigger batch, since only some have people in them; if none do (or the server leaves people out) it falls back to plain Recent
		return recent(120).then((all) => {
			const list = all.filter((a) => !!a.people && a.people.length > 0);
			const batch = keep(list, true);
			return batch.ids.length > 0 ? batch : searchIn(server, 'immich:recent', away);
		});
	}
	if (sel.indexOf('immich:album:') === 0) return random({albumIds: [sel.slice(13)]}, false);
	return random({}, true);
}

/** last: the days of the picture picked last time, which the next pick keeps away from */
interface Pool { sel: string; ids: string[]; shown: string[]; days?: Record<string, string>; last?: string[] }

/** The next picture for a source: from the pool left by the last search, searching again when it runs dry. Skips recently shown ones, and
 *  prefers one taken on other days than the last picture, searching again for one before giving that up. */
export function nextAsset (server: ImmichServer, sel: string, avoid: string): Promise<string | null> {
	const saved = readCache<Pool>(POOL_KEY);
	// a new Recent window starts a new pool, so the next picture already comes from it
	const tag = sel.indexOf('immich:recent') !== -1 ? `${sel}@${loadConfig().immichRecentDays}` : sel;
	const pool: Pool = saved && saved.sel === tag ? saved : {sel: tag, ids: [], shown: []};
	const days = pool.days || {};
	const last = pool.last || [];
	// shown and avoid are kept per photo, since the same portrait can turn up in a different pair after the next search
	const avoided = pictureIds(avoid);
	const fresh = (pic: string, seen: string[]) => !pictureIds(pic).some((x) => avoided.indexOf(x) !== -1 || seen.indexOf(x) !== -1);
	const daysOf = (pic: string) => pictureIds(pic).map((x) => days[x]).filter((d) => !!d);
	const take = (strict: boolean): string | null => {
		pool.ids = pool.ids.filter((x) => fresh(x, pool.shown));
		const i = strict ? pool.ids.findIndex((x) => !daysOf(x).some((d) => last.indexOf(d) !== -1)) : (pool.ids.length > 0 ? 0 : -1);
		return i === -1 ? null : pool.ids.splice(i, 1)[0];
	};
	const done = (id: string | null) => {
		if (id) { pool.shown = pictureIds(id).concat(pool.shown).slice(0, SHOWN_MAX); pool.last = daysOf(id); }
		// only the days of photos still in the pool are kept, so the record does not grow
		const keep: Record<string, string> = {};
		pool.ids.concat(id ? [id] : []).forEach((x) => pictureIds(x).forEach((y) => { if (days[y]) keep[y] = days[y]; }));
		pool.days = keep;
		writeCache(POOL_KEY, pool);
		return id;
	};
	const located = (id: string | null) => id ? readFaces(server, id).then(() => id) : null;
	const first = take(true);
	if (first) return located(done(first)) || Promise.resolve(null);
	return search(server, sel).then((batch) => {
		Object.keys(batch.days).forEach((k) => { days[k] = batch.days[k]; });
		pool.ids = batch.ids.concat(pool.ids.filter((x) => batch.ids.indexOf(x) === -1));
		let id = take(true) || take(false);
		if (!id && batch.ids.length > 0) { pool.shown = []; pool.ids = batch.ids.slice(); id = take(true) || take(false); }   // everything seen: start over
		return located(done(id));
	});
}

/** The picture itself, 1440 px on its long side by default in Immich: plenty for the TV, and quick to load on the home network. */
export function previewUrl (server: ImmichServer, id: string): string {
	return `${server.url}/api/assets/${id}/thumbnail?size=preview&apiKey=${encodeURIComponent(server.key)}`;
}

/** The pictures to show for a pick from nextAsset: one URL, or two for a portrait pair (left, then right). */
export function previewUrls (server: ImmichServer, pic: string): string[] { return pictureIds(pic).map((id) => previewUrl(server, id)); }

/** The blurred copy for the full-screen backdrops and the cold-start backdrop, as a data: URL, made from the small thumbnails: blurred once
 *  on a canvas here, never per frame. A portrait pair is blurred side by side, as it is shown. */
export function blurredCopy (server: ImmichServer, pic: string): Promise<string> {
	return Promise.all(pictureIds(pic).map((id) => thumbnail(server, id))).then((imgs) => blur(imgs, 480, 270, 14, 0.7));
}

function thumbnail (server: ImmichServer, id: string): Promise<HTMLImageElement> {
	return immichRequest(server.url, server.key, `/api/assets/${id}/thumbnail?size=thumbnail`, {as: 'base64'}).then((b64) => {
		const data = b64.trim();
		if (!data) throw new Error('no thumbnail');
		const mime = data.indexOf('/9j/') === 0 ? 'image/jpeg' : data.indexOf('iVBOR') === 0 ? 'image/png' : 'image/webp';
		return new Promise<HTMLImageElement>((resolve, reject) => {
			const img = new Image();
			img.onload = () => resolve(img);
			img.onerror = () => reject(new Error('thumbnail did not decode'));
			img.src = `data:${mime};base64,${data}`;
		});
	});
}

function blur (imgs: HTMLImageElement[], w: number, h: number, radius: number, quality: number): string {
	const canvas = document.createElement('canvas');
	canvas.width = w; canvas.height = h;
	const ctx = canvas.getContext('2d') as CanvasRenderingContext2D;
	ctx.filter = `blur(${radius}px)`;
	const cw = w / imgs.length;
	imgs.forEach((img, i) => {
		// cover-crop each thumbnail to its column, drawn a little oversize so the blur does not pull in transparent edges
		const scale = Math.max(cw / img.naturalWidth, h / img.naturalHeight) * 1.08;
		const dw = img.naturalWidth * scale, dh = img.naturalHeight * scale;
		ctx.save();
		ctx.beginPath();
		ctx.rect(i * cw, 0, cw, h);
		ctx.clip();
		ctx.drawImage(img, i * cw + (cw - dw) / 2, (h - dh) / 2, dw, dh);
		ctx.restore();
	});
	return canvas.toDataURL('image/jpeg', quality);
}
