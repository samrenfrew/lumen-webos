// Still wallpaper behind the home screen. Choices: local:N (bundled in the app, always works), cloud:<id> (from the cloud
// library in config/wallpapers.ts, loaded over the internet when picked), shuffle (a random cloud one each launch), none.
// A cloud picture that can't load falls back to the bundled default, so the screen is never blank.
//
// Cold start with a cloud picture (a fixed one or a shuffle): the picture the next launch will show is chosen and downloaded while Lumen is
// idle (lib/wpstore.ts keeps it on the TV), and its strongly blurred copy (a few KB) goes into the boot record, so index.html paints that
// blur before the script has loaded. The app then draws, and the sharp picture, read from the TV rather than the internet, fades in over it.
// Immich (immich:<source>): the same, except that the sharp picture comes from the Immich server on the home network and the blurred copies
// are made on the TV from its thumbnail (lib/immich.ts).

import {LOCAL_WALLPAPERS, WALLPAPERS, cloudUrl} from '../config/wallpapers';
import {BOOT_WALLPAPER_KEY, readCache, writeCache, type BootWallpaper} from './boot';
import {blobToDataUrl, fetchBlob, getPics, keepOnly, putPics, type Pics} from './wpstore';
import {blurredCopies, cachedServer, loadServer, nextAsset, previewUrl} from './immich';
import {bootMark} from './log';

const HTML_CLASS = 'wp-image';
const PROXY = 'https://images.weserv.nl/';       // free image resizer: a 1920-wide copy instead of the multi-MB original
const FADE_MS = 450;
/** how long after a picture is up before the next launch's one is fetched, so it never competes with start-up */
const PREPARE_DELAY_MS = 8000;

/** Resized copy of a cloud picture (width in px). */
export function resized (url: string, width: number): string {
	return `${PROXY}?url=${encodeURIComponent(url.replace(/^https?:\/\//, ''))}&w=${width}&h=${Math.round(width * 9 / 16)}&fit=cover&q=${width > 800 ? 82 : 70}&output=jpg`;
}

let favorites: string[] = [];
/** the favourite cloud ids, for the 'shuffle:Favorites' choice */
export function setFavorites (ids: string[]): void { favorites = ids; }

function group (sel: string) {
	// 'shuffle' = anything from the cloud library; 'shuffle:<category>' = only that category
	if (sel === 'shuffle:Favorites') return WALLPAPERS.filter((w) => favorites.indexOf(w.id) !== -1);
	if (sel.indexOf('shuffle:') === 0) return WALLPAPERS.filter((w) => w.category === sel.slice(8));
	return WALLPAPERS;
}

let shownId = '';     // the cloud id on screen, if any

/** The cloud id to show for a choice: a fixed one, or for a shuffle the one prepared by the last launch (else a random one). */
function cloudIdFor (sel: string): string {
	if (sel.indexOf('cloud:') === 0) return sel.slice(6);
	if (sel.indexOf('shuffle') !== 0) return '';
	const list = group(sel);
	if (list.length === 0) return '';
	const boot = readCache<BootWallpaper>(BOOT_WALLPAPER_KEY);
	if (boot && boot.sel === sel && boot.next && boot.next !== shownId && list.some((w) => w.id === boot.next)) return boot.next;
	return list[Math.floor(Math.random() * list.length)].id;
}

interface Candidate { src: string; id?: string }

function candidates (sel: string): Candidate[] {
	const fallback: Candidate[] = LOCAL_WALLPAPERS.length > 0 ? [{src: LOCAL_WALLPAPERS[0].file}] : [];
	if (sel === 'none') return [];
	if (sel.indexOf('local:') === 0) {
		const item = LOCAL_WALLPAPERS[Number(sel.slice(6))];
		return item ? ([{src: item.file}] as Candidate[]).concat(fallback) : fallback;
	}
	const id = cloudIdFor(sel);
	const item = WALLPAPERS.filter((w) => w.id === id)[0];
	if (!item) return fallback;
	const url = cloudUrl(item);
	return ([{src: resized(url, 1920), id}, {src: url, id}] as Candidate[]).concat(fallback);
}

/** The light blurred copy the glass shapes show: bundled defaults have a -blur.jpg beside them; cloud pictures get one from the resizer. */
function lightFor (shown: string): string | undefined {
	if (shown.indexOf('wallpapers/default-') === 0) return new URL(shown.replace(/\.jpg$/, '-blur.jpg'), location.href).href;
	if (shown.indexOf(PROXY) === 0) return shown.replace(/w=\d+&h=\d+/, 'w=960&h=540').replace(/q=\d+/, 'q=80') + '&blur=5';
	return undefined;
}
/** The strong, smaller blurred copy shown behind the lower rows. */
function strongFor (light: string): string {
	return light.replace(/-blur\.jpg$/, '-bg.jpg').replace('w=960&h=540', 'w=480&h=270').replace(/blur=\d+/, 'blur=18');
}

/** Settings > Live blur: the glass shapes blur what is behind them (backdrop-filter), so the light copy (--wp-blur) is not loaded. The strong
 *  copy (--wp-bg) still is: a full-screen live blur behind the lower rows made moving between apps lag on the TV. */
let liveBlur = false;
export function setLiveBlur (on: boolean): void {
	if (on === liveBlur) return;
	liveBlur = on;
	setBlur(shown);
}

/** Object URLs of the stored copies of the picture on screen (cloud pictures read from the TV), in place of the resizer's URLs. */
let local: {full: string; light: string; strong: string} | null = null;
function useLocal (pics: Pics | {full: string; light: string; strong: string} | null): void {
	if (local) [local.full, local.light, local.strong].forEach((u) => { if (u.indexOf('blob:') === 0) URL.revokeObjectURL(u); });
	if (!pics) { local = null; return; }
	const url = (v: Blob | string) => typeof v === 'string' ? v : URL.createObjectURL(v);
	local = {full: url(pics.full), light: url(pics.light), strong: url(pics.strong)};
}

function setBlur (shown: string | null): void {
	const style = document.documentElement.style;
	const light = local ? local.light : shown ? lightFor(shown) : undefined;
	if (!light) { style.removeProperty('--wp-blur'); style.removeProperty('--wp-bg'); return; }
	const load = (name: string, src: string) => {
		const probe = new Image();
		probe.onload = () => { style.setProperty(name, `url("${src}")`); saveBoot(); bootMark(name.slice(2) + '-set'); };
		probe.src = src;
	};
	if (liveBlur) style.removeProperty('--wp-blur');
	else load('--wp-blur', local ? local.light : light);
	load('--wp-bg', local ? local.strong : strongFor(light));
}

let element: HTMLElement | null = null;
let shown: string | null = null;      // the picture on screen (its resizer URL for a cloud picture), kept so it can be put back after it was released
let releaseTimer = 0;
let released = false;
let token = 0;
const decoded: {img: HTMLImageElement | null} = {img: null};   // held so the decoded picture stays in the cache while it is on screen

/** A bundled picture is a local file, so index.html paints it sharp before the script loads. A cloud picture is not painted from the
 *  network there: the boot record holds the blurred copy of the next launch's picture instead (prepareNext). */
function saveBoot (): void {
	if (!shown || released || !currentSel) return;
	if (currentSel.indexOf('local:') !== 0) return;      // cloud choices are written by prepareNext
	const style = document.documentElement.style;
	const boot: BootWallpaper = {sel: currentSel, url: shown, blur: style.getPropertyValue('--wp-blur') || undefined, bg: style.getPropertyValue('--wp-bg') || undefined};
	writeCache(BOOT_WALLPAPER_KEY, boot);
}

let currentSel = '';
let prepareTimer = 0;

/** Chooses the picture the next launch opens on (the same one for a fixed choice, another one for a shuffle), stores it on the TV, and puts
 *  its strongly blurred copy in the boot record for index.html. */
function prepareNext (sel: string): void {
	window.clearTimeout(prepareTimer);
	if (sel.indexOf('immich:') === 0) { prepareTimer = window.setTimeout(() => prepareImmich(sel), PREPARE_DELAY_MS); return; }
	if (sel.indexOf('cloud:') !== 0 && sel.indexOf('shuffle') !== 0) return;
	prepareTimer = window.setTimeout(() => {
		if (sel !== currentSel) return;
		let id = shownId;
		if (sel.indexOf('shuffle') === 0) {
			const others = group(sel).filter((w) => w.id !== shownId);
			if (others.length > 0) id = others[Math.floor(Math.random() * others.length)].id;
		}
		const item = WALLPAPERS.filter((w) => w.id === id)[0];
		if (!item) return;
		const full = resized(cloudUrl(item), 1920);
		const light = lightFor(full) as string;
		getPics(full)
			.then((have) => have || Promise.all([fetchBlob(full), fetchBlob(light), fetchBlob(strongFor(light))])
				.then(([f, l, s]) => { const pics = {full: f, light: l, strong: s}; return putPics(full, pics).then(() => pics); }))
			.then((pics) => blobToDataUrl(pics.strong))
			.then((bg) => {
				if (sel !== currentSel) return;
				writeCache(BOOT_WALLPAPER_KEY, {sel, next: id, bg} as BootWallpaper);
				const keep = [full];
				if (shown && shown.indexOf(PROXY) === 0) keep.push(shown);
				return keepOnly(keep);
			})
			.catch(() => undefined);    // offline or no storage: the next launch uses the network, as before
	}, PREPARE_DELAY_MS);
}

/** Immich: the next picture is chosen now and its blurred copies made, so the next launch (or the next auto-change) paints them at once. */
function prepareImmich (sel: string): void {
	const server = cachedServer();
	if (!server || sel !== currentSel) return;
	nextAsset(server, sel, shownId)
		.then((id) => id ? blurredCopies(server, id).then((b) => ({id, b})) : null)
		.then((r) => { if (r && sel === currentSel) writeCache(BOOT_WALLPAPER_KEY, {sel, next: r.id, bg: r.b.strong, blur: r.b.light} as BootWallpaper); })
		.catch(() => undefined);
}

/** The Immich picture to show now: the one prepared by the last launch (its blurred copies are in the boot record), else a fresh pick. */
function immichNow (sel: string): Promise<{id: string; full: string; light: string; strong: string} | null> {
	const boot = readCache<BootWallpaper>(BOOT_WALLPAPER_KEY);
	const server = cachedServer();
	if (server && boot && boot.sel === sel && boot.next && boot.next !== shownId && boot.bg) {
		return Promise.resolve({id: boot.next, full: previewUrl(server, boot.next), light: boot.blur || boot.bg, strong: boot.bg});
	}
	return (server ? Promise.resolve(server) : loadServer()).then((srv) => {
		if (!srv) return null;
		return nextAsset(srv, sel, shownId).then((id) => id ? blurredCopies(srv, id).then((b) => ({id, full: previewUrl(srv, id), light: b.light, strong: b.strong})) : null);
	});
}

function ensureElement (): HTMLElement {
	if (element) return element;
	// index.html has the element already (with the last picture, or the next one's blur, in it); dev builds and older pages may not
	element = document.getElementById('wpimage');
	if (!element) {
		element = document.createElement('div');
		element.id = 'wpimage';
		document.body.insertBefore(element, document.body.firstChild);
	}
	document.addEventListener('visibilitychange', () => {
		window.clearTimeout(releaseTimer);
		if (document.hidden) {
			releaseTimer = window.setTimeout(() => {
				if (!element || !shown) return;
				released = true;
				decoded.img = null;
				element.style.backgroundImage = 'none';
				document.documentElement.style.removeProperty('--wp-blur');
				document.documentElement.style.removeProperty('--wp-bg');
			}, 5000);
		} else if (released && element && shown) {
			released = false;
			element.style.backgroundImage = `url("${local ? local.full : shown}")`;
			setBlur(shown);
		}
	});
	return element;
}

/** Shows a picture: a hard cut when nothing is there yet, otherwise it fades in over what is (the blur index.html painted, or the last picture). */
let fadeTimer = 0;
function paint (el: HTMLElement, src: string): void {
	const before = el.style.backgroundImage;
	window.clearTimeout(fadeTimer);
	const old = el.querySelector('.wpfade');
	if (old) old.remove();
	const next = `url("${src}")`;
	if (!before || before === 'none' || before === next || document.documentElement.classList.contains('reduce-motion')) { el.style.backgroundImage = next; return; }
	// the new picture on a layer inside #wpimage (so night dimming covers it), faded with opacity only, then moved onto #wpimage itself
	const layer = document.createElement('div');
	layer.className = 'wpfade';
	layer.style.backgroundImage = `url("${src}")`;
	el.appendChild(layer);
	window.requestAnimationFrame(() => window.requestAnimationFrame(() => { layer.style.opacity = '1'; }));
	fadeTimer = window.setTimeout(() => { el.style.backgroundImage = next; layer.remove(); }, FADE_MS + 60);
}

/** Resolves once the first wallpaper of this launch is on screen (or there is none to show), so heavier work can wait for it. */
let markShown: () => void = () => undefined;
export const wallpaperShown: Promise<void> = new Promise((resolve) => { markShown = resolve; });
wallpaperShown.then(() => bootMark('wallpaper'));

export function setWallpaper (sel: string): void {
	bootMark('wp-start');
	const el = ensureElement();
	const mine = ++token;
	currentSel = sel;
	const list = candidates(sel);
	const root = document.documentElement.classList;
	if (list.length === 0) { root.remove(HTML_CLASS); el.style.backgroundImage = 'none'; shown = null; shownId = ''; useLocal(null); setBlur(null); writeCache(BOOT_WALLPAPER_KEY, null); markShown(); return; }
	const show = (c: Candidate, img: HTMLImageElement, src: string) => {
		shown = c.src;
		shownId = c.id || '';
		decoded.img = img;
		released = false;
		paint(el, src);
		root.add(HTML_CLASS);
		setBlur(c.src);
		saveBoot();
		prepareNext(sel);
		markShown();
	};
	if (sel.indexOf('immich:') === 0) {
		// on failure (no server set up, server away, nothing matches) the bundled default is shown, as for a cloud picture that will not load
		const fallback = () => { if (mine === token) { useLocal(null); tryAt(list.length - 1); } };
		immichNow(sel).then((pic) => {
			if (mine !== token) return;
			if (!pic) { fallback(); return; }
			const img = new Image();
			img.onload = () => {
				bootMark('wp-loaded', `${img.naturalWidth}px`);
				const ready = typeof img.decode === 'function' ? img.decode().catch(() => undefined) : Promise.resolve();
				ready.then(() => { bootMark('wp-decoded'); if (mine !== token) return; useLocal(pic); show({src: pic.full, id: pic.id}, img, pic.full); });
			};
			img.onerror = fallback;
			img.src = pic.full;
		}, fallback);
		return;
	}
	const tryAt = (i: number) => {
		if (mine !== token) return;                      // a newer choice replaced this one
		if (i >= list.length) { root.remove(HTML_CLASS); markShown(); return; }
		const c = list[i];
		// a cloud picture already on the TV (prepared by the last launch) is read from there; anything else comes from its URL
		const stored = c.src.indexOf(PROXY) === 0 ? getPics(c.src) : Promise.resolve(null);
		stored.then((pics) => {
			if (mine !== token) return;
			useLocal(pics);
			const src = local ? local.full : c.src;
			const img = new Image();
			img.onload = () => {
				bootMark('wp-loaded', `${img.naturalWidth}px`);
				// decode off the main thread before the swap, so the first frame with the new picture does not stall on decoding it
				const ready = typeof img.decode === 'function' ? img.decode().catch(() => undefined) : Promise.resolve();
				ready.then(() => { bootMark('wp-decoded'); if (mine === token) show(c, img, src); });
			};
			img.onerror = () => { useLocal(null); tryAt(i + 1); };
			img.src = src;
		});
	};
	tryAt(0);
}
