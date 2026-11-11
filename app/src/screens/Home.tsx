import {t as tr} from '../lib/i18n';
import type {Weather} from '../lib/weather';
import {VideoCaption} from '../components/VideoCaption';
import {LowerVeil} from '../components/LowerVeil';
import {useCallback, useEffect, useLayoutEffect, useRef, useState} from 'preact/hooks';
import {Glyph, type GlyphName} from '../components/Glyph';
import {canUninstall} from '../lib/root';
import {growFrom, DOUBLE_DOWN_MS} from '../components/ControlCenter';
import {Row} from '../components/Row';
import {Tile, type TileModel} from '../components/Tile';
import {useKeys, clamp, type NavKey} from '../hooks/useKeys';
import {useOkPress} from '../hooks/useOkPress';
import {Status} from '../components/Status';
import {Forecast} from '../components/Forecast';
import {BUILTIN} from '../config/builtinApps';
import {bootMark} from '../lib/log';
import {TIMING} from '../config/constants';

export interface HomeRow {
	id: string;
	label: string;
	items: TileModel[];
	emptyText?: string;
	/** show the row's name above it (the Recent row); the app rows have none, as on tvOS */
	titled?: boolean;
	/** extra class on the row (row--live) */
	className?: string;
	/** how much higher the row must sit when focused, to leave room under it (the Live TV row's now/next block) */
	below?: number;
	/** see Row's slot */
	slot?: number;
}

interface Props {
	rows: HomeRow[];
	/** the top bar: Control Center, Settings and Inputs */
	topItems: TileModel[];
	/** what the Inputs panel lists */
	inputs: TileModel[];
	active: boolean;
	onActivate: (rowId: string, item: TileModel) => void;
	/** show AM/PM beside the time (12-hour clocks) */
	showPeriod?: boolean;
	/** hold OK on an app: move it along the rows, or into or out of a folder */
	onEdit?: (action: 'move' | 'remove' | 'uninstall' | 'folder-add' | 'folder-out', rowId: string, key: string, arg?: number | string) => void;
	/** the apps inside each folder, by folder name */
	folders?: Record<string, TileModel[]>;
	/** shown beside the date under the clock, top left */
	weather?: Weather | null;
	/** the weather on the line under the clock (Settings > Home Screen > Date and weather, and not when the large widget shows it) */
	weatherLine?: boolean;
	/** the weather widget above the dock, and its size */
	forecast?: 'small' | 'large';
	/** off | down | idle | both: when the dock slides away */
	dockHide?: string;
	/** darken the wallpaper while the dock is hidden */
	dockDim?: boolean;
	captions?: boolean;
	/** put the focus on this row (a launch with {"view":"recent"}); a new seq asks again */
	focusRow?: {id: string; seq: number};
	/** tile size on the rows (Settings > Apps in the glass bar: 7 apps need smaller tiles); the first row is `perRow` tiles wide */
	tiles?: {size: number; height: number; gap: number};
	perRow?: number;
}


/** the rows whose apps can be moved or removed: the glass bar and the grid rows ('dock', 'dock:1'...) */
const editable = (rowId: string) => rowId === 'apps' || rowId.indexOf('dock') === 0;
/** where the glass bar's row goes when the remote is down in the grid: near the top of the screen, as on tvOS (its tiles sit 40px lower) */
const GRID_TOP = 76;
/** the lowest a focused grid row may sit before the grid scrolls on (its tiles 40px lower) */
const GRID_BOTTOM = 780;   // 40px less than it looks: the grid rows sit 40px lower while the remote is in them (theme/home.css)
const FOLDER_PRESETS = ['Games', 'Tools', 'Media'];
const FOLDER_PREFIX = 'folder:';
/** the remote's 1 to 6 open the glass bar's apps at those places (keyCodes for the digit row and the number pad) */
const dockDigit = (e: KeyboardEvent) => (e.keyCode >= 49 && e.keyCode <= 54 ? e.keyCode - 48 : e.keyCode >= 97 && e.keyCode <= 102 ? e.keyCode - 96 : 0);

/** one choice in the hold-OK menu */
interface CtxItem { id: string; label: string; glyph: GlyphName }
/** the menu's pills: wider than the Inputs list's, for "Press OK again to uninstall"; 72px tall with 12px between (.cc__opt) */
const CTX_W = 400;
const CTX_ITEM_H = 84;
const CTX_GAP = 24;
const CTX_EDGE = 60;

/** the menu rises from the app (drops, when under it): the panel moves and each pill fades on its own, as growFrom does for Inputs, since fading
 *  the panel would cut the pills' live blur off from the wallpaper; nothing is left applied afterwards */
function riseFrom (panel: HTMLElement, below: boolean): void {
	if (!panel.animate || document.documentElement.classList.contains('reduce-motion')) return;
	const opts = {duration: 200, easing: 'cubic-bezier(0.2, 0.8, 0.2, 1)'};
	panel.style.transformOrigin = below ? '50% 0' : '50% 100%';
	panel.animate([{transform: `translateY(${below ? -14 : 14}px) scale(0.96)`}, {transform: 'none'}], opts);
	Array.from(panel.querySelectorAll<HTMLElement>('.cc__mod')).forEach((el) => el.animate([{opacity: 0}, {opacity: 1}], opts));
}

/** Where the remote's focus is: a row of apps, the top bar, or one of the two open panels. */
type Zone = 'rows' | 'top' | 'panel' | 'ctx' | 'folder';

interface Nav {
	/** false = resting: nothing is focused until the remote (or the pointer) is used */
	engaged: boolean;
	zone: Zone;
	rowIndex: number;
	columns: Record<string, number>;
	topIndex: number;
	panelIndex: number;
	ctxIndex: number;
	/** the key of the app being moved with the arrows, or null */
	moving: string | null;
	/** the OK that brought the focus up must not also open the app on release */
	swallowOk: boolean;
	/** a press-and-hold with the pointer just opened the menu: ignore the click that follows */
	suppressClick: boolean;
	/** the open folder (its name) and the focused app in it */
	folder: string;
	folderIndex: number;
	/** where the hold-OK menu was opened from, and where it returns to */
	ctxFrom: 'rows' | 'folder';
	/** the menu's first list, or its list of folders */
	ctxLevel: 'main' | 'folder';
	/** Uninstall was picked once: the next OK uninstalls */
	ctxConfirm: boolean;
	/** the lifted tile's place on screen when the menu opened, so the menu sits over it; null for a menu inside a folder */
	ctxAnchor: {x: number; top: number; bottom: number} | null;
	/** who moved the focus last: leaving the dock with the pointer rests the icons, with the keys the focus stays */
	mode: 'key' | 'pointer';
	/** the dock is slid away; any key brings it back */
	dockHidden: boolean;
	/** where the Inputs panel was opened from (the top bar button or the Inputs tile), and where Back returns to */
	panelFrom: 'top' | 'rows';
}

const PANEL_COLUMNS = 3;
const INPUTS_KEY = 'inputs';

/** The home screen, navigated like tvOS:
 *  - Left/Right move along a row; Up/Down move between rows (same column, as far as that row reaches).
 *  - Up from the first row reaches the top bar (Control Center, Settings, Inputs); Down goes back to the same app.
 *  - OK on Inputs opens the inputs panel; Back closes it.
 *  - The second row stays hidden until the remote goes down to it. */
export function Home ({rows, topItems, inputs, active, onActivate, showPeriod, onEdit, folders, weather, weatherLine, forecast, dockHide, dockDim, captions = true, focusRow, tiles, perRow = 6}: Props) {
	const nav = useRef<Nav>({engaged: false, zone: 'rows', rowIndex: 0, columns: {}, topIndex: 0, panelIndex: 0, ctxIndex: 0, moving: null, swallowOk: false, suppressClick: false, folder: '', folderIndex: 0, ctxFrom: 'rows', ctxLevel: 'main', ctxConfirm: false, ctxAnchor: null, mode: 'key', dockHidden: false, panelFrom: 'top'});
	// when Down was last pressed at the last input, with nowhere lower to go
	const bottomDown = useRef(0);
	const latest = useRef({rows, topItems, inputs, onActivate, onEdit, folders, dockHide, perRow});
	latest.current = {rows, topItems, inputs, onActivate, onEdit, folders, dockHide, perRow};
	// false while the browser only reports a mouse-enter because the rows slid under a resting pointer (webOS 25: Down bounced back to the bar)
	const realPointer = useRef(true);
	useEffect(() => {
		let lastX = -1, lastY = -1;
		const track = (e: MouseEvent) => { realPointer.current = e.clientX !== lastX || e.clientY !== lastY; lastX = e.clientX; lastY = e.clientY; };
		window.addEventListener('mouseover', track, true);
		window.addEventListener('mousemove', track, true);
		return () => { window.removeEventListener('mouseover', track, true); window.removeEventListener('mousemove', track, true); };
	}, []);
	const [late, setLate] = useState(false);   // the rows below the first are drawn a moment after launch
	useEffect(() => { if (late) bootMark('grid'); }, [late]);
	useEffect(() => { const id = window.setTimeout(() => { bootMark('grid-start'); setLate(true); }, 3500); return () => window.clearTimeout(id); }, []);
	const [, redraw] = useState(0);
	const render = () => redraw((n) => n + 1);

	// a launch asked for a row: focus it as soon as it exists (the Recent row only appears once the app list has loaded)
	const wanted = useRef<{id: string; until: number} | null>(null);
	const focusWanted = (): boolean => {
		const w = wanted.current;
		if (!w || Date.now() > w.until) { wanted.current = null; return false; }
		const r = latest.current.rows.findIndex((row) => row.id === w.id);
		if (r < 0) return false;
		const n = nav.current;
		n.engaged = true; n.mode = 'key'; n.zone = 'rows'; n.rowIndex = r; n.columns[w.id] = 0; n.dockHidden = false;
		setLate(true);
		render();
		return true;
	};
	useEffect(() => {
		if (!focusRow) return;
		wanted.current = {id: focusRow.id, until: Date.now() + 8000};
		focusWanted();
	}, [focusRow && focusRow.seq]);
	useEffect(() => { if (wanted.current) focusWanted(); }, [rows]);

	// coming back from another app stays where it left, as the TV shows Lumen's last frame first and a reset would jump (docs/research/startup-perf.md); a launch that asks for a row still goes there
	useEffect(() => {
		const back = () => { if (!document.hidden) focusWanted(); };
		document.addEventListener('visibilitychange', back);
		return () => document.removeEventListener('visibilitychange', back);
	}, []);

	// the idle timer follows the option; moving the pointer brings a hidden dock back
	useEffect(() => {
		const m = nav.current;
		if (dockHide === 'off' || dockHide === 'idle' || !dockHide) { if (dockHide !== 'idle' && m.dockHidden) { m.dockHidden = false; render(); } }
		armIdle();
		const show = () => { const k = nav.current; if (k.dockHidden) { k.dockHidden = false; armIdle(); render(); } };
		window.addEventListener('mousemove', show);
		return () => { window.removeEventListener('mousemove', show); window.clearTimeout(idleTimer.current); };
	}, [dockHide]);

	// while an app is being moved, keep the focus on it as it changes places (it may cross from one row to the next)
	useEffect(() => {
		const n = nav.current;
		if (!n.moving) return;
		for (let r = 0; r < rows.length; r++) {
			const c = rows[r].items.findIndex((t) => t.key === n.moving);
			if (c >= 0) { n.rowIndex = r; n.columns[rows[r].id] = c; render(); return; }
		}
	}, [rows]);

	// a folder that has become empty closes
	useEffect(() => {
		const n = nav.current;
		if (n.zone === 'folder' && !((folders || {})[n.folder] || []).length) { n.zone = 'rows'; render(); }
	}, [folders]);

	const columnOf = (row: HomeRow) => clamp(nav.current.columns[row.id] || 0, 0, Math.max(0, row.items.length - 1));

	const activateTop = (i: number) => {
		const n = nav.current;
		const item = latest.current.topItems[i];
		if (!item) return;
		if (item.key === INPUTS_KEY) { n.panelIndex = 0; n.panelFrom = 'top'; n.zone = 'panel'; render(); }
		else latest.current.onActivate('top', item);
	};
	const activatePanel = (i: number) => {
		const item = latest.current.inputs[i];
		if (!item) return;
		nav.current.zone = nav.current.panelFrom;
		render();
		latest.current.onActivate('inputs', item);
	};
	/** what the hold-OK menu offers now: inside a folder, only taking the app out; on a lifted app, Uninstall (apps the TV lets go only), Remove and Add to Folder */
	const ctxItems = (): CtxItem[] => {
		const n = nav.current;
		if (n.ctxFrom === 'folder') return [{id: 'out', label: 'Take out of Folder', glyph: 'folder'}];
		if (n.ctxLevel === 'folder') return FOLDER_PRESETS.map((f): CtxItem => ({id: FOLDER_PREFIX + f, label: f, glyph: 'folder'}));
		const list: CtxItem[] = [];
		if (n.moving && canUninstall(n.moving)) list.push({id: 'uninstall', label: n.ctxConfirm ? 'Press OK again to uninstall' : 'Uninstall App', glyph: 'trash'});
		return list.concat([{id: 'remove', label: 'Remove from Home Screen', glyph: 'minus'}, {id: 'folder', label: 'Add to Folder', glyph: 'folder'}]);
	};
	const activateCtx = (i: number) => {
		const n = nav.current;
		setLate(true);
		const {rows: allRows, folders: fl, onEdit: edit} = latest.current;
		if (n.ctxFrom === 'folder') {
			const inside = ((fl || {})[n.folder] || [])[n.folderIndex];
			n.zone = 'folder';
			if (inside && edit) edit('folder-out', n.folder, inside.key);
			render();
			return;
		}
		const pick = ctxItems()[i];
		const key = n.moving;
		const row = allRows[clamp(n.rowIndex, 0, allRows.length - 1)];
		if (!pick || !key || !row) return;
		if (pick.id === 'folder') { n.ctxLevel = 'folder'; n.ctxIndex = 0; render(); return; }
		// uninstalling cannot be undone, so it takes a second OK
		if (pick.id === 'uninstall' && !n.ctxConfirm) { n.ctxConfirm = true; render(); return; }
		n.zone = 'rows';
		n.moving = null;
		if (edit) {
			if (pick.id === 'uninstall') edit('uninstall', row.id, key);
			else if (pick.id === 'remove') { edit('remove', row.id, key); n.columns[row.id] = clamp(columnOf(row), 0, Math.max(0, row.items.length - 2)); }
			else edit('folder-add', row.id, key, pick.id.slice(FOLDER_PREFIX.length));
		}
		render();
	};
	/** opens the hold-OK menu over the lifted app (or in the middle, inside a folder) */
	const openCtx = (from: 'rows' | 'folder') => {
		const n = nav.current;
		const tile = from === 'rows' && rowsRef.current ? rowsRef.current.querySelector('.tile--focused') : null;
		const r = tile && tile.getBoundingClientRect();
		n.ctxAnchor = r ? {x: r.left + r.width / 2, top: r.top, bottom: r.bottom} : null;
		n.ctxFrom = from; n.ctxLevel = 'main'; n.ctxConfirm = false; n.ctxIndex = 0; n.zone = 'ctx';
		render();
	};

	/** OK on a tile in a row: a folder opens, anything else is handed to the app */
	const activateRowItem = (rowId: string, item: TileModel) => {
		const n = nav.current;
		if (item.key.indexOf(FOLDER_PREFIX) === 0) { n.folder = item.key.slice(FOLDER_PREFIX.length); n.folderIndex = 0; n.zone = 'folder'; render(); return; }
		if (item.key === BUILTIN.inputs) { n.panelIndex = 0; n.panelFrom = 'rows'; n.zone = 'panel'; render(); return; }
		latest.current.onActivate(rowId, item);
	};

	/** a short OK press: open / activate whatever is focused */
	const press = () => {
		const n = nav.current;
		if (n.swallowOk) { n.swallowOk = false; return; }
		if (!n.engaged) return;
		const {rows: allRows, topItems: tops, onActivate: act} = latest.current;
		if (n.zone === 'panel') activatePanel(n.panelIndex);
		else if (n.zone === 'ctx') activateCtx(n.ctxIndex);
		else if (n.zone === 'folder') {
			const inside = ((latest.current.folders || {})[n.folder] || [])[n.folderIndex];
			if (inside) { n.zone = 'rows'; render(); act('folder', inside); }
		}
		else if (n.zone === 'top') { if (tops[n.topIndex]) activateTop(n.topIndex); }
		else if (n.moving) { n.moving = null; render(); }
		else {
			const row = allRows[clamp(n.rowIndex, 0, allRows.length - 1)];
			const item = row && row.items[columnOf(row)];
			if (item) activateRowItem(row.id, item);
		}
	};

	/** holding OK on an app lifts it to move, as on tvOS; holding OK again while it is lifted opens its menu (uninstall, remove, folder) */
	const longPress = () => {
		const n = nav.current;
		if (n.engaged && n.zone === 'folder') { openCtx('folder'); return; }
		if (!n.engaged || n.zone !== 'rows') return;
		if (n.moving) { openCtx('rows'); return; }
		const {rows: allRows} = latest.current;
		const row = allRows[clamp(n.rowIndex, 0, allRows.length - 1)];
		const held = row && row.items[columnOf(row)];
		if (!row || !editable(row.id) || !held || held.key.indexOf(FOLDER_PREFIX) === 0) return;
		n.moving = held.key; setLate(true); render();
	};
	/** a hold does something only on an app in the bar or grid (lift, then its menu) and on an app in a folder (its menu); a resting screen keeps key up, so the OK that brings the focus up is swallowed as before */
	const canHold = () => {
		const n = nav.current;
		if (!n.engaged || n.zone === 'folder') return true;
		if (n.zone !== 'rows') return false;
		if (n.moving) return true;
		const {rows: allRows} = latest.current;
		const row = allRows[clamp(n.rowIndex, 0, allRows.length - 1)];
		const item = row && row.items[columnOf(row)];
		return !!row && editable(row.id) && !!item && item.key.indexOf(FOLDER_PREFIX) !== 0;
	};
	useOkPress({onPress: press, onLongPress: longPress, canHold}, active);

	const holdTimer = useRef(0);
	const holdStart = () => {
		window.clearTimeout(holdTimer.current);
		holdTimer.current = window.setTimeout(() => { const n = nav.current; n.suppressClick = true; longPress(); }, TIMING.longPressMs);
	};
	const holdCancel = () => window.clearTimeout(holdTimer.current);

	const pointerLeft = () => {
		holdCancel();
		const n = nav.current;
		if (n.mode === 'pointer' && n.zone !== 'panel') { n.engaged = false; render(); }
	};

	const idleTimer = useRef(0);
	const armIdle = () => {
		window.clearTimeout(idleTimer.current);
		const mode = latest.current.dockHide;
		if (mode !== 'idle' && mode !== 'both') return;
		idleTimer.current = window.setTimeout(() => {
			const m = nav.current;
			if (m.zone === 'rows' && !m.moving && !document.hidden) { m.dockHidden = true; render(); }
		}, 30000);
	};

	const handleKey = useCallback((key: NavKey): boolean => {
		const n = nav.current;
		setLate(true);
		const {rows: allRows, topItems: tops, inputs: ins, onEdit: edit, folders: fl} = latest.current;
		n.mode = 'key';
		const hideMode = latest.current.dockHide;
		if (n.dockHidden) { n.dockHidden = false; armIdle(); render(); return true; }   // the key that brings the dock back does nothing else
		armIdle();
		if (!n.engaged) { n.engaged = true; if (key === 'enter') n.swallowOk = true; render(); return true; }   // the first press only brings the focus up
		if (key === 'enter') return true;   // handled by useOkPress (press vs hold)

		if (n.zone === 'panel') {
			const last = ins.length - 1;
			switch (key) {
				case 'up': if (n.panelIndex === 0) n.zone = n.panelFrom; else n.panelIndex--; break;   // Up past the first input closes the list
				case 'down':
					if (n.panelIndex < last) { n.panelIndex++; bottomDown.current = 0; }
					// a second Down at the last input soon after the first closes the list and goes to the glass bar, as in Control Center
					else if (Date.now() - bottomDown.current < DOUBLE_DOWN_MS) { n.zone = 'rows'; n.rowIndex = 0; bottomDown.current = 0; }
					else bottomDown.current = Date.now();
					break;
				case 'back': n.zone = n.panelFrom; break;
			}
			render();
			return true;
		}
		if (n.zone === 'folder') {
			const last = ((fl || {})[n.folder] || []).length - 1;
			switch (key) {
				case 'left': n.folderIndex = clamp(n.folderIndex - 1, 0, last); break;
				case 'right': n.folderIndex = clamp(n.folderIndex + 1, 0, last); break;
				case 'up': if (n.folderIndex - PANEL_COLUMNS >= 0) n.folderIndex -= PANEL_COLUMNS; break;
				case 'down': if (n.folderIndex + PANEL_COLUMNS <= last) n.folderIndex += PANEL_COLUMNS; break;
				case 'back': n.zone = 'rows'; break;
			}
			render();
			return true;
		}
		if (n.zone === 'ctx') {
			const count = ctxItems().length;
			switch (key) {
				case 'up': n.ctxIndex = clamp(n.ctxIndex - 1, 0, count - 1); n.ctxConfirm = false; break;
				case 'down': n.ctxIndex = clamp(n.ctxIndex + 1, 0, count - 1); n.ctxConfirm = false; break;
				// Back from the folders returns to the first list, on Add to Folder; from the first list the app stays lifted
				case 'back': if (n.ctxLevel === 'folder') { n.ctxLevel = 'main'; n.ctxIndex = ctxItems().length - 1; } else n.zone = n.ctxFrom; break;
			}
			render();
			return true;
		}
		if (n.zone === 'top') {
			switch (key) {
				case 'left': n.topIndex = clamp(n.topIndex - 1, 0, tops.length - 1); break;
				case 'right': n.topIndex = clamp(n.topIndex + 1, 0, tops.length - 1); break;
				case 'down': case 'back': n.zone = 'rows'; break;
			}
			render();
			return true;
		}

		const rowIndex = clamp(n.rowIndex, 0, allRows.length - 1);
		const row = allRows[rowIndex];
		if (!row) return false;
		// like tvOS, moving to another row keeps the same column (as far as that row reaches)
		const moveRow = (next: number) => {
			const target = allRows[next];
			if (!target) return;
			n.columns[target.id] = clamp(columnOf(row), 0, target.items.length - 1);
			n.rowIndex = next;
		};
		if (n.moving) {
			if (key === 'left' || key === 'right') { if (edit) edit('move', row.id, n.moving, key === 'left' ? -1 : 1); return true; }
			// a whole row up or down: the rows are one list cut every perRow apps
			if (key === 'up' || key === 'down') { if (edit) edit('move', row.id, n.moving, (key === 'up' ? -1 : 1) * latest.current.perRow); return true; }
			if (key === 'back') n.moving = null;
			render();
			return true;
		}
		switch (key) {
			case 'up': if (rowIndex === 0) n.zone = 'top'; else moveRow(rowIndex - 1); break;
			case 'down':
				if (rowIndex >= allRows.length - 1 && (hideMode === 'down' || hideMode === 'both')) n.dockHidden = true;
				else moveRow(Math.min(allRows.length - 1, rowIndex + 1));
				break;
			case 'left': n.columns[row.id] = clamp(columnOf(row) - 1, 0, row.items.length - 1); break;
			case 'right': n.columns[row.id] = clamp(columnOf(row) + 1, 0, row.items.length - 1); break;
			// Back in the grid closes it, back to the glass bar's app it left from; on the bar it does nothing
			case 'back': if (rowIndex === 0) return true; n.rowIndex = 0; break;
		}
		render();
		return true;
	}, []);

	useKeys(handleKey, active);

	// number keys open the glass bar's apps; 7 to 9 are left alone, since the grid's places cannot be seen from the bar
	useEffect(() => {
		if (!active) return;
		const onDigit = (e: KeyboardEvent) => {
			const d = dockDigit(e);
			const n = nav.current;
			if (!d || n.moving || (n.zone !== 'rows' && n.zone !== 'top')) return;
			const bar = latest.current.rows[0];
			const item = bar && bar.items[d - 1];
			if (!item) return;
			e.preventDefault();
			n.engaged = true; n.mode = 'key'; n.zone = 'rows'; n.rowIndex = 0; n.columns[bar.id] = d - 1; n.dockHidden = false;
			render();
			activateRowItem(bar.id, item);
		};
		window.addEventListener('keydown', onDigit);
		return () => window.removeEventListener('keydown', onDigit);
	}, [active]);

	const n = nav.current;
	const rowIndex = clamp(n.rowIndex, 0, Math.max(0, rows.length - 1));
	// the grid stays open under the hold-OK menu of an app in it
	const lower = (n.zone === 'rows' || (n.zone === 'ctx' && n.ctxFrom === 'rows')) && rowIndex >= 1;
	// going down into the grid slides everything up until the glass bar is near the top of the screen with the grid under it, as on tvOS;
	// a focused row lower than GRID_BOTTOM scrolls the grid on
	const rowsRef = useRef<HTMLDivElement>(null);
	useLayoutEffect(() => {
		const el = rowsRef.current;
		if (!el) return;
		const kids = el.children as HTMLCollectionOf<HTMLElement>;
		const tilesTop = (row: HTMLElement) => el.offsetTop + row.offsetTop + ((row.querySelector('.row__viewport') as HTMLElement | null)?.offsetTop || 0);
		let shift = 0;
		if (lower && kids[0] && kids[rowIndex]) {
			shift = tilesTop(kids[0]) - GRID_TOP;
			shift = Math.max(shift, tilesTop(kids[rowIndex]) - GRID_BOTTOM + (rows[rowIndex].below || 0));
		}
		// only on a change: even a same-value write restyles the rows, and the bar's glass then repaints
		if (el.style.getPropertyValue('--lower-shift') !== shift + 'px') el.style.setProperty('--lower-shift', shift + 'px');
	});
	const overlay = n.zone === 'panel' || n.zone === 'ctx' || n.zone === 'folder';
	const topPointer = (i: number) => () => { if (!overlay && realPointer.current) { n.mode = 'pointer'; n.engaged = true; n.zone = 'top'; n.topIndex = i; render(); } };
	const focusedTop = (i: number) => n.engaged && n.zone === 'top' && i === n.topIndex;
	return (
		<div class={`screen${active ? '' : ' screen--inactive'}${lower ? ' screen--lower' : ''}${n.moving ? ' screen--moving' : ''}${n.dockHidden ? ' screen--dockhidden' : ''}${dockDim ? ' screen--dockdim' : ''}`}>
			<div class="dockdim" />
			<LowerVeil />
			<Status weather={weatherLine ? weather : null} showPeriod={showPeriod} />
			{captions && <VideoCaption />}
			{forecast && weather && <div class={`homecast homecast--${forecast}`}><Forecast weather={weather} size={forecast} /></div>}
			<div class="dockhint" onClick={() => { n.dockHidden = false; render(); }}><svg viewBox="0 0 48 28" aria-hidden="true"><path d="M6 22 24 6l18 16" /></svg></div>
			<div class="topbar">
				{topItems.map((item, i) => (
					<div
						key={item.key}
						data-top={item.key}
						class={`pill${focusedTop(i) ? ' pill--focused' : ''}${n.zone === 'panel' && item.key === INPUTS_KEY ? ' pill--open' : ''}`}
						onMouseLeave={pointerLeft}
						onMouseEnter={topPointer(i)}
						onClick={() => { n.topIndex = i; activateTop(i); }}
					>
						{item.glyph && <Glyph name={item.glyph} />}
						<span class="pill__label">{tr(item.label)}</span>
					</div>
				))}
			</div>
			<div ref={rowsRef} class={`rows${lower ? ' rows--lower' : ''}${n.dockHidden ? ' rows--hidden' : ''}`} style={tiles ? `--layout-tile-size:${tiles.size}px;--layout-tile-height:${tiles.height}px;--layout-tile-gap:${tiles.gap}px;--first-w:${perRow * (tiles.size + tiles.gap) - tiles.gap}px` : undefined} onMouseDown={() => { n.mode = 'pointer'; holdStart(); }} onMouseUp={holdCancel}>
				{rows.map((row, i) => (
					<Row
						key={row.id}
						label={tr(row.label)}
						titled={row.titled}
						className={row.className}
						slot={row.slot}
						tileSize={tiles && tiles.size}
						tileGap={tiles && tiles.gap}
						items={i > 1 && !late ? [] : row.items}
						emptyText={row.emptyText && tr(row.emptyText)}
						focused={n.engaged && n.zone === 'rows' && i === rowIndex}
						rest={!n.engaged}
						onLeave={pointerLeft}
						selected={columnOf(row)}
						onSelect={(col) => { if (overlay || !realPointer.current) return; n.mode = 'pointer'; n.engaged = true; n.zone = 'rows'; n.rowIndex = i; n.columns[row.id] = col; render(); }}
						onActivate={(col) => { holdCancel(); if (n.suppressClick) { n.suppressClick = false; return; } activateRowItem(row.id, row.items[col]); }}
					/>
				))}
			</div>
			{n.zone === 'panel' && (
				<>
					<div class="panel-backdrop panel-backdrop--clear" onClick={() => { n.zone = n.panelFrom; render(); }} />
					<div class="panel panel--cc panel--inputs" ref={(el) => { if (el && !el.dataset.grown) { el.dataset.grown = '1'; growFrom(el, INPUTS_KEY); } }}>
						{inputs.map((item, i) => (
							<div
								key={item.key}
								class={`cc__mod cc__tile cc__opt${i === n.panelIndex ? ' cc__mod--focused' : ''}`}
								onMouseEnter={() => { n.panelIndex = i; render(); }}
								onClick={() => activatePanel(i)}
							>
								<div class="cc__icon">
									{item.glyphImageUrl ? <img class="cc__inputicon" src={item.glyphImageUrl} alt="" decoding="async" /> : item.glyph ? <Glyph name={item.glyph} /> : null}
								</div>
								<div class="cc__text"><div class="cc__name">{tr(item.label)}</div></div>
							</div>
						))}
						{inputs.length === 0 && <div class="cc__caption">No inputs selected: add some in Settings &gt; Sources</div>}
					</div>
				</>
			)}
			{n.moving && <div class="movehint">{tr('Move with the arrows · OK to drop · Hold OK for more')}</div>}
			{n.zone === 'folder' && (
				<>
					<div class="panel-backdrop" onClick={() => { n.zone = 'rows'; render(); }} />
					<div class="panel panel--folder">
						<div class="panel__title">{n.folder}</div>
						<div class="panel__grid" onMouseDown={holdStart} onMouseUp={holdCancel} onMouseLeave={holdCancel}>
							{((folders || {})[n.folder] || []).map((item, i) => (
								<Tile
									key={item.key}
									model={item}
									focused={i === n.folderIndex}
									onHover={() => { n.folderIndex = i; render(); }}
									onActivate={() => { holdCancel(); if (n.suppressClick) { n.suppressClick = false; return; } n.zone = 'rows'; render(); latest.current.onActivate('folder', item); }}
								/>
							))}
						</div>
					</div>
				</>
			)}
			{n.zone === 'ctx' && (() => {
				// the menu sits over the lifted app, inside the screen's edges; under it when the app is too near the top for the menu to fit above
				const items = ctxItems();
				const h = items.length * CTX_ITEM_H;
				const a = n.ctxAnchor;
				const left = a ? clamp(a.x - CTX_W / 2, CTX_EDGE, 1920 - CTX_EDGE - CTX_W) : (1920 - CTX_W) / 2;
				const above = !a || a.top - CTX_GAP - h >= CTX_EDGE;
				const top = !a ? 300 : above ? a.top - CTX_GAP - h : a.bottom + CTX_GAP;
				return (
					<>
						<div class="panel-backdrop panel-backdrop--clear" onClick={() => { n.zone = n.ctxFrom; render(); }} />
						<div key={n.ctxLevel} class="panel panel--cc panel--inputs panel--tilemenu" ref={(el) => { if (el && !el.dataset.risen) { el.dataset.risen = '1'; riseFrom(el, !above); } }} style={{left: left + 'px', top: top + 'px', width: CTX_W + 'px'}}>
							{items.map((item, i) => (
								<div
									key={item.id}
									class={`cc__mod cc__tile cc__opt${i === n.ctxIndex ? ' cc__mod--focused' : ''}${item.id === 'uninstall' && n.ctxConfirm ? ' cc__opt--warn' : ''}`}
									onMouseEnter={() => { n.ctxIndex = i; render(); }}
									onClick={() => activateCtx(i)}
								>
									<div class="cc__icon"><Glyph name={item.glyph} /></div>
									<div class="cc__text"><div class="cc__name">{tr(item.label)}</div></div>
								</div>
							))}
						</div>
					</>
				);
			})()}
		</div>
	);
}
