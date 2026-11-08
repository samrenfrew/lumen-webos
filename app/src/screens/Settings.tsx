import {t as tr} from '../lib/i18n';
import {ScreenBackdrop} from '../components/LowerVeil';
import {useEffect, useRef, useState} from 'preact/hooks';
import {Grid} from '../components/Grid';
import {SettingsList, type ListRow, type OptionGroup, type OptionItem} from '../components/SettingsList';
import {Glyph, type GlyphName} from '../components/Glyph';
import type {TileModel} from '../components/Tile';
import {useKeys, clamp, type NavKey} from '../hooks/useKeys';
import {useOkPress} from '../hooks/useOkPress';
import {TIMING} from '../config/constants';
import {moveItem} from '../lib/order';
import {LAYOUT} from '../theme/tokens';
import {WallpaperPicker, WALLPAPER_COLUMNS, type WallpaperEntry} from '../components/WallpaperPicker';

export interface SettingsTab {
	id: string;
	label: string;
	glyph: GlyphName;
	/** noun for the re-order button, e.g. "apps" → "Re-order apps" */
	noun: string;
	/** everything available */
	items: TileModel[];
	/** what's on the home screen, in home-screen order */
	shown: TileModel[];
}

export interface WallpaperTab {
	/** empty: the wallpaper type has nothing to choose (Plain) */
	categories: string[];
	entries: WallpaperEntry[];
	/** OK on a tile: pick or unpick it (a Select all tile does its whole category) */
	onPick: (key: string) => void;
	/** under Choose wallpaper: what is picked now */
	summary?: string;
}

interface Props {
	tabs: SettingsTab[];
	wallpaper: WallpaperTab;
	options: OptionItem[];
	active: boolean;
	onToggle: (tabId: string, key: string) => void;
	onReorder: (tabId: string, keys: string[]) => void;
	onOption: (optionId: string) => void;
	/** a value picked from an option's choices page */
	onChoose: (optionId: string, index: number) => void;
	/** text saved on an option's text page */
	onText: (optionId: string, value: string) => void;
	onClose: () => void;
}

/** The option pages, in the order the top page lists them (after the grid pages). */
const GROUPS: {id: OptionGroup; label: string; glyph: GlyphName}[] = [
	{id: 'home', label: 'Home Screen', glyph: 'house'},
	{id: 'livetv', label: 'Live TV', glyph: 'guide'},
	{id: 'wallpaper', label: 'Wallpaper', glyph: 'picture'},
	{id: 'screensaver', label: 'Screensaver', glyph: 'screensaver'},
	{id: 'brightness', label: 'Brightness', glyph: 'sun'},
	{id: 'appearance', label: 'Appearance', glyph: 'contrast'},
	{id: 'photos', label: 'Immich', glyph: 'image'},
	{id: 'region', label: 'Language and Region', glyph: 'globe'},
	{id: 'tv', label: 'Home and Power', glyph: 'power'},
	{id: 'privacy', label: 'Privacy', glyph: 'eye'},
	{id: 'system', label: 'System', glyph: 'gear'}
];
const CHOOSE_WALLPAPER = 'choose-wallpaper';

type Page =
	| {kind: 'root'}
	| {kind: 'group'; id: OptionGroup}
	| {kind: 'grid'; tabId: string}
	| {kind: 'wallpaper'}
	| {kind: 'choice'; optionId: string}
	| {kind: 'text'; optionId: string};

/** A page on the stack, and the row in focus on it (kept for when Back returns to it). */
interface Entry {
	page: Page;
	focus: number;
}

/** On a grid page focus is on the re-order button or the grid; on the wallpaper page, the category chips or the grid. */
type Zone = 'button' | 'content' | 'chips';

/** A tile that has been picked up in re-order mode, and the order to restore on Back. */
interface Held {
	index: number;
	original: string[];
}

const HINT = {
	select: 'OK adds or removes an item · Hold OK to re-order · Back returns',
	reorder: 'OK picks up an item · Hold OK or choose Done to finish',
	holding: 'Arrows move it · OK puts it down · Back cancels',
	wallpaper: 'Left and right change category · OK picks or unpicks · Back returns'
} as const;

/** What a list page shows, and what OK on each row does. */
interface ListPage {
	title: string;
	glyph: GlyphName;
	rows: ListRow[];
	descriptions: string[];
	ticks?: boolean;
	activate: (i: number) => void;
}

/** dev only: ?settings=weather or ?settings=grid:apps opens Settings on that page */
function startStack (): Entry[] {
	const stack: Entry[] = [{page: {kind: 'root'}, focus: 0}];
	if (!import.meta.env.DEV) return stack;
	const at = (/[?&]settings=([\w:-]+)/.exec(location.search) || [])[1];
	if (!at) return stack;
	if (at.indexOf('grid:') === 0) stack.push({page: {kind: 'grid', tabId: at.slice(5)}, focus: 0});
	else if (at === 'wallpaper-picker') stack.push({page: {kind: 'wallpaper'}, focus: 0});
	else stack.push({page: {kind: 'group', id: at as OptionGroup}, focus: 0});
	return stack;
}

/** Settings, laid out like Apple TV's: a stack of pages. OK on a row opens its page, Back goes up one, Back on the top page closes.
 *
 *  List pages: the top page, one page per option group, and a choices page for each list option.
 *  Grid pages (Apps, Sources, Channels): select mode, OK adds/removes an item; re-order mode (hold OK, or the button) shows only
 *  what's on the home screen, in order: OK picks an item up, arrows move it, OK puts it down. Hold OK or press Done to leave. */
export function Settings ({tabs, wallpaper, options, active, onToggle, onReorder, onOption, onChoose, onText, onClose}: Props) {
	const [stack, setStack] = useState<Entry[]>(startStack);
	// keys can arrive faster than renders: changes go through the latest stack, not the one this render saw
	const latest = useRef(stack);
	const commit = (next: Entry[]) => { latest.current = next; setStack(next); };
	const [zone, setZone] = useState<Zone>('content');
	const [gridIndex, setGridIndex] = useState(0);
	const [reordering, setReordering] = useState(false);
	const [held, setHeld] = useState<Held | null>(null);
	const [wpCat, setWpCat] = useState(0);
	const [wpShown, setWpShown] = useState(0);      // the category the grid shows; it follows wpCat once the pill settles
	const [wpIndex, setWpIndex] = useState(0);
	const [typed, setTyped] = useState('');
	const field = useRef<HTMLInputElement>(null);

	const top = stack[stack.length - 1];
	const page = top.page;
	const tab = page.kind === 'grid' ? tabs.filter((t) => t.id === page.tabId)[0] : undefined;
	const onWallpaper = page.kind === 'wallpaper';
	const wpOf = (c: number) => wallpaper.entries.filter((e) => e.category === wallpaper.categories[c]);
	const wpName = wallpaper.categories[wpShown];
	const wpList = wpOf(wpShown);
	const textOption = page.kind === 'text' ? options.filter((o) => o.id === page.optionId)[0] : undefined;
	const items = tab ? (reordering ? tab.shown : tab.items) : [];
	const cols = LAYOUT.gridColumns;
	const keysOf = (list: TileModel[]) => list.map((t) => t.key);

	const push = (next: Page, focus = 0) => {
		commit(latest.current.concat({page: next, focus}));
		if (next.kind === 'grid') { setGridIndex(0); setZone('content'); }
		if (next.kind === 'wallpaper') { setWpIndex(0); setZone('chips'); }
	};
	const pop = () => {
		if (latest.current.length === 1) { onClose(); return; }
		commit(latest.current.slice(0, -1));
		setReordering(false);
		setHeld(null);
	};
	const setFocus = (i: number) => { const now = latest.current; commit(now.slice(0, -1).concat({page: now[now.length - 1].page, focus: i})); };
	/** the page this render drew is still the top one (a key may have opened or closed one since) */
	const current = () => latest.current[latest.current.length - 1].page === page;

	const groupGlyph = (id: OptionGroup) => GROUPS.filter((g) => g.id === id)[0].glyph;

	/** a list of two flips in place on OK, like a switch: a page for one other value is a wasted step */
	const flips = (o: OptionItem) => !!o.choices && o.choices.length === 2 && !o.picks;

	/** OK on an option: a list option opens its choices page, a text option its text page, anything else is handed on (switches flip in place) */
	const activateOption = (o: OptionItem) => {
		if (o.text) { setTyped(o.text.value); push({kind: 'text', optionId: o.id}); }
		else if (flips(o)) onChoose(o.id, o.choice === 1 ? 0 : 1);
		else if (o.choices && o.choices.length > 0) push({kind: 'choice', optionId: o.id}, Math.max(0, o.choice || 0));
		else onOption(o.id);
	};

	const listPage = (): ListPage | null => {
		if (page.kind === 'root') {
			const groups = GROUPS.filter((g) => g.id === 'wallpaper' || options.some((o) => o.group === g.id));
			return {
				title: 'Settings',
				glyph: top.focus < tabs.length ? tabs[top.focus].glyph : groups[top.focus - tabs.length].glyph,
				rows: tabs.map((t): ListRow => ({key: 'tab:' + t.id, label: t.label, chevron: true}))
					.concat(groups.map((g): ListRow => ({key: g.id, label: g.label, chevron: true}))),
				descriptions: [],
				activate: (i) => {
					if (i < tabs.length) push({kind: 'grid', tabId: tabs[i].id});
					else if (groups[i - tabs.length]) push({kind: 'group', id: groups[i - tabs.length].id});
				}
			};
		}
		if (page.kind === 'group') {
			const g = GROUPS.filter((x) => x.id === page.id)[0];
			const list = options.filter((o) => o.group === page.id);
			const link = page.id === 'wallpaper' && wallpaper.categories.length > 0;
			const rows = list.map((o): ListRow => ({
				key: o.id,
				label: o.label,
				value: o.valueText !== undefined ? o.valueText : o.value ? 'On' : 'Off',
				chevron: !!(o.text || (o.choices && o.choices.length && !flips(o)))
			}));
			const descriptions = list.map((o) => o.description);
			// Choose wallpaper sits under Wallpaper type, since the type decides what it offers
			const at = link ? list.findIndex((o) => o.id === 'wp-type') + 1 : -1;
			if (at >= 0) {
				rows.splice(at, 0, {key: CHOOSE_WALLPAPER, label: 'Choose wallpaper', chevron: true} as ListRow);
				descriptions.splice(at, 0, wallpaper.summary || '');
			}
			return {
				title: g.label,
				glyph: g.glyph,
				rows,
				descriptions,
				activate: (i) => {
					if (i === at) { push({kind: 'wallpaper'}); return; }
					const o = list[at >= 0 && i > at ? i - 1 : i];
					if (o) activateOption(o);
				}
			};
		}
		if (page.kind === 'choice') {
			const o = options.filter((x) => x.id === page.optionId)[0];
			if (!o || !o.choices) return null;
			return {
				title: o.label,
				glyph: groupGlyph(o.group),
				rows: o.choices.map((c, i): ListRow => ({key: String(i), label: c, ticked: o.picks ? o.picks.indexOf(i) >= 0 : i === o.choice})),
				descriptions: o.choices.map(() => o.description),
				ticks: true,
				activate: (i) => { onChoose(o.id, i); if (!o.picks) pop(); }
			};
		}
		return null;
	};
	const list = listPage();

	const changeCategory = (i: number) => { setWpCat(clamp(i, 0, wallpaper.categories.length - 1)); setWpIndex(0); };
	/** Down or OK from the bar: show the bar's category at once, then focus the grid. */
	const enterWallpaperGrid = () => { if (wpOf(wpCat).length === 0) return; setWpShown(wpCat); setZone('content'); };
	useEffect(() => {
		if (wpShown === wpCat) return;
		const timer = window.setTimeout(() => setWpShown(wpCat), TIMING.wallpaperGridMs);
		return () => window.clearTimeout(timer);
	}, [wpCat]);

	/** Switch re-order mode, keeping focus on the same item where it exists in both lists. */
	const setMode = (next: boolean) => {
		if (!tab || next === reordering) return;
		const focusedKey = items[gridIndex] && items[gridIndex].key;
		const target = next ? tab.shown : tab.items;
		const i = focusedKey ? keysOf(target).indexOf(focusedKey) : -1;
		setGridIndex(Math.max(0, i));
		setHeld(null);
		setReordering(next);
		if (next && target.length === 0) setZone('button');
	};

	/** Move the held tile to `to`, updating the home screen as it goes. */
	const moveHeld = (to: number) => {
		if (!tab || !held) return;
		const dest = clamp(to, 0, items.length - 1);
		if (dest === held.index) return;
		onReorder(tab.id, moveItem(keysOf(tab.shown), held.index, dest));
		setHeld({...held, index: dest});
		setGridIndex(dest);
	};

	const activateGridItem = (i: number) => {
		if (!tab || !items[i]) return;
		if (!reordering) { onToggle(tab.id, items[i].key); return; }
		if (held) {                                                   // put down (pointer: at the clicked tile)
			if (i !== held.index) moveHeld(i);
			setHeld(null);
			return;
		}
		setHeld({index: i, original: keysOf(tab.shown)});             // pick up
		setGridIndex(i);
	};

	const onPress = () => {
		if (!current()) return;
		if (textOption) { onText(textOption.id, field.current ? field.current.value : typed); pop(); return; }
		if (list) { list.activate(latest.current[latest.current.length - 1].focus); return; }
		if (zone === 'button') { setMode(!reordering); return; }
		if (zone === 'chips') { enterWallpaperGrid(); return; }
		if (onWallpaper) { if (wpList[wpIndex]) wallpaper.onPick(wpList[wpIndex].key); return; }
		activateGridItem(gridIndex);
	};

	const onLongPress = () => {
		if (tab) setMode(!reordering);
	};

	useOkPress({onPress, onLongPress}, active);

	const handleKey = (key: NavKey): boolean => {
		if (key === 'enter') return true;           // handled by useOkPress (press vs hold)

		if (key === 'back') {
			if (held && tab) { onReorder(tab.id, held.original); setGridIndex(held.index); setHeld(null); }
			else if (reordering) setMode(false);
			else pop();
			return true;
		}

		if (!current()) return true;

		if (textOption) return false;              // the arrows move the caret in the text field

		if (list) {
			const at = latest.current[latest.current.length - 1].focus;
			if (key === 'up') setFocus(clamp(at - 1, 0, list.rows.length - 1));
			else if (key === 'down') setFocus(clamp(at + 1, 0, list.rows.length - 1));
			return true;
		}

		if (held) {
			if (key === 'left') moveHeld(held.index - 1);
			else if (key === 'right') moveHeld(held.index + 1);
			else if (key === 'up') moveHeld(held.index - cols);
			else if (key === 'down') moveHeld(held.index + cols);
			return true;
		}

		if (zone === 'chips') {
			if (key === 'left') changeCategory(wpCat - 1);
			else if (key === 'right') changeCategory(wpCat + 1);
			else if (key === 'down') enterWallpaperGrid();
			return true;
		}
		if (zone === 'button') {
			if (key === 'down' && items.length > 0) setZone('content');
			return true;
		}

		if (onWallpaper) {
			const last = wpList.length - 1;
			if (key === 'left') setWpIndex((i) => clamp(i - 1, 0, last));
			else if (key === 'right') setWpIndex((i) => clamp(i + 1, 0, last));
			else if (key === 'up') { if (wpIndex < WALLPAPER_COLUMNS) setZone('chips'); else setWpIndex((i) => i - WALLPAPER_COLUMNS); }
			else if (key === 'down') setWpIndex((i) => (i + WALLPAPER_COLUMNS <= last ? i + WALLPAPER_COLUMNS : i));
			return true;
		}

		const count = items.length;
		switch (key) {
			case 'left': setGridIndex((i) => clamp(i - 1, 0, count - 1)); break;
			case 'right': setGridIndex((i) => clamp(i + 1, 0, count - 1)); break;
			case 'up': if (gridIndex >= cols) setGridIndex((i) => i - cols); else setZone('button'); break;
			case 'down': setGridIndex((i) => (i + cols < count ? i + cols : i)); break;
		}
		return true;
	};

	useKeys(handleKey, active);

	const depth = stack.length;

	// a text page: the field takes focus, which brings up the TV's keyboard (verify)
	useEffect(() => { if (textOption && field.current) field.current.focus(); }, [page]);

	if (textOption && textOption.text) {
		return (
			<div class="screen screen--settings">
				<ScreenBackdrop />
				<div class="settings settings--list" key={depth}>
					<div class="slist">
						<h1 class="slist__title">{tr(textOption.label)}</h1>
						<div class="slist__side">
							<div class="slist__icon"><Glyph name={groupGlyph(textOption.group)} /></div>
							<p class="slist__description">{textOption.description}</p>
						</div>
						<div class="slist__view">
							<input
								ref={field}
								class="slist__input"
								type="text"
								value={typed}
								placeholder={textOption.text.placeholder}
								autocomplete="off"
								autocapitalize="off"
								spellcheck={false}
								onInput={(e) => setTyped((e.target as HTMLInputElement).value)}
							/>
							<p class="slist__note">{tr('OK saves · Back cancels')}</p>
						</div>
					</div>
				</div>
			</div>
		);
	}

	if (list) {
		return (
			<div class="screen screen--settings">
				<ScreenBackdrop />
				<div class="settings settings--list" key={depth}>
					<SettingsList
						title={list.title}
						glyph={list.glyph}
						description={list.descriptions[top.focus]}
						rows={list.rows}
						ticks={list.ticks}
						selected={top.focus}
						onSelect={setFocus}
						onActivate={(i) => { setFocus(i); list.activate(i); }}
					/>
				</div>
			</div>
		);
	}

	const title = tab ? tab.label : 'Wallpaper';
	const hint = onWallpaper ? HINT.wallpaper : held ? HINT.holding : reordering ? HINT.reorder : HINT.select;
	const buttonClass = ['settings__button', reordering && 'settings__button--active', zone === 'button' && 'settings__button--focused'].filter(Boolean).join(' ');

	return (
		<div class="screen screen--settings">
			<ScreenBackdrop />
			<div class={`settings settings--grid${tab ? ' settings--tiles' : ''}`} key={depth}>
				<div class="settings__head">
					<h1 class="settings__title">{tr(title)}</h1>
					{tab && (
						<div
							class={buttonClass}
							onMouseEnter={() => setZone('button')}
							onClick={() => setMode(!reordering)}
						>
							{reordering ? tr('Done') : tr(`Re-order ${tab.noun}`)}
						</div>
					)}
				</div>
				<p class="settings__hint">{tr(hint)}</p>
				{tab && (
					<Grid
						items={items}
						mode={reordering ? 'reorder' : 'select'}
						checkedKeys={keysOf(tab.shown)}
						liftedIndex={held ? held.index : -1}
						focused={zone === 'content'}
						selected={gridIndex}
						onSelect={(i) => { if (!held) { setZone('content'); setGridIndex(i); } }}
						onActivate={(i) => { setZone('content'); activateGridItem(i); }}
					/>
				)}
				{onWallpaper && (
					<WallpaperPicker
						categories={wallpaper.categories}
						categoryIndex={wpCat}
						entries={wpList}
						chipsFocused={zone === 'chips'}
						gridFocused={zone === 'content'}
						selected={wpIndex}
						onCategory={(i) => { setZone('chips'); changeCategory(i); }}
						onSelect={(i) => { setZone('content'); setWpIndex(i); }}
						onActivate={(i) => { setZone('content'); if (wpList[i]) wallpaper.onPick(wpList[i].key); }}
						note={wpName === 'Video' ? tr('Pick one or more places and the video wallpaper plays them in random order. All places plays everything: aerial videos and free stock videos of European countries. Videos stream from the internet (up to about 3 GB an hour) and take a few seconds to start. Settings > Wallpaper > Video wallpaper hours limits when they play.') : undefined}
					/>
				)}
				{reordering && items.length === 0 && <p class="settings__hint">{tr('Nothing on the home screen yet.')}</p>}
			</div>
		</div>
	);
}
