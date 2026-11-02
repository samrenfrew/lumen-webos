// Still wallpaper behind the home screen. Choices: pick (one of the pictures picked in Settings: local:N, bundled in the app and always there,
// or cloud:<id>, from the library in config/wallpapers.ts, loaded over the internet), immich:<source>|<source>, or none.
// A cloud picture that can't load falls back to the bundled default, so the screen is never blank.
//
// Cold start with a cloud picture: the picture the next launch will show is chosen and downloaded while Lumen is
// idle (lib/wpstore.ts keeps it on the TV), and its strongly blurred copy (a few KB) goes into the boot record, so index.html paints that
// blur before the script has loaded. The app then draws, and the sharp picture, read from the TV rather than the internet, fades in over it.
// Immich (immich:<source>): the same, except that the sharp picture comes from the Immich server on the home network and the blurred copy
// is made on the TV from its thumbnail (lib/immich.ts). Two portrait photos show side by side as one picture: their URLs travel as one
// src joined by PAIR_SEP.

import {LOCAL_WALLPAPERS, WALLPAPERS, cloudUrl} from '../config/wallpapers';
import {BOOT_WALLPAPER_KEY, readCache, writeCache, type BootWallpaper} from './boot';
import {blobToDataUrl, fetchBlob, getPics, keepOnly, putPics, type Pics} from './wpstore';
import {blurredCopy, cachedServer, loadServer, nextAsset, picturePosition, previewUrls} from './immich';
import {bootMark} from './log';

const HTML_CLASS = 'wp-image';
const PROXY = 'https://images.weserv.nl/';       // free image resizer: a 1920-wide copy instead of the multi-MB original
const FADE_MS = 450;
/** the new picture's layer is drawn this long before its fade starts, so its first draw does not stall the fade */
const FADE_WARM_MS = 150;
/** how long after a picture is up before the next launch's one is fetched, so it never competes with start-up */
const PREPARE_DELAY_MS = 8000;
/** never in a URL, so it can join the two URLs of a portrait pair */
const PAIR_SEP = '\n';

/** The background-image value for a src: one picture, or a portrait pair (laid out by the wp-pair class). */
function cssImage (src: string): string { return src.split(PAIR_SEP).map((u) => `url("${u}")`).join(', '); }
function isPair (src: string): boolean { return src.indexOf(PAIR_SEP) !== -1; }

/** Resized copy of a cloud picture (width in px). */
export function resized (url: string, width: number): string {
	return `${PROXY}?url=${encodeURIComponent(url.replace(/^https?:\/\//, ''))}&w=${width}&h=${Math.round(width * 9 / 16)}&fit=cover&q=${width > 800 ? 82 : 70}&output=jpg`;
}

let picks: string[] = [];
let newOnLaunch = true;
/** the pictures 'pick' chooses from (cloud:<id>, local:N), and whether each launch opens on a new one */
const isPick = (k: string) => k.indexOf('local:') === 0 ? !!LOCAL_WALLPAPERS[Number(k.slice(6))] : WALLPAPERS.some((w) => w.id === k.slice(6));
export function setPicks (keys: string[], fresh: boolean): void {
	picks = keys.filter(isPick);
	newOnLaunch = fresh;
	// the picture on screen was just taken out of the picks: show one that is still in
	if (currentSel === 'pick' && shownKey && picks.indexOf(shownKey) === -1) setWallpaper('pick');
}

let shownId = '';     // the cloud or Immich id on screen, if any
let shownKey = '';    // for 'pick': the picture on screen (cloud:<id> or local:N)

const randomOf = (list: string[]) => list[Math.floor(Math.random() * list.length)];

/** The picture to show for 'pick': the one the last launch prepared (else a random one). A later change in this launch takes another. */
function keyFor (sel: string): string {
	if (sel !== 'pick') return sel;
	if (picks.length === 0) return '';
	const boot = readCache<BootWallpaper>(BOOT_WALLPAPER_KEY);
	if (boot && boot.sel === sel && boot.next && (!shownKey || boot.next !== shownKey) && picks.indexOf(boot.next) !== -1) return boot.next;
	const others = picks.filter((k) => k !== shownKey);
	return randomOf(others.length > 0 ? others : picks);
}

/** Art pictures get the Print look, so a painting reads as a canvas on the wall rather than a lit screen. */
function isArt (key: string): boolean {
	return key.indexOf('cloud:') === 0 && WALLPAPERS.some((w) => w.id === key.slice(6) && w.category === 'Art');
}

interface Candidate { src: string; id?: string; key?: string; caption?: PictureCaption }

export interface PictureCaption { title: string; credit?: string }
let caption: PictureCaption | null = null;
/** the title and credit of the cloud picture on screen, for the home screen caption; null for anything else */
export function getPictureCaption (): PictureCaption | null { return caption; }
function setCaption (c: PictureCaption | null): void { caption = c; window.dispatchEvent(new Event('lumen-wpcaption')); }

function candidates (sel: string): Candidate[] {
	const fallback: Candidate[] = LOCAL_WALLPAPERS.length > 0 ? [{src: LOCAL_WALLPAPERS[0].file}] : [];
	if (sel === 'none') return [];
	if (sel.indexOf('immich:') === 0) return fallback;
	const key = keyFor(sel);
	if (key.indexOf('local:') === 0) {
		const item = LOCAL_WALLPAPERS[Number(key.slice(6))];
		return item ? ([{src: item.file, key}] as Candidate[]).concat(fallback) : fallback;
	}
	const id = key.slice(6);
	const item = WALLPAPERS.filter((w) => w.id === id)[0];
	if (!item) return fallback;
	const url = cloudUrl(item);
	const cap = item.title ? {title: item.title, credit: item.credit} : undefined;
	return ([{src: resized(url, 1920), id, key, caption: cap}, {src: url, id, key, caption: cap}] as Candidate[]).concat(fallback);
}

/** The small, strongly blurred copy shown full-screen behind the lower rows, Settings and the Guide (a full-screen live blur there made
 *  moving between apps lag on the TV): bundled defaults have a -bg.jpg beside them; cloud pictures get one from the resizer. */
function strongFor (shown: string): string | undefined {
	if (shown.indexOf('wallpapers/default-') === 0) return new URL(shown.replace(/\.jpg$/, '-bg.jpg'), location.href).href;
	if (shown.indexOf(PROXY) === 0) return shown.replace(/w=\d+&h=\d+/, 'w=480&h=270').replace(/q=\d+/, 'q=80') + '&blur=18';
	return undefined;
}

/** Object URLs of the stored copies of the picture on screen (cloud pictures read from the TV), in place of the resizer's URLs. */
let local: {full: string; strong: string} | null = null;
function useLocal (pics: Pics | {full: string; strong: string} | null): void {
	if (local) [local.full, local.strong].forEach((u) => { if (u.indexOf('blob:') === 0) URL.revokeObjectURL(u); });
	if (!pics) { local = null; return; }
	const url = (v: Blob | string) => typeof v === 'string' ? v : URL.createObjectURL(v);
	local = {full: url(pics.full), strong: url(pics.strong)};
}

function setBlur (shown: string | null): void {
	const style = document.documentElement.style;
	const strong = local ? local.strong : shown ? strongFor(shown) : undefined;
	if (!strong) { style.removeProperty('--wp-bg'); return; }
	const probe = new Image();
	probe.onload = () => { style.setProperty('--wp-bg', `url("${strong}")`); saveBoot(); bootMark('wp-bg-set'); };
	probe.src = strong;
}

let element: HTMLElement | null = null;
let shown: string | null = null;      // the picture on screen (its resizer URL for a cloud picture), kept so it can be put back after it was released
let releaseTimer = 0;
let released = false;
let token = 0;
const decoded: {imgs: HTMLImageElement[]} = {imgs: []};   // held so the decoded picture stays in the cache while it is on screen

/** A bundled picture is a local file, so index.html paints it sharp before the script loads. A cloud picture is not painted from the
 *  network there: the boot record holds the blurred copy of the next launch's picture instead (prepareNext). */
function saveBoot (): void {
	if (!shown || released || !currentSel) return;
	if (currentSel.indexOf('local:') !== 0) return;      // cloud choices are written by prepareNext
	const style = document.documentElement.style;
	const boot: BootWallpaper = {sel: currentSel, url: shown, bg: style.getPropertyValue('--wp-bg') || undefined};
	writeCache(BOOT_WALLPAPER_KEY, boot);
}

let currentSel = '';
let prepareTimer = 0;

/** Chooses the picture the next launch opens on (another of the picks, or the same one when a new one each launch is off), stores it on the TV, and puts
 *  its strongly blurred copy in the boot record for index.html. */
function prepareNext (sel: string): void {
	window.clearTimeout(prepareTimer);
	if (sel.indexOf('immich:') === 0) { prepareTimer = window.setTimeout(() => prepareImmich(sel), PREPARE_DELAY_MS); return; }
	if (sel !== 'pick') return;
	prepareTimer = window.setTimeout(() => {
		if (sel !== currentSel || (!shownKey && picks.length > 0)) return;
		const others = picks.filter((k) => k !== shownKey);
		// nothing picked: the next launch opens on the default picture, as this one does
		const next = picks.length === 0 ? 'local:0' : newOnLaunch && others.length > 0 ? randomOf(others) : shownKey;
		if (next.indexOf('local:') === 0) {
			// a bundled picture: index.html paints it sharp, with its blurred copy beside it
			const item = LOCAL_WALLPAPERS[Number(next.slice(6))];
			const strong = item && strongFor(item.file);
			if (strong) writeCache(BOOT_WALLPAPER_KEY, {sel, next, url: item.file, bg: `url("${strong}")`} as BootWallpaper);
			return;
		}
		const id = next.slice(6);
		const item = WALLPAPERS.filter((w) => w.id === id)[0];
		if (!item) return;
		const full = resized(cloudUrl(item), 1920);
		const strong = strongFor(full) as string;
		getPics(full)
			.then((have) => have || Promise.all([fetchBlob(full), fetchBlob(strong)])
				.then(([f, s]) => { const pics = {full: f, strong: s}; return putPics(full, pics).then(() => pics); }))
			.then((pics) => blobToDataUrl(pics.strong))
			.then((bg) => {
				if (sel !== currentSel) return;
				writeCache(BOOT_WALLPAPER_KEY, {sel, next, bg} as BootWallpaper);
				const keep = [full];
				if (shown && shown.indexOf(PROXY) === 0) keep.push(shown);
				return keepOnly(keep);
			})
			.catch(() => undefined);    // offline or no storage: the next launch uses the network, as before
	}, PREPARE_DELAY_MS);
}

/** Immich: the next picture is chosen now and its blurred copy made, so the next launch (or the next auto-change) paints them at once. */
function prepareImmich (sel: string): void {
	const server = cachedServer();
	if (!server || sel !== currentSel) return;
	nextAsset(server, sel, shownId)
		.then((id) => id ? blurredCopy(server, id).then((bg) => ({id, bg})) : null)
		.then((r) => { if (r && sel === currentSel) writeCache(BOOT_WALLPAPER_KEY, {sel, next: r.id, bg: r.bg} as BootWallpaper); })
		.catch(() => undefined);
}

/** The Immich picture to show now: the one prepared by the last launch (its blurred copy is in the boot record), else a fresh pick. */
function immichNow (sel: string): Promise<{id: string; full: string; strong: string} | null> {
	const boot = readCache<BootWallpaper>(BOOT_WALLPAPER_KEY);
	const server = cachedServer();
	if (server && boot && boot.sel === sel && boot.next && boot.next !== shownId && boot.bg) {
		return Promise.resolve({id: boot.next, full: previewUrls(server, boot.next).join(PAIR_SEP), strong: boot.bg});
	}
	return (server ? Promise.resolve(server) : loadServer()).then((srv) => {
		if (!srv) return null;
		return nextAsset(srv, sel, shownId).then((id) => id ? blurredCopy(srv, id).then((strong) => ({id, full: previewUrls(srv, id).join(PAIR_SEP), strong})) : null);
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
				decoded.imgs = [];
				element.style.backgroundImage = 'none';
				document.documentElement.style.removeProperty('--wp-bg');
			}, 5000);
		} else if (released && element && shown) {
			released = false;
			element.style.backgroundImage = cssImage(local ? local.full : shown);
			setBlur(shown);
		}
	});
	return element;
}

/** Shows a picture: a hard cut when nothing is there yet, otherwise it fades in over what is (the blur index.html painted, or the last picture). */
let fadeTimer = 0;
function paint (el: HTMLElement, src: string, pos: string): void {
	const before = el.style.backgroundImage;
	window.clearTimeout(fadeTimer);
	const old = el.querySelector('.wpfade');
	if (old) old.remove();
	const next = cssImage(src);
	const pair = isPair(src);
	if (!before || before === 'none' || before === next || document.documentElement.classList.contains('reduce-motion')) { el.style.backgroundImage = next; el.style.backgroundPosition = pos; el.classList.toggle('wp-pair', pair); return; }
	// the new picture on a layer inside #wpimage (so night dimming covers it), faded with opacity only, then moved onto #wpimage itself
	const layer = document.createElement('div');
	layer.className = pair ? 'wpfade wp-pair' : 'wpfade';
	layer.style.backgroundImage = next;
	layer.style.backgroundPosition = pos;
	// its own GPU layer from the start, drawn while still invisible, so the fade only changes opacity
	layer.style.willChange = 'opacity';
	el.appendChild(layer);
	fadeTimer = window.setTimeout(() => {
		layer.style.opacity = '1';
		fadeTimer = window.setTimeout(() => { el.style.backgroundImage = next; el.style.backgroundPosition = pos; el.classList.toggle('wp-pair', pair); layer.remove(); }, FADE_MS + 60);
	}, FADE_WARM_MS);
}

/** The sharp picture on screen now (one URL, or two for a portrait pair) and its Immich id, or null when none is painted. */
export function shownWallpaper (): {urls: string[]; id: string} | null {
	const src = local ? local.full : shown;
	if (!src || released || !element || element.style.backgroundImage === 'none') return null;
	return {urls: src.split(PAIR_SEP), id: shownId};
}

/** The next picture for Lumen's screensaver showing Wallpaper pictures: another of the picks, or of the bundled pictures when none are picked. */
export function nextPicture (keys: string[], shown: string): {id: string; urls: string[]} | null {
	const valid = keys.filter(isPick);
	const pool = valid.length > 0 ? valid : LOCAL_WALLPAPERS.map((_, i) => `local:${i}`);
	const others = pool.filter((k) => k !== shown);
	const key = others.length > 0 ? randomOf(others) : pool[0];
	if (!key) return null;
	if (key.indexOf('local:') === 0) return {id: key, urls: [LOCAL_WALLPAPERS[Number(key.slice(6))].file]};
	const item = WALLPAPERS.filter((w) => w.id === key.slice(6))[0];
	return {id: key, urls: [resized(cloudUrl(item), 1920)]};
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
	if (list.length === 0) { root.remove(HTML_CLASS); root.remove('wp-art'); el.style.backgroundImage = 'none'; shown = null; shownId = ''; shownKey = ''; setCaption(null); useLocal(null); setBlur(null); writeCache(BOOT_WALLPAPER_KEY, null); markShown(); return; }
	const show = (c: Candidate, imgs: HTMLImageElement[], src: string) => {
		shown = c.src;
		shownId = c.id || '';
		shownKey = c.key || '';
		setCaption(c.caption || null);
		root.toggle('wp-art', isArt(shownKey));
		decoded.imgs = imgs;
		released = false;
		// an Immich photo is placed on its faces (lib/immich.ts); anything else keeps theme/base.css's centre
		paint(el, src, c.id && currentSel.indexOf('immich:') === 0 ? picturePosition(c.id) : '');
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
			// both halves of a portrait pair are loaded and decoded before either shows
			const load = (url: string) => new Promise<HTMLImageElement>((resolve, reject) => {
				const img = new Image();
				img.onload = () => resolve(img);
				img.onerror = reject;
				img.src = url;
			});
			const decode = (img: HTMLImageElement) => typeof img.decode === 'function' ? img.decode().catch(() => undefined) : Promise.resolve();
			Promise.all(pic.full.split(PAIR_SEP).map(load)).then((imgs) => {
				bootMark('wp-loaded', `${imgs.map((i) => i.naturalWidth).join('+')}px`);
				return Promise.all(imgs.map(decode)).then(() => {
					bootMark('wp-decoded');
					if (mine !== token) return;
					useLocal(pic);
					show({src: pic.full, id: pic.id}, imgs, pic.full);
				});
			}, fallback);
		}, fallback);
		return;
	}
	const tryAt = (i: number) => {
		if (mine !== token) return;                      // a newer choice replaced this one
		if (i >= list.length) { root.remove(HTML_CLASS); setCaption(null); markShown(); return; }
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
				ready.then(() => { bootMark('wp-decoded'); if (mine === token) show(c, [img], src); });
			};
			img.onerror = () => { useLocal(null); tryAt(i + 1); };
			img.src = src;
		});
	};
	tryAt(0);
}
