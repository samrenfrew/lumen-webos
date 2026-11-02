import {useEffect, useLayoutEffect, useRef, useState} from 'preact/hooks';
import {Status} from './Status';
import {Forecast} from './Forecast';
import {cachedServer, loadServer, nextAsset, picturePosition, previewUrls} from '../lib/immich';
import {nextPicture} from '../lib/wallpaper';
import {setAmbientShowing} from '../lib/system';
import {readCache, writeCache} from '../lib/boot';
import {dimForScreensaver, restoreBacklight} from '../lib/tvcontrols';
import type {Weather} from '../lib/weather';

interface Layer { urls: string[]; pos: string }
const NO_LAYER: Layer = {urls: [], pos: ''};
interface Pick { id: string; urls: string[]; pos: string }

/** how long the wallpaper the home screen showed stays before the first new photo starts loading */
const WARM_DELAY_MS = 3000;
/** the same, when a photo was made ready beforehand: only the clock's slide (900 ms) to wait for */
const WARM_READY_MS = 1200;
/** how long after a crossfade starts the next photo is made ready, so the download does not run during the fade */
const PREPARE_DELAY_MS = 3000;
/** how long the way back to the home screen takes (matches ambient-out in theme/ambient.css) */
const CLOSE_MS = 500;
/** the last photo shown, so the TV's screensaver opens on a picture (from the browser cache) rather than black */
const LAST_KEY = 'lumen-ambient-last';

/** the next photo for the screensaver, picked and loaded; with no Immich server it falls back to the wallpaper pictures */
function pickNext (show: string, source: string, pictures: string[], avoid: string): Promise<Pick | null> {
	const pic = () => { const p = nextPicture(pictures, avoid); return p ? {...p, pos: ''} : null; };
	const load = (p: Pick | null) => !p ? null : Promise.all(p.urls.map((url) => {
		const img = new Image();
		img.src = url;
		return typeof img.decode === 'function' ? img.decode() : Promise.resolve();
	})).then(() => p);
	if (show !== 'immich') return Promise.resolve(load(pic()));
	const cached = cachedServer();
	return (cached ? Promise.resolve(cached) : loadServer())
		.then((srv) => (!srv ? pic() : nextAsset(srv, source, avoid).then((id) => (id ? {id, urls: previewUrls(srv, id), pos: picturePosition(id)} : null))))
		.then(load);
}

/** one photo picked and downloaded ahead, so neither the screensaver's opening nor its next change waits on the network; it outlives the screensaver */
let ready: {key: string; pick: Promise<Pick | null>} | null = null;
const readyKey = (show: string, source: string, pictures: string[]) => `${show}|${source}|${pictures.join(',')}`;

/** Makes the next screensaver photo ready, unless one already is for these settings. The home screen calls it, so Control Center's Art mode and
 *  the idle timer both open on a photo that is already loaded. `avoid` is the photo on screen now. */
export function prepareAmbient (show: string, source: string, pictures: string[], avoid = ''): void {
	const key = readyKey(show, source, pictures);
	if (ready && ready.key === key) return;
	const last = readCache<{id: string}>(LAST_KEY);
	ready = {key, pick: pickNext(show, source, pictures, avoid || (last ? last.id : '')).catch(() => null)};
}

/** the photo made ready, or a fresh one when there is none for these settings or it failed */
function takeNext (show: string, source: string, pictures: string[], avoid: string): Promise<Pick | null> {
	const fresh = () => pickNext(show, source, pictures, avoid);
	const r = ready;
	ready = null;
	if (!r || r.key !== readyKey(show, source, pictures)) return fresh();
	return r.pick.then((p) => (p && p.id !== avoid ? p : fresh()));
}

/** where the clock must move to sit on the home screen's clock, as a transform; '' when either is missing */
function toHomeClock (clock: HTMLElement | null): string {
	const home = document.querySelector('.screen:not(.ambient__status) > .pill.status .clock');
	if (!home || !clock) return '';
	const a = home.getBoundingClientRect(), b = clock.getBoundingClientRect();
	// the home clock is smaller: it grows to the screensaver's size as it slides
	return `translate(${a.left - b.left}px, ${a.top - b.top}px) scale(${b.width ? a.width / b.width : 1})`;
}

/** Lumen's screensaver: the wallpaper pictures picked in Choose wallpaper, or your Immich photos (`show`; with no Immich server it falls back to the pictures).
 *  Opened by watch.sh with {"view":"ambient"} when the TV's screensaver starts, instead of LG's.
 *  Two full-screen layers: the next photo is loaded and decoded on the hidden one, then they swap with a crossfade (opacity only) every `seconds` (Settings > Screensaver > Time on each photo, 30 by default); each photo
 *  stays still. The clock sits bottom left, the weather and date bottom right, and the weather widget (when on) between them; a large widget takes the right side instead, with the date under the clock. Any key or a click goes back to the home screen.
 *  Started from the home screen (Control Center, or the idle timer), it opens on the wallpaper already there (`opening`): the home screen fades away, the clock
 *  slides from its place top left to bottom left, and the first new photo comes only after that, so nothing heavy runs while the screen changes. The home screen
 *  makes that photo ready beforehand (prepareAmbient), and each next one is made ready while the one before it shows. */
export function Ambient ({show, source, pictures, seconds = 30, backlight, weather, weatherLine, forecast, showPeriod, print, opening, onExit}: {show: string; source: string; pictures: string[]; seconds?: number; backlight?: number; weather?: Weather | null; weatherLine?: boolean; forecast?: 'small' | 'large'; showPeriod?: boolean; print?: boolean; opening?: {urls: string[]; id: string} | null; onExit: () => void}) {
	// each layer holds one photo's URL, or two for a portrait pair shown side by side, and where they sit (placed on their faces)
	const [last] = useState(() => (opening ? null : readCache<{id: string; urls: string[]}>(LAST_KEY)));
	const first = opening || last;
	const [layers, setLayers] = useState<[Layer, Layer]>([first ? {urls: first.urls, pos: first.id ? picturePosition(first.id) : ''} : NO_LAYER, NO_LAYER]);
	const [front, setFront] = useState(0);
	const state = useRef({front: 0, shown: first ? first.id : '', alive: true, timer: 0, prep: 0});
	const warm = useRef(!!opening).current;
	const status = useRef<HTMLDivElement>(null);
	const root = useRef<HTMLDivElement>(null);
	const exitRef = useRef(onExit);
	exitRef.current = onExit;
	const closing = useRef(false);

	// the clock slides from where the home screen shows it to its place here (transform only)
	useLayoutEffect(() => {
		const clock = status.current && status.current.querySelector<HTMLElement>('.pill.status .clock');
		const from = toHomeClock(clock);
		if (!warm || !clock || !from || document.documentElement.classList.contains('reduce-motion')) return;
		clock.style.transformOrigin = '0 0';
		clock.style.transform = from;
		void clock.offsetWidth;
		clock.style.transition = 'transform 900ms cubic-bezier(0.4, 0, 0.2, 1)';
		clock.style.transform = '';
		// the date, the weather and the widget sit elsewhere here, so they fade in rather than slide
		const fade = status.current ? status.current.querySelectorAll<HTMLElement>('.status__row, .forecast') : [];
		for (let i = 0; i < fade.length; i++) if (typeof fade[i].animate === 'function') fade[i].animate([{opacity: 0}, {opacity: 1}], {duration: 600, delay: 300, easing: 'ease-out', fill: 'backwards'});
	}, []);

	// the TV's backlight goes down while the screensaver shows (Settings > Screensaver > Brightness) and back up when it ends
	useEffect(() => {
		if (backlight === undefined) return;
		let on = true;
		dimForScreensaver(backlight, () => on);
		return () => { on = false; restoreBacklight(); };
	}, [backlight]);

	useEffect(() => {
		const s = state.current;
		s.alive = true;
		setAmbientShowing(true);
		const advance = () => {
			if (!s.alive) return;
			takeNext(show, source, pictures, s.shown)
				.then((pic) => {
					if (!pic || !s.alive) return;
					s.shown = pic.id;
					writeCache(LAST_KEY, {id: pic.id, urls: pic.urls});
					const back = 1 - s.front;
					setLayers((l) => { const n: [Layer, Layer] = [l[0], l[1]]; n[back] = {urls: pic.urls, pos: pic.pos}; return n; });
					// two frames so the hidden layer has its picture before it becomes the visible one
					window.requestAnimationFrame(() => window.requestAnimationFrame(() => { s.front = back; setFront(back); }));
					window.clearTimeout(s.prep);
					s.prep = window.setTimeout(() => { if (s.alive) prepareAmbient(show, source, pictures, pic.id); }, PREPARE_DELAY_MS);
				})
				.catch(() => undefined)
				.then(() => { if (s.alive) s.timer = window.setTimeout(advance, seconds * 1000); });
		};
		// on the wallpaper already showing, the next photo waits until the change of screen is over
		const prepared = !!ready && ready.key === readyKey(show, source, pictures);
		if (warm) s.timer = window.setTimeout(advance, prepared ? WARM_READY_MS : WARM_DELAY_MS); else advance();
		// any key goes back: the clock slides back to the home screen's clock while the screensaver fades away, then the screen changes
		const exit = (e: Event) => {
			e.preventDefault();
			if (closing.current) return;
			closing.current = true;
			const el = root.current;
			if (!el || document.documentElement.classList.contains('reduce-motion')) { exitRef.current(); return; }
			const clock = status.current && status.current.querySelector<HTMLElement>('.pill.status .clock');
			const to = toHomeClock(clock);
			if (clock && to) {
				clock.style.transformOrigin = '0 0';
				clock.style.transition = `transform ${CLOSE_MS}ms cubic-bezier(0.4, 0, 0.2, 1)`;
				clock.style.transform = to;
			}
			el.classList.add('ambient--closing');
			window.setTimeout(() => exitRef.current(), CLOSE_MS);
		};
		window.addEventListener('keydown', exit, true);
		window.addEventListener('mousedown', exit, true);
		return () => {
			s.alive = false;
			window.clearTimeout(s.timer);
			window.clearTimeout(s.prep);
			window.removeEventListener('keydown', exit, true);
			window.removeEventListener('mousedown', exit, true);
			setAmbientShowing(false);
		};
	}, [show, source, pictures, seconds]);

	return (
		<div ref={root} class={`ambient${warm ? ' ambient--warm' : ''}${forecast === 'large' && weather ? ' ambient--castlarge' : ''}`}>
			{layers.map(({urls, pos}, i) => (
				<div key={i + urls.join(' ')} class={`ambient__photo${i === front ? ' ambient__photo--on' : ''}${urls.length > 1 ? ' ambient__photo--pair' : ''}`} style={urls.length > 0 ? {backgroundImage: urls.map((u) => `url("${u}")`).join(', '), backgroundPosition: pos || undefined} : undefined} />
			))}
			{print && <div class="ambient__print" />}
			<div class="ambient__night" />
			<div class="ambient__blur" />
			<div class="ambient__shade" />
			<div class="screen ambient__status" ref={status}><Status weather={weatherLine ? weather : null} showPeriod={showPeriod} middle={forecast && weather ? <Forecast weather={weather} size={forecast} /> : null} /></div>
		</div>
	);
}
