import {t as tr} from '../lib/i18n';
import {useCallback, useEffect, useLayoutEffect, useRef, useState} from 'preact/hooks';
import {Glyph, type GlyphName} from './Glyph';
import {useKeys, clamp, type NavKey} from '../hooks/useKeys';
import {PICTURE_LABEL, PICTURE_MODES, SOUND_LABEL, SOUND_MODES, setSoundMode, readBacklight, readControls, setBacklight, setEyeComfort, restartTv, screenOff, setPictureMode, type Controls} from '../lib/tvcontrols';

export type CenterAction = 'lg-home' | 'art';

interface Props {
	onClose: () => void;
	/** Down twice at the bottom row: close and go to the glass bar */
	onCloseDown: () => void;
	onError: (text: string) => void;
	/** buttons that leave the launcher or open another screen */
	onAction: (action: CenterAction) => void;
	darkMode: string;
	clockFormat: string;
	onDarkMode: (mode: string) => void;
	onClockFormat: (format: string) => void;
}

type CellId = 'sound' | 'picture' | 'art' | 'home' | 'screenoff' | 'restart' | 'backlight' | 'eye' | 'dark' | 'clock';
/** the tiles that open a list of choices in place of the grid, as tvOS does for Sleep Timer */
type MenuCell = 'sound' | 'picture' | 'dark' | 'clock';
const MENU_CELLS: CellId[] = ['sound', 'picture', 'dark', 'clock'];
const MENU_GLYPH: Record<MenuCell, GlyphName> = {sound: 'speaker', picture: 'picture', dark: 'screensaver', clock: 'timer'};

/** The layout, like the tvOS Control Center: a grid four squares wide, where a tile spans two squares and the backlight bar three. */
const GRID: {id: CellId; span: number}[][] = [
	[{id: 'sound', span: 2}, {id: 'picture', span: 2}],
	[{id: 'dark', span: 2}, {id: 'clock', span: 2}],
	[{id: 'art', span: 1}, {id: 'home', span: 1}, {id: 'screenoff', span: 1}, {id: 'restart', span: 1}],
	[{id: 'backlight', span: 3}, {id: 'eye', span: 1}]
];

/** The cell in a row that covers grid column x (0 to 3), and the column it starts at. */
const cellAt = (row: number, x: number) => {
	let start = 0;
	for (const c of GRID[row]) { if (x < start + c.span) return {cell: c.id, start}; start += c.span; }
	return {cell: GRID[row][GRID[row].length - 1].id, start: 4 - GRID[row][GRID[row].length - 1].span};
};
const startOf = (row: number, i: number) => GRID[row].slice(0, i).reduce((n, c) => n + c.span, 0);

/** how soon a second Down at the bottom row must follow the first to close the panel */
export const DOUBLE_DOWN_MS = 1000;

/** Each backlight change is a root command (a luna-send process on the TV), so a run of presses sends only the last level, this long after. */
const BACKLIGHT_DELAY_MS = 300;

/** Opening a list, timed off the tvOS 26 Control Center frame by frame (30 fps): the tile grows into a glass pill the list's size while its text
 *  fades and the rest of the grid shrinks toward it and fades; the list fades in inside the pill, then the pill's glass clears. tvOS also
 *  darkens the pill on the way, which looked jarring here, so this keeps the tile's glass. Closing has no pill, since a big pill shrinking
 *  is heavy on the TV: the list shrinks back toward the tile as it fades out, and the grid fades back in, growing back from GRID_SHRINK. */
const OPEN_MS = 330;
const CLOSE_MS = 300;
/** a long list (Sound, Picture: 7 choices) runs this much quicker, as its bigger pill felt slow on the TV */
const LONG_LIST = 4;
const LONG_STRETCH = 0.85;
/** keyframes per animation; enough that the pill's corners stay round between them */
const STEPS = 20;
/** how far the rest of the grid shrinks toward the tile */
const GRID_SHRINK = 0.15;
/** the space round the list inside the pill, so the options never touch its edge */
const PILL_PAD = 18;
/** how soon in the open the pill takes the list's shape (its height grows faster than its width), as a share of OPEN_MS */
const SHAPE_BY = 0.35;
/** how far past the tile the pill shrinks when closing, before it settles */
/** how far the pill has grown at each video frame (every 33 ms) of tvOS's open, measured off the frames */
const GROW = [0, 0.12, 0.35, 0.6, 0.8, 0.9, 0.94, 0.97, 0.985, 0.995, 1];
/** where the pill is: its offset and scale from the tile's own box, and its corner radius on screen */
interface End {tx: number; ty: number; sx: number; sy: number; r: number}
/** 0 before a, 1 after b, a straight line between */
const ramp = (t: number, a: number, b: number) => Math.max(0, Math.min(1, (t - a) / (b - a)));
const easeOut = (t: number) => 1 - Math.pow(1 - t, 3);
/** a value read off evenly spaced samples, t from 0 to 1 */
const track = (points: number[], t: number) => {
	const at = Math.max(0, Math.min(1, t)) * (points.length - 1);
	const i = Math.min(points.length - 2, Math.floor(at));
	return points[i] + (points[i + 1] - points[i]) * (at - i);
};
/** a list choice's height and the gap between choices, as .cc__opt in theme/panels.css */
const OPT_H = 72;
const OPT_GAP = 12;
const listHeight = (count: number) => count * OPT_H + (count - 1) * OPT_GAP;

/** the TV's values as last read or set, so the panel opens on them rather than on defaults that jump when the read lands */
let lastValues: Controls = {volume: 0, muted: false, picture: 'normal', sound: 'standard', backlight: 100, eyeComfort: false};
/** Read the TV's values ahead of the first open (app.tsx calls it 4 s after each return). */
export function prefetchControls (): void { readControls().then((v) => { lastValues = v; }).catch(() => undefined); }

type Phase = 'grid' | 'opening' | 'list' | 'closing';
interface Morph {kind: 'open' | 'close'; cell: MenuCell; geo: {x: number; y: number; w: number; h: number; fw: number; fh: number; tile: End; list: End}}

/** Grows a top-bar panel in from the button that opened it (the top-bar pill with data-top={key}): a short scale and fade, as tvOS does. */
export function growFrom (panel: HTMLElement | null, key: string, ms = 260): void {
	const button = document.querySelector(`.topbar [data-top="${key}"]`);
	if (!panel || !button || !panel.animate || document.documentElement.classList.contains('reduce-motion')) return;
	const b = button.getBoundingClientRect();
	const p = panel.getBoundingClientRect();
	panel.style.transformOrigin = `${b.left + b.width / 2 - p.left}px ${b.top + b.height / 2 - p.top}px`;
	const opts = {duration: ms, easing: 'cubic-bezier(0.2, 0.8, 0.2, 1)'};
	panel.animate([{transform: 'scale(0.6)'}, {transform: 'none'}], opts);
	// each control fades on its own: fading the panel would cut their live blur off from the wallpaper
	Array.from(panel.querySelectorAll<HTMLElement>('.cc__mod, .cc__caption')).forEach((el) => el.animate([{opacity: 0}, {opacity: 1}], opts));
}

/** Control Center (opened from the round button top right): sound, picture mode, appearance, clock, the screensaver (Lumen's), LG Home, screen off,
 *  restart, backlight and Eye Comfort. Volume is left to the remote's own buttons. Arrows move between controls, OK presses one, Left/Right on
 *  the backlight bar change it, Back closes, and Down twice on the bottom row closes and goes to the glass bar. */
export function ControlCenter ({onClose, onCloseDown, onError, onAction, darkMode, clockFormat, onDarkMode, onClockFormat}: Props) {
	const outer = useRef({darkMode, clockFormat, onDarkMode, onClockFormat});
	outer.current = {darkMode, clockFormat, onDarkMode, onClockFormat};
	const [values, setValues] = useState<Controls>(lastValues);
	useEffect(() => { lastValues = values; }, [values]);
	// x is the grid column the remote is on, kept when moving up and down, so Down from Restart lands on Eye Comfort
	const [pos, setPos] = useState({row: 0, x: 0});
	const [confirm, setConfirm] = useState(false);
	// the open list: which tile it belongs to and the focused choice
	const [menu, setMenu] = useState<{cell: MenuCell; idx: number} | null>(null);
	const [phase, setPhase] = useState<Phase>('grid');
	// the pill that stands in for the tile while the list opens or closes; the tile itself is hidden meanwhile
	const [morph, setMorph] = useState<Morph | null>(null);
	const state = useRef({values, pos, confirm, menu, phase});
	state.current = {values, pos, confirm, menu, phase};
	// when Down was last pressed with nowhere lower to go
	const bottomDown = useRef(0);
	const tiles = useRef<Partial<Record<CellId, HTMLElement | null>>>({});
	const panel = useRef<HTMLDivElement>(null);
	const timers = useRef<number[]>([]);
	useEffect(() => () => timers.current.forEach((id) => window.clearTimeout(id)), []);
	const still = () => document.documentElement.classList.contains('reduce-motion');
	const after = (ms: number, fn: () => void) => { timers.current.push(window.setTimeout(fn, still() ? 0 : ms)); };

	// longer than Inputs: the first frame lands late, so a 260 ms grow showed only half its frames (Sam chose 460 on the TV)
	useLayoutEffect(() => growFrom(panel.current, 'controls', 460), []);

	// read the TV's values a moment after the panel is on screen, so opening it is never held up by the TV answering
	useEffect(() => {
		const id = window.setTimeout(() => { readControls().then(setValues); }, 250);
		return () => window.clearTimeout(id);
	}, []);

	const fail = (what: string) => () => onError(`Couldn't change ${what}`);
	const patch = (p: Partial<Controls>) => setValues((v) => ({...v, ...p}));

	// the backlight level waiting to be sent; sent at once if the panel closes first
	const pendingBacklight = useRef<{id: number; level: number} | null>(null);
	const sendBacklight = () => {
		const p = pendingBacklight.current;
		if (!p) return;
		window.clearTimeout(p.id);
		pendingBacklight.current = null;
		setBacklight(p.level).catch(() => onError("Couldn't change the backlight"));
	};
	useEffect(() => sendBacklight, []);
	const backlight = (next: number) => {
		const level = clamp(next, 0, 100);
		patch({backlight: level});
		if (pendingBacklight.current) window.clearTimeout(pendingBacklight.current.id);
		pendingBacklight.current = {id: window.setTimeout(sendBacklight, BACKLIGHT_DELAY_MS), level};
	};

	const press = (cell: CellId) => {
		if (state.current.phase !== 'grid') return;
		const v = state.current.values;
		if (cell === 'eye') {
			const on = !v.eyeComfort;
			patch({eyeComfort: on}); setEyeComfort(on).catch(() => { patch({eyeComfort: !on}); onError("Couldn't change Eye Comfort"); });
		}
		else if (MENU_CELLS.indexOf(cell) >= 0) openMenu(cell as MenuCell);
		else if (cell === 'art') onAction('art');
		else if (cell === 'home') onAction('lg-home');
		else if (cell === 'screenoff') { onClose(); screenOff().catch(() => onError("Couldn't turn the screen off (it needs the Homebrew Channel)")); }
		else if (cell === 'restart') {
			if (!state.current.confirm) { setConfirm(true); return; }
			onClose(); restartTv().catch(() => onError("Couldn't restart the TV (it needs the Homebrew Channel)"));
		}
	};

	const options = (cell: MenuCell): {value: string; label: string}[] =>
		cell === 'sound' ? SOUND_MODES.map((m) => ({value: m, label: SOUND_LABEL[m] || m}))
			: cell === 'picture' ? PICTURE_MODES.map((m) => ({value: m, label: PICTURE_LABEL[m] || m}))
				: cell === 'dark' ? [{value: 'off', label: tr('Light')}, {value: 'on', label: tr('Dark')}, {value: 'auto', label: tr('Auto')}]
					: [{value: 'auto', label: tr('Auto')}, {value: '12h', label: tr('12-hour')}, {value: '24h', label: tr('24-hour')}];
	const chosen = (cell: MenuCell): string => {
		const v = state.current.values;
		const o = outer.current;
		return cell === 'sound' ? v.sound : cell === 'picture' ? v.picture : cell === 'dark' ? o.darkMode : o.clockFormat;
	};

	const apply = (cell: MenuCell, value: string) => {
		if (value === chosen(cell)) return;
		const v = state.current.values;
		if (cell === 'sound') {
			const before = v.sound;
			patch({sound: value}); setSoundMode(value).catch(() => { patch({sound: before}); onError("This TV didn't accept that sound mode"); });
		}
		else if (cell === 'picture') {
			// each picture mode has its own backlight, so show the new mode's
			patch({picture: value}); setPictureMode(value).then(readBacklight).then((b) => { if (b !== null) patch({backlight: b}); }).catch(fail('the picture mode'));
		}
		else if (cell === 'dark') outer.current.onDarkMode(value);
		else outer.current.onClockFormat(value);
	};

	/** Where the copy sits (the tile's own box, so its corners match) and the transforms for its two ends: the focused tile (scaled 1.1 about
	 *  its centre) and the whole list. */
	/** The pill's geometry for a tile and a list of count choices. The pill is laid out in the tile's own box, so its ends match the tile. */
	const geometry = (cell: MenuCell, count: number) => {
		const el = tiles.current[cell];
		const w = el ? el.offsetWidth : 1;
		const h = el ? el.offsetHeight : 1;
		const x = el ? el.offsetLeft : 0;
		const y = el ? el.offsetTop : 0;
		const full = panel.current ? panel.current.offsetWidth : w;
		const grown = (s: number): End => ({tx: ((1 - s) * w) / 2, ty: ((1 - s) * h) / 2, sx: s, sy: s, r: (h / 2) * s});
		return {
			x, y, w, h, fw: full, fh: listHeight(count),
			tile: grown(1.1), // focused tiles are drawn at 1.1
			// the pill ends PILL_PAD bigger than the list all round, with an option's corners
			list: {tx: -x - PILL_PAD, ty: -y - PILL_PAD, sx: (full + 2 * PILL_PAD) / w, sy: (listHeight(count) + 2 * PILL_PAD) / h, r: OPT_H / 2}
		};
	};

	/** The pill partway through opening. Its size follows tvOS's measured growth, but its height grows faster than its width, so it has the
	 *  list's shape by SHAPE_BY and the list fills it from early on. Each edge moves in step with its own growth, so the pill stays between the
	 *  tile and the list. */
	const opening = (g: Morph['geo'], t: number): End => {
		const k = track(GROW, t);
		const l0 = g.x + g.tile.tx, t0 = g.y + g.tile.ty, w0 = g.w * g.tile.sx, h0 = g.h * g.tile.sy;
		const l1 = g.x + g.list.tx, t1 = g.y + g.list.ty, w1 = g.w * g.list.sx, h1 = g.h * g.list.sy;
		const h = h0 + (h1 - h0) * k;
		const shape = w0 / h0 + (w1 / h1 - w0 / h0) * easeOut(ramp(t, 0, SHAPE_BY));
		const w = Math.max(Math.min(w0, w1), Math.min(Math.max(w0, w1), h * shape));
		const px = w1 === w0 ? k : (w - w0) / (w1 - w0);
		const left = l0 + (l1 - l0) * px;
		const top = t0 + (t1 - t0) * k;
		return {tx: left - g.x, ty: top - g.y, sx: w / g.w, sy: h / g.h, r: g.tile.r + (g.list.r - g.tile.r) * k};
	};

	/** Every layer's keyframes for one open or close, sampled from the same timeline. */
	const timeline = (m: Morph) => {
		const g = m.geo;
		const pill: Keyframe[] = [], glass: Keyframe[] = [], rim: Keyframe[] = [], text: Keyframe[] = [], list: Keyframe[] = [], listFade: Keyframe[] = [], grid: Keyframe[] = [], gridFade: Keyframe[] = [];
		for (let i = 0; i <= STEPS; i++) {
			const t = i / STEPS;
			const open = m.kind === 'open';
			// closing has no pill: the list shrinks back toward the tile as it fades, and the grid, tile included, fades back in
			const at = open ? opening(g, t) : opening(g, 1 - easeOut(ramp(t, 0, 0.7)));
			// the corners ease from the tile's round ends to an option's radius as the pill grows
			const r = at.r;
			pill.push({transform: `translate(${at.tx}px, ${at.ty}px) scale(${at.sx}, ${at.sy})`, borderRadius: `${r / at.sx}px / ${r / at.sy}px`});
			// the tile's text follows the pill's centre at its own size
			const dx = at.tx + (g.w * at.sx) / 2 - g.w / 2;
			const dy = at.ty + (g.h * at.sy) / 2 - g.h / 2;
			text.push({transform: `translate(${dx}px, ${dy}px) scale(1.1)`, opacity: 1 - ramp(t, 0.06, 0.27)});
			// opening: the glass is gone by 167 ms, while the list is still fading in and growing
			glass.push({opacity: 1 - ramp(t, 0.2, 0.5)});
			// the edge highlight and shadow grow with the pill's scale, so they go before it grows
			rim.push({opacity: 1 - ramp(t, 0, 0.1)});
			// the list sits inside the pill at the pill's size, as tvOS's does: scaled evenly to fit inside its padding, and centred
			const pw = g.w * at.sx;
			const ph = g.h * at.sy;
			const fit = Math.max(0, Math.min(1, (pw - 2 * PILL_PAD) / g.fw, (ph - 2 * PILL_PAD) / g.fh));
			const lx = g.x + at.tx + (pw - fit * g.fw) / 2;
			const ly = g.y + at.ty + (ph - fit * g.fh) / 2;
			// opacity goes on each piece of glass, not the list or grid: a fading parent would cut its live blur off from the wallpaper
			list.push({transform: `translate(${lx}px, ${ly}px) scale(${fit})`});
			listFade.push({opacity: open ? easeOut(ramp(t, 0.08, 0.8)) : 1 - ramp(t, 0.1, 0.7)});
			grid.push({transform: open ? `scale(${1 - GRID_SHRINK * easeOut(ramp(t, 0, 0.4))})` : `scale(${1 - GRID_SHRINK + GRID_SHRINK * easeOut(ramp(t, 0.2, 1))})`});
			gridFade.push({opacity: open ? 1 - ramp(t, 0.08, 0.4) : ramp(t, 0.25, 0.85)});
		}
		return {pill, glass, rim, text, list, listFade, grid, gridFade};
	};

	const pillEl = useRef<HTMLDivElement>(null);
	const glassEl = useRef<HTMLDivElement>(null);
	const rimEl = useRef<HTMLDivElement>(null);
	const textEl = useRef<HTMLDivElement>(null);
	const listEl = useRef<HTMLDivElement>(null);
	const gridEl = useRef<HTMLDivElement>(null);
	const running = useRef<Animation[]>([]);
	// start every layer in the same frame the pill is drawn, before the browser paints it
	useLayoutEffect(() => {
		if (!morph) return;
		running.current.forEach((a) => a.cancel());
		const k = timeline(morph);
		const ms = still() ? 0 : Math.round((morph.kind === 'open' ? OPEN_MS : CLOSE_MS) * (options(morph.cell).length > LONG_LIST ? LONG_STRETCH : 1));
		const opts: KeyframeAnimationOptions = {duration: ms, easing: 'linear', fill: 'both'};
		const open = morph.kind === 'open';
		const tile = tiles.current[morph.cell];
		const each = (root: HTMLElement | null, sel: string, frames: Keyframe[]) => (root ? Array.from(root.querySelectorAll<HTMLElement>(sel)) : []).map((el): [HTMLElement, Keyframe[]] => [el, frames]);
		const go = (pairs: [HTMLElement | null | undefined, Keyframe[]][]) => pairs.filter(([el]) => el && el.animate).map(([el, frames]) => (el as HTMLElement).animate(frames, opts));
		const moving = go([[pillEl.current, k.pill], [glassEl.current, k.glass], [rimEl.current, k.rim], [textEl.current, k.text], [listEl.current, k.list], [gridEl.current, k.grid]]);
		// opening, the tile is hidden from the first frame, as the pill stands in for it
		const fading = go([...each(listEl.current, '.cc__opt', k.listFade), ...each(gridEl.current, '.cc__mod, .cc__caption', k.gridFade).filter(([el]) => !(open && el === tile)),
			...(open ? [[tile, [{opacity: 0}, {opacity: 0}]] as [HTMLElement | undefined, Keyframe[]]] : [])]);
		running.current = moving.concat(fading);
		const grid = gridEl.current;
		if (grid) grid.style.transformOrigin = `${morph.geo.x + morph.geo.w / 2}px ${morph.geo.y + morph.geo.h / 2}px`;
		after(ms + 20, () => {
			if (open) { setPhase('list'); setMorph(null); }
			else { setMorph(null); setMenu(null); setPhase('grid'); }
			// the fades keep holding the grid hidden behind the open list: hiding it by class repaints every control (docs/frame-rate.md)
			moving.forEach((a) => a.cancel());
			if (!open) fading.forEach((a) => a.cancel());
			running.current = open ? fading : [];
		});
	}, [morph]);

	const openMenu = (cell: MenuCell) => {
		if (state.current.phase !== 'grid') return;
		const list = options(cell);
		const idx = Math.max(0, list.map((o) => o.value).indexOf(chosen(cell)));
		setConfirm(false); setMenu({cell, idx}); setPhase('opening');
		setMorph({kind: 'open', cell, geo: geometry(cell, list.length)});
	};

	/** Back to the grid, after applying the choice if one was picked. */
	const closeMenu = (pick: boolean, idx?: number) => {
		const {menu: m, phase: ph} = state.current;
		if (!m || ph !== 'list') return;
		if (pick) apply(m.cell, options(m.cell)[idx === undefined ? m.idx : idx].value);
		setPhase('closing');
		setMorph({kind: 'close', cell: m.cell, geo: geometry(m.cell, options(m.cell).length)});
	};

	const move = (row: number, x: number) => { setPos({row, x}); setConfirm(false); bottomDown.current = 0; };

	const handleKey = useCallback((key: NavKey): boolean => {
		const {pos: p, values: v, menu: m, phase: ph} = state.current;
		if (ph === 'opening' || ph === 'closing') return true;
		if (m) {
			if (key === 'back') closeMenu(false);
			else if (key === 'enter') closeMenu(true);
			else if (key === 'up' || key === 'down') setMenu({cell: m.cell, idx: clamp(m.idx + (key === 'up' ? -1 : 1), 0, options(m.cell).length - 1)});
			return true;
		}
		const {cell, start} = cellAt(p.row, p.x);
		if (key === 'back') { onClose(); return true; }
		if (key === 'enter') { press(cell); return true; }
		if (cell === 'backlight' && (key === 'left' || key === 'right')) { backlight(v.backlight + (key === 'left' ? -5 : 5)); return true; }
		if (key === 'left' || key === 'right') {
			const i = GRID[p.row].map((c) => c.id).indexOf(cell);
			const j = clamp(i + (key === 'left' ? -1 : 1), 0, GRID[p.row].length - 1);
			if (j !== i) move(p.row, startOf(p.row, j));
		}
		else if (key === 'up' && p.row === 0) onClose();   // Up past the top row closes, back to the top buttons
		else if (key === 'down' && p.row === GRID.length - 1) {
			// someone heading for the glass bar may not see the panel is open, so a second Down here soon after the first takes them there
			const now = Date.now();
			if (now - bottomDown.current < DOUBLE_DOWN_MS) onCloseDown();
			else bottomDown.current = now;
		}
		else if (key === 'up' || key === 'down') move(clamp(p.row + (key === 'up' ? -1 : 1), 0, GRID.length - 1), start);
		return true;
	}, []);
	useKeys(handleKey, true);

	const current = cellAt(pos.row, pos.x).cell;
	const focus = (cell: CellId) => () => {
		// the pointer can still reach the grid behind an open list, which is hidden by its fades, not pointer-events
		if (state.current.phase !== 'grid') return;
		for (let r = 0; r < GRID.length; r++) { const i = GRID[r].map((c) => c.id).indexOf(cell); if (i >= 0) { setPos({row: r, x: startOf(r, i)}); return; } }
	};

	/** What a list tile shows: icon, name, current choice, and whether it shows as on. */
	const info = (cell: MenuCell): {glyph: GlyphName; title: string; sub: string; on: boolean} =>
		cell === 'sound' ? {glyph: 'speaker', title: tr('Sound'), sub: SOUND_LABEL[values.sound] || values.sound, on: false}
			: cell === 'picture' ? {glyph: 'picture', title: tr('Picture'), sub: PICTURE_LABEL[values.picture] || values.picture, on: false}
				: cell === 'dark' ? {glyph: 'screensaver', title: tr('Theme'), sub: darkMode === 'on' ? tr('Dark') : darkMode === 'auto' ? tr('Auto') : tr('Light'), on: darkMode === 'on'}
					: {glyph: 'timer', title: tr('Clock'), sub: clockFormat === '12h' ? tr('12-hour') : clockFormat === '24h' ? tr('24-hour') : tr('Auto'), on: clockFormat !== 'auto'};
	const tileInside = (i: {glyph: GlyphName; title: string; sub: string}) => [
		<div key="i" class="cc__icon"><Glyph name={i.glyph} /></div>,
		<div key="t" class="cc__text"><div class="cc__name">{i.title}</div><div class="cc__sub">{i.sub}</div></div>
	];
	const wide = (cell: MenuCell) => {
		const i = info(cell);
		return (
			<div
				key={cell}
				ref={(el) => { tiles.current[cell] = el; }}
				class={`cc__mod cc__tile${current === cell ? ' cc__mod--focused' : ''}${i.on ? ' cc__mod--on' : ''}`}
				onMouseEnter={focus(cell)}
				onClick={() => press(cell)}
			>
				{tileInside(i)}
			</div>
		);
	};
	// icon-only circles, as on tvOS; the focused one is named in the caption under the panel
	const CIRCLES: {cell: CellId; glyph: GlyphName; label: string}[] = [
		{cell: 'home', glyph: 'house', label: 'LG Home'}, {cell: 'art', glyph: 'image', label: 'Screensaver'},
		{cell: 'screenoff', glyph: 'power', label: 'Screen off'}, {cell: 'restart', glyph: 'restart', label: 'Restart'},
		{cell: 'eye', glyph: 'eye', label: 'Eye Comfort'}
	];
	const circle = (cell: CellId, on = false) => {
		const c = CIRCLES.filter((q) => q.cell === cell)[0];
		return <div key={cell} class={`cc__mod cc__circle${current === cell ? ' cc__mod--focused' : ''}${on ? ' cc__mod--on' : ''}`} onMouseEnter={focus(cell)} onClick={() => press(cell)}><Glyph name={c.glyph} /></div>;
	};
	const named = CIRCLES.filter((q) => q.cell === current)[0];
	const caption = !named || menu ? '' : named.cell === 'restart' && confirm ? tr('Press OK again') : named.cell === 'eye' ? `${tr(named.label)}: ${tr(values.eyeComfort ? 'On' : 'Off')}` : tr(named.label);
	// the backlight label, drawn twice: light over the glass, dark over the filled part, so it reads at any level
	const sliderText = <div class="cc__slidertext"><Glyph name="sun" /><span>{tr('Backlight')}</span><span class="cc__num">{values.backlight}</span></div>;

	return (
		<>
			<div class="panel-backdrop panel-backdrop--clear" onClick={onClose} />
			<div class="panel panel--cc" ref={panel}>
				<div ref={gridEl} class="cc__grid">
				<div class="cc__row">
					{wide('sound')}
					{wide('picture')}
				</div>
				<div class="cc__row">
					{wide('dark')}
					{wide('clock')}
				</div>
				<div class="cc__row">{circle('art')}{circle('home')}{circle('screenoff')}{circle('restart')}</div>
				<div class="cc__row">
					<div
						class={`cc__mod cc__slider${current === 'backlight' ? ' cc__mod--focused' : ''}`}
						onMouseEnter={focus('backlight')}
						onClick={(e: MouseEvent) => { if (state.current.phase !== 'grid') return; const box = (e.currentTarget as HTMLElement).getBoundingClientRect(); backlight(Math.round(((e.clientX - box.left) / box.width) * 100)); }}
					>
						{sliderText}
						<div class="cc__fill" style={`width:${values.backlight}%`}>{sliderText}</div>
					</div>
					{circle('eye', values.eyeComfort)}
				</div>
				<div class="cc__caption">{caption}</div>
				</div>
				{morph && morph.kind === 'open' && (() => {
					const box = `left:${morph.geo.x}px;top:${morph.geo.y}px;width:${morph.geo.w}px;height:${morph.geo.h}px`;
					const i = info(morph.cell);
					return [
						<div key="pill" ref={pillEl} class="cc__morph" style={box}>
							<div ref={glassEl} class="cc__mod cc__mod--focused cc__morph__glass" />
							<div ref={rimEl} class="cc__morph__rim" />
						</div>,
						<div key="text" ref={textEl} class={`cc__tile cc__morph__text${i.on ? ' cc__mod--on' : ''}`} style={box}>{tileInside(i)}</div>
					];
				})()}
				{menu && (
					<div ref={listEl} class="cc__list">
						{options(menu.cell).map((o, i) => (
							<div
								key={o.value}
								class={`cc__mod cc__tile cc__opt${menu.idx === i ? ' cc__mod--focused' : ''}${o.value === chosen(menu.cell) ? ' cc__mod--on' : ''}`}
								onMouseEnter={() => { if (state.current.phase === 'list') setMenu({cell: menu.cell, idx: i}); }}
								onClick={() => closeMenu(true, i)}
							>
								<div class="cc__icon"><Glyph name={MENU_GLYPH[menu.cell]} /></div>
								<div class="cc__text"><div class="cc__name">{o.label}</div></div>
							</div>
						))}
					</div>
				)}
			</div>
		</>
	);
}
