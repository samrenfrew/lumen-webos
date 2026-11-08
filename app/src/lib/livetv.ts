// Live TV: the aerial channels, their logos, and what is on now and next (docs/plans/live-tv-build.md). Everything comes through the root shell
// (lib/root.ts). A cold start draws the last answer from localStorage at once (lib/boot.ts); the TV is asked again after the first frame.

import {isWebOS} from './luna';
import {channelList, eventList, EVENT_BATCH, nowNext, readIcons, schedule} from './root';
import {logError} from './log';
import {readCache, writeCache} from './boot';
import {timeParts} from './clock';
import {getClockMode} from './i18n';
import {cachedTimeFormat} from './timeFormat';

export interface Channel {
	/** what openChannel takes: "1_41_2_102_16522_17472_9018" */
	id: string;
	/** the last three numbers of the id, which the guide's rows carry: "16522_17472_9018" */
	signal: string;
	number: string;
	name: string;
	/** the logo's file on the TV (/tmp/gibbips/logo/...), when it has one */
	logoPath?: string;
}

/** One listing; times in ms since the epoch. */
export interface Programme {
	title: string;
	start: number;
	end: number;
	/** subtitles and audio description, when the schedule says so (Now and Next does not read them) */
	subs?: boolean;
	ad?: boolean;
	/** the blurb, from the guide service or the schedule (Now and Next does not read it) */
	desc?: string;
}

/** The listings read for each channel (about two: now and next), by signal id, earliest first. */
export type Listings = Record<string, Programme[]>;

export interface NowNext {
	now?: Programme;
	next?: Programme;
}

const CHANNEL_ID = /^\d+(_\d+){6}$/;
const SIGNAL_ID = /^\d+_\d+_\d+$/;

// ---------- channels ----------

interface RawChannel {
	channelId?: string;
	signalChannelId?: string;
	channelNumber?: string | number;
	channelName?: string;
	imgUrl?: string;
	Radio?: boolean;
	Data?: boolean;
	skipped?: boolean;
}

/** The TV's channels in number order, without radio, data and skipped channels. */
export function readChannels (): Promise<Channel[]> {
	if (!isWebOS()) return Promise.resolve(import.meta.env.DEV ? MOCK_CHANNELS : []);
	return channelList().then(parseChannels);
}

export function parseChannels (text: string): Channel[] {
	const reply = JSON.parse(text) as Record<string, unknown>;
	if (reply.returnValue === false) throw new Error('getChannelList refused');
	// the list's key is not documented: take the reply's array of channels, whatever it is called
	const raw = (Array.isArray(reply.channelList) ? reply.channelList : Object.keys(reply).map((k) => reply[k]).filter(Array.isArray)[0] || []) as RawChannel[];
	const out: Channel[] = [];
	for (const c of raw) {
		if (!c || c.Radio || c.Data || c.skipped || typeof c.channelId !== 'string' || !CHANNEL_ID.test(c.channelId)) continue;
		const signal = typeof c.signalChannelId === 'string' && SIGNAL_ID.test(c.signalChannelId) ? c.signalChannelId : c.channelId.split('_').slice(4).join('_');
		out.push({
			id: c.channelId,
			signal,
			number: String(c.channelNumber === undefined ? '' : c.channelNumber),
			name: c.channelName || String(c.channelNumber || ''),
			logoPath: c.imgUrl || undefined
		});
	}
	const num = (c: Channel) => { const n = parseFloat(c.number); return isNaN(n) ? Infinity : n; };
	return out.sort((a, b) => num(a) - num(b) || (a.number < b.number ? -1 : a.number > b.number ? 1 : 0));
}

// ---------- brand colours ----------

/** Each channel's brand colour, behind its white logo on the Live TV row and in the Guide, as tvOS does for apps. The logos are white on
 *  clear, so there is no colour to sample from them. Keys are lower case, without "HD", "+1" or a region; channels not listed stay dark. */
const CHANNEL_COLORS: Record<string, string> = {
	'bbc one': '#b5121b', 'bbc two': '#0e7c7b', 'bbc three': '#d1006a', 'bbc four': '#2b2e83', 'bbc news': '#a30000',
	'bbc parliament': '#1b5e3b', 'bbc scotland': '#005eb8', 'bbc alba': '#1f6f8b', 'cbbc': '#4f9d2d', 'cbeebies': '#d97a00',
	'itv1': '#0b8a8f', 'itv': '#0b8a8f', 'stv': '#0b8a8f', 'itv2': '#c2185b', 'itv3': '#6d2077', 'itv4': '#1d3c78', 'itvbe': '#9e2a8d',
	'itv quiz': '#e05a00', 'channel 4': '#22348c', 'e4': '#5b2a86', 'e4 extra': '#5b2a86', 'more4': '#1f6f63', 'film4': '#a3162c',
	'4seven': '#3a3f9e', '5': '#6b2c91', 'channel 5': '#6b2c91', '5usa': '#0b3d91', '5star': '#d1007a', '5action': '#b3261e',
	'sky arts': '#5b1f6e', 'sky news': '#b0001e', 'quest': '#d35400', 'quest red': '#a5122a', 'dave': '#2f3640', 'yesterday': '#7a4e2d',
	'drama': '#8e1f3a', 'really': '#c2185b', 'pick': '#2e7d32', 'challenge': '#1565c0', 'gb news': '#0a2a66', 'al jazeera eng': '#b8860b',
	's4c': '#c8102e', 'great! movies': '#c62828', 'legend': '#4e342e'
};

/** a channel's brand colour, when the table has it: "BBC ONE Lon HD" and "ITV1 +1" find "bbc one" and "itv1" */
export function channelColor (name: string): string | undefined {
	const key = name.toLowerCase().replace(/\(.*?\)/g, ' ').replace(/\bhd\b|\+\s*\d+/g, ' ').replace(/\s+/g, ' ').trim();
	if (CHANNEL_COLORS[key]) return CHANNEL_COLORS[key];
	// a regional name ("bbc one lon") finds the longest key it starts with
	let best = '';
	for (const k in CHANNEL_COLORS) if (key.startsWith(k + ' ') && k.length > best.length) best = k;
	return best ? CHANNEL_COLORS[best] : undefined;
}

/** channels on the Live TV row until some are picked in Settings > Channels */
export const DEFAULT_ROW_CHANNELS = 5;

/** The Live TV row's channels: the ones picked (ids, in order, those the TV still has), or the first few by number. */
export function rowChannels (channels: Channel[], ids: string[] | null): Channel[] {
	if (!ids) return channels.slice(0, DEFAULT_ROW_CHANNELS);
	const byId: Record<string, Channel> = {};
	channels.forEach((c) => { byId[c.id] = c; });
	return ids.map((id) => byId[id]).filter((c): c is Channel => !!c);
}

/** Where typing a number lands: the channel with that number, else the first one after it (the list is in number order), else -1. */
export function findChannel (channels: Channel[], typed: string): number {
	const exact = channels.findIndex((c) => c.number === typed);
	if (exact >= 0) return exact;
	const n = Number(typed);
	return channels.findIndex((c) => parseFloat(c.number) >= n);
}

// ---------- now and next ----------

interface RawProgramme {
	signalChannelId?: string;
	programName?: string;
	startTime?: string;
	endTime?: string;
	isSubtitle?: boolean;
	isAD?: boolean;
	description?: string;
}

/** db8 returns up to 500 rows a page; about two per channel, so one page is expected. More than this many pages is a reply gone wrong. */
const MAX_PAGES = 4;

/** Now and next for every channel. Asks for the five fields Lumen reads; if the TV refuses the select, asks for whole rows instead. */
export function readListings (): Promise<Listings> {
	if (!isWebOS()) return Promise.resolve(import.meta.env.DEV ? mockListings() : {});
	const rows: RawProgramme[] = [];
	const page = (key: string | undefined, slim: boolean, n: number): Promise<Listings> => nowNext(key, slim).then((text) => {
		const reply = JSON.parse(text) as {returnValue?: boolean; results?: RawProgramme[]; next?: string};
		if (reply.returnValue === false) {
			if (slim && n === 0) return page(undefined, false, 0);
			throw new Error('epgdb find refused');
		}
		rows.push(...(reply.results || []));
		return reply.next && n + 1 < MAX_PAGES ? page(reply.next, slim, n + 1) : groupListings(rows);
	});
	return page(undefined, true, 0);
}

/** "2026,10,17,17,05,00" (UTC) as ms since the epoch, or NaN. */
export function utcTime (text: string | undefined): number {
	const p = (text || '').split(',').map(Number);
	if (p.length < 6 || p.some(isNaN)) return NaN;
	return Date.UTC(p[0], p[1] - 1, p[2], p[3], p[4], p[5]);
}

/** A description without Freeview's trailing flags ("[S]", "[AD,S]", "[HD]"). */
function blurb (text: string | undefined): string {
	return (text || '').replace(/(\s*\[[A-Z,\s]{1,12}\])+\s*$/, '').trim();
}

export function groupListings (rows: RawProgramme[]): Listings {
	const out: Listings = {};
	for (const r of rows) {
		const start = utcTime(r.startTime);
		const end = utcTime(r.endTime);
		if (!r.signalChannelId || isNaN(start) || isNaN(end) || end <= start) continue;
		const p: Programme = {title: r.programName || '', start, end};
		if (r.isSubtitle) p.subs = true;
		if (r.isAD) p.ad = true;
		const desc = blurb(r.description);
		if (desc) p.desc = desc;
		(out[r.signalChannelId] || (out[r.signalChannelId] = [])).push(p);
	}
	for (const k in out) out[k].sort((a, b) => a.start - b.start);
	return out;
}

// ---------- the full schedule (Full guide) ----------

/** the schedule holds about a week, over 20,000 rows (seen on the TV 2026-10-17), 500 a page: more pages than this is a reply gone wrong */
const MAX_SCH_PAGES = 120;
/** the grid is drawn again after every this many pages, so it fills while the rest is read */
const PART_EVERY = 3;

/** Every channel's programmes that overlap `from` to `to`, from the TV's schedule. The kind has no index on the times (seen on the TV
 *  2026-10-17: "no index for query" for both), so this reads every page and keeps only what overlaps the window. `onPart` gets what is
 *  kept so far every few pages; `stopped` ends the read early (the Full guide closed). One line goes to the TV's log per read. */
export function readSchedule (from: number, to: number, onPart?: (l: Listings) => void, stopped?: () => boolean): Promise<Listings> {
	if (!isWebOS()) return Promise.resolve(import.meta.env.DEV ? mockSchedule(from, to) : {});
	const began = Date.now();
	const kept: RawProgramme[] = [];
	let seen = 0;
	const page = (key: string | undefined, n: number): Promise<Listings> => schedule(key).then((text) => {
		const reply = JSON.parse(text) as {returnValue?: boolean; results?: RawProgramme[]; next?: string; errorText?: string};
		if (reply.returnValue === false) throw new Error(`schedule refused: ${reply.errorText || ''}`);
		const got = reply.results || [];
		seen += got.length;
		for (const r of got) if (utcTime(r.endTime) > from && utcTime(r.startTime) < to) kept.push(r);
		const more = !!reply.next && n + 1 < MAX_SCH_PAGES && !(stopped && stopped());
		if (more) {
			if (onPart && (n + 1) % PART_EVERY === 0) onPart(groupListings(kept));
			return page(reply.next, n + 1);
		}
		logError('guide', `schedule: ${n + 1} pages, ${seen} rows, ${kept.length} kept, ${Date.now() - began} ms${reply.next ? ', stopped early' : ''}`);
		return groupListings(kept);
	});
	return page(undefined, 0).catch((e: Error) => {
		logError('guide', e.message);
		throw e;
	});
}

// ---------- the guide service (iepg/getEventList): what LG's own Guide reads ----------

/** ms as the guide service's local-time stamp */
function localStamp (ms: number): string {
	const d = new Date(ms);
	const two = (n: number) => String(n).padStart(2, '0');
	return `${d.getFullYear()},${two(d.getMonth() + 1)},${two(d.getDate())},${two(d.getHours())},${two(d.getMinutes())},${two(d.getSeconds())}`;
}

/** The programmes overlapping `from` to `to` on up to EVENT_BATCH channels. The reply's startTime and endTime are UTC like the database's
 *  (its DST* times are local), so groupListings reads them as before. Rejects when the TV refuses the call. */
export function readEvents (channels: Channel[], from: number, to: number): Promise<Listings> {
	if (!isWebOS()) {
		const all = import.meta.env.DEV ? mockSchedule(from, to) : {};
		const out: Listings = {};
		for (const c of channels) if (all[c.signal]) out[c.signal] = all[c.signal];
		return Promise.resolve(out);
	}
	return eventList(channels.map((c) => c.id), localStamp(from), localStamp(to)).then((text) => {
		const reply = JSON.parse(text) as {returnValue?: boolean; programList?: RawProgramme[]; errorText?: string};
		if (reply.returnValue === false) throw new Error(`events refused: ${reply.errorText || ''}`);
		// no subtitle flag seen in the reply yet: the description carries "[S]"
		const rows = (reply.programList || []).map((r) => r.isSubtitle === undefined && /\[S\]/.test(r.description || '') ? {...r, isSubtitle: true} : r);
		return groupListings(rows.filter((r) => utcTime(r.endTime) > from && utcTime(r.startTime) < to));
	});
}

/** rows above the focus when the Full guide is scrolled (keep in step with FOCUS_AT in screens/FullGuide.tsx) */
const ROWS_ABOVE_FOCUS = 3;

/** The next channels to read: from the top row in view down, then upward from just above it, skipping those read already. */
function nextBatch (channels: Channel[], done: boolean[], focus: number): number[] {
	const out: number[] = [];
	const start = Math.max(0, Math.min(channels.length - 1, focus - ROWS_ABOVE_FOCUS));
	for (let i = start; i < channels.length && out.length < EVENT_BATCH; i++) if (!done[i]) out.push(i);
	for (let i = start - 1; i >= 0 && out.length < EVENT_BATCH; i--) if (!done[i]) out.push(i);
	return out;
}

/** Every channel's programmes from `from` to `to`, read from the guide service ten channels a call, nearest the focus first: the rows in view
 *  and the three under them, then on down, then up. `focus()` is asked before each call, so a jump down the list moves the reading with it.
 *  `onPart` gets everything read so far and the channels just asked after each call; `stopped` ends the reading early; `skip` marks channels not to ask. Rejects if the first call fails or finds
 *  nothing, so the caller can fall back to reading the database. One line goes to the TV's log per reading. */
export function readEventsNear (channels: Channel[], from: number, to: number, focus: () => number, onPart?: (l: Listings, batch: Channel[]) => void, stopped?: () => boolean, skip?: boolean[]): Promise<Listings> {
	const began = Date.now();
	const done = channels.map((_, i) => !!(skip && skip[i]));
	const out: Listings = {};
	let calls = 0;
	let programmes = 0;
	const step = (): Promise<Listings> => {
		const batch = stopped && stopped() ? [] : nextBatch(channels, done, focus());
		if (batch.length === 0) {
			logError('guide', `events: ${calls} calls, ${programmes} programmes, ${Date.now() - began} ms${stopped && stopped() ? ', stopped early' : ''}`);
			return Promise.resolve(out);
		}
		batch.forEach((i) => { done[i] = true; });
		const first = calls === 0;
		calls++;
		return readEvents(batch.map((i) => channels[i]), from, to).then((got) => {
			for (const k in got) { out[k] = got[k]; programmes += got[k].length; }
			if (first && programmes === 0) throw new Error('events: the first call found nothing');
			if (onPart) onPart({...out}, batch.map((i) => channels[i]));
			return step();
		}, (e: Error) => {
			if (first) throw e;
			logError('guide', e.message);     // one batch lost: carry on with the rest
			return step();
		});
	};
	return step().catch((e: Error) => { logError('guide', e.message); throw e; });
}

/** Each channel's Now and Next listings, with the schedule's programmes added around them. The schedule kind does not hold what is on
 *  now and next (seen on the TV 2026-10-17), so Now and Next's rows always stay, and a schedule row that overlaps one is dropped. */
export function mergeListings (sch: Listings, pf: Listings): Listings {
	const out: Listings = {...pf};
	for (const k in sch) {
		// Now and Next has no blurb: take the schedule's for the same programme
		const keep = (pf[k] || []).map((q) => { const s = q.desc ? null : sch[k].find((p) => p.start === q.start && p.desc); return s ? {...q, desc: s.desc} : q; });
		for (const p of sch[k]) if (!keep.some((q) => p.start < q.end && q.start < p.end)) keep.push(p);
		out[k] = keep.sort((a, b) => a.start - b.start);
	}
	return out;
}

/** What is on at `t`, and what follows. Worked out from the times rather than the TV's isPresent, so listings read a while ago still
 *  show the right programme. No `now` with a `next`: the channel is off air until next.start. */
export function nowNextAt (list: Programme[] | undefined, t: number): NowNext {
	if (!list) return {};
	let now: Programme | undefined;
	for (const p of list) {
		if (p.start <= t && t < p.end) now = p;
		else if (p.start > t) return {now, next: p};
	}
	return {now};
}

/** How far through a programme `t` is, 0 to 1. */
export function progress (p: Programme, t: number): number {
	return Math.max(0, Math.min(1, (t - p.start) / (p.end - p.start)));
}

/** "18:45" or "6:45 PM", following the clock setting. */
export function clockText (ms: number): string {
	const mode = getClockMode();
	const {time, period} = timeParts(new Date(ms), mode === '12h' || mode === '24h' ? mode : cachedTimeFormat());
	return period ? `${time} ${period}` : time;
}

/** "40 min left", "1 hr 5 min left" */
export function timeLeft (p: Programme, t: number): string {
	const min = Math.max(1, Math.ceil((p.end - t) / 60000));
	return min < 60 ? `${min} min left` : `${Math.floor(min / 60)} hr${min % 60 ? ` ${min % 60} min` : ''} left`;
}

// ---------- logos ----------

/** Logos as data: URIs, by their path on the TV. Kept apart from the channel list, so a full localStorage loses only the logos. */
const LOGO_CACHE_KEY = 'lumen-livetv-logos-v2';

/** How far a logo is scaled up once, on load: the TV's are as small as 133 x 100, and a focused tile shows them larger than that. */
const LOGO_SCALE = 3;

/** Draws a logo three times larger and pushes its soft edge pixels towards solid or clear, so a stretched logo stays crisp. The logos are
 *  white shapes on a clear ground, so the outline is all there is to sharpen. Done once per logo, before it is cached; any failure keeps
 *  the original. */
function crispLogo (src: string): Promise<string> {
	return new Promise((resolve) => {
		const img = new Image();
		img.onload = () => {
			try {
				const w = img.naturalWidth * LOGO_SCALE, h = img.naturalHeight * LOGO_SCALE;
				const canvas = document.createElement('canvas');
				canvas.width = w; canvas.height = h;
				const ctx = canvas.getContext('2d');
				if (!ctx || !w || !h) return resolve(src);
				ctx.imageSmoothingEnabled = true;
				ctx.imageSmoothingQuality = 'high';
				ctx.drawImage(img, 0, 0, w, h);
				const data = ctx.getImageData(0, 0, w, h);
				const px = data.data;
				for (let i = 3; i < px.length; i += 4) {
					// a smoothstep over alpha 48..208: the blurred edge narrows to about a pixel, with a little softness left so it does not jag
					const t = Math.min(1, Math.max(0, (px[i] - 48) / 160));
					px[i] = Math.round(t * t * (3 - 2 * t) * 255);
				}
				ctx.putImageData(data, 0, 0);
				resolve(canvas.toDataURL('image/png'));
			} catch { resolve(src); }
		};
		img.onerror = () => resolve(src);
		img.src = src;
	});
}

export function cachedLogos (): Record<string, string> {
	return readCache<Record<string, string>>(LOGO_CACHE_KEY) || {};
}

/** Reads the logos not in the cache yet, in one shell round-trip, and returns the whole cache. A logo whose file is missing (the TV
 *  clears /tmp on a reboot and fetches them again later) is simply asked for again next time. */
/** logo files asked for and not found while this page runs, so each guide open does not spawn a shell to ask again */
const logosNotFound = new Set<string>();

export function readLogos (channels: Channel[]): Promise<Record<string, string>> {
	const cache = cachedLogos();
	const missing = channels.map((c) => c.logoPath).filter((p, i, all): p is string => !!p && !cache[p] && !logosNotFound.has(p) && all.indexOf(p) === i);
	if (!missing.length || !isWebOS()) return Promise.resolve(cache);
	missing.forEach((p) => logosNotFound.add(p));
	return readIcons(missing).then((out) => {
		const jobs: Promise<void>[] = [];
		for (const line of out.split('\n')) {
			const m = /^(\d+) ([A-Za-z0-9+\/=]+)$/.exec(line);
			const path = m && missing[Number(m[1])];
			if (m && path) logosNotFound.delete(path);
			if (m && path) jobs.push(crispLogo(`data:image/png;base64,${m[2]}`).then((uri) => { cache[path] = uri; }));
		}
		// a logo whose file is still missing adds nothing: skip the save, since rewriting the 3 MB cache stalls the guide's open by 100 to 370 ms
		if (!jobs.length) return cache;
		return Promise.all(jobs).then(() => {
			try { localStorage.removeItem('lumen-livetv-logos-v1'); } catch { /* the unsharpened logos only take space */ }
			writeCache(LOGO_CACHE_KEY, cache);
			return cache;
		});
	});
}

// ---------- what the last launch showed ----------

const CACHE_KEY = 'lumen-boot-livetv-v1';

export interface LiveCache {
	channels: Channel[];
	listings: Listings;
	/** what the guide service read for the row and the first channels, from now for a few hours */
	schedule?: Listings;
}

export function cachedLive (): LiveCache {
	const saved = readCache<LiveCache>(CACHE_KEY);
	return saved && Array.isArray(saved.channels) && saved.listings ? saved : {channels: [], listings: {}};
}

export function saveLive (value: LiveCache): void {
	// blurbs are for the Full guide, which reads its own: keep the cache small for start-up
	const slim = (l: Listings) => { const out: Listings = {}; for (const k in l) out[k] = l[k].map(({desc, ...p}) => p); return out; };
	writeCache(CACHE_KEY, {...value, listings: slim(value.listings), schedule: value.schedule && slim(value.schedule)});
}

// ---------- dev preview (not on a TV) ----------

const MOCK_CHANNELS: Channel[] = [
	['1', 'BBC ONE HD'], ['2', 'BBC TWO HD'], ['3', 'ITV1 HD'], ['4', 'Channel 4 HD'], ['5', '5'], ['6', 'ITV2'], ['7', 'BBC THREE'],
	['8', 'More4'], ['9', 'BBC FOUR'], ['10', 'Sky Arts'], ['11', 'Quest'], ['12', 'E4']
].map(([number, name], i) => ({id: `1_41_${number}_10${i}_16522_174${i}_9018`, signal: `16522_174${i}_9018`, number, name}));

const MOCK_TITLES = [
	['BBC News at Six', 'Look North'], ['Rick Stein\'s Cornwall', 'Gardeners\' World'], ['ITV Evening News', 'Emmerdale'],
	['Four in a Bed', 'Come Dine with Me'], ['Judge Rinder', 'Police Interceptors'], ['Family Fortunes', 'The Masked Singer'],
	['Gavin and Stacey', 'Pointless'], ['Grand Designs', 'Location, Location, Location'], ['', 'Timeshift'], ['Classic Albums', 'Landscape Artist of the Year'],
	['Gold Rush', 'Wheeler Dealers'], ['The Big Bang Theory', 'Hollyoaks']
];

/** Made-up programmes of 25 to 70 minutes for every mock channel, enough to fill the grid. */
const MOCK_BLURBS = [
	'Rick visits the fishing villages of the north coast, cooks crab on the harbour wall and meets the family who have smoked mackerel in the same shed for four generations.',
	'The latest national and international news, with reports from around the region.',
	'Three couples who have never met take turns to host a dinner party, and the winner takes home a thousand pounds. Tonight, the competition heads to Bristol.',
	'Kevin McCloud follows a couple in Somerset who plan to turn a derelict water tower into a family home in under a year, on a budget that leaves no room for mistakes.'
];

function mockSchedule (from: number, to: number): Listings {
	const out: Listings = {};
	const words = MOCK_TITLES.reduce<string[]>((all, pair) => all.concat(pair.filter(Boolean)), []);
	MOCK_CHANNELS.forEach((c, i) => {
		const list: Programme[] = [];
		let t = from - (i % 3) * 20 * 60000;
		for (let k = 0; t < to; k++) {
			const len = (25 + ((i * 7 + k * 13) % 4) * 15) * 60000;
			if (!(i === 8 && k === 0)) list.push({title: words[(i * 3 + k) % words.length], start: t, end: t + len, subs: k % 2 === 0, ad: k % 3 === 0, desc: k % 4 === 3 ? undefined : MOCK_BLURBS[(i + k) % MOCK_BLURBS.length]});
			t += len;
		}
		out[c.signal] = list;
	});
	return out;
}

function mockListings (): Listings {
	const t = Date.now();
	const hour = t - (t % 3600000);
	const out: Listings = {};
	MOCK_CHANNELS.forEach((c, i) => {
		const [now, next] = MOCK_TITLES[i];
		const start = hour - (i % 3) * 20 * 60000;
		const end = start + (30 + (i % 4) * 15) * 60000;
		// the ninth channel is off air until the next hour
		out[c.signal] = now
			? [{title: now, start, end}, {title: next, start: end, end: end + 3600000}]
			: [{title: next, start: hour + 3600000, end: hour + 7200000}];
	});
	return out;
}
