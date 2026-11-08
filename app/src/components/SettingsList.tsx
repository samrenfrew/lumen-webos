import {t as tr} from '../lib/i18n';
import {Glyph, type GlyphName} from './Glyph';
import {rowWindow} from '../lib/windowedList';

/** Which Settings page an option is listed on (Settings.tsx GROUPS). */
export type OptionGroup = 'home' | 'livetv' | 'wallpaper' | 'screensaver' | 'brightness' | 'appearance' | 'photos' | 'region' | 'tv' | 'privacy' | 'system';

export interface OptionItem {
	id: string;
	group: OptionGroup;
	label: string;
	description: string;
	value: boolean;
	/** shown instead of On/Off (options that pick from a list, and actions) */
	valueText?: string;
	/** the values to pick from: OK opens them on a page of their own */
	choices?: string[];
	/** the index in choices that is set now */
	choice?: number;
	/** a list that keeps several: the indexes set now; OK ticks or unticks one and the page stays open */
	picks?: number[];
	/** typed text: OK opens a page with a text field (the TV's keyboard), OK there saves it */
	text?: {value: string; placeholder: string};
}

export interface ListRow {
	key: string;
	label: string;
	/** grey text on the right */
	value?: string;
	/** opens a page */
	chevron?: boolean;
	/** a choices page: the value set now */
	ticked?: boolean;
}

interface Props {
	title: string;
	glyph: GlyphName;
	/** under the glyph: what the focused row does */
	description?: string;
	rows: ListRow[];
	/** a choices page: rows get a tick column */
	ticks?: boolean;
	selected: number;
	onSelect: (index: number) => void;
	onActivate: (index: number) => void;
}

const ROW_H = 84;           // .slist__row height + margin (theme/settings.css)
const VIEW_H = 840;         // .slist__view height
const SPARE = 6;            // rows drawn above and below the window, so a quick scroll never shows a gap
const SHADOW_ROOM = 48;     // under the last row: its focus shadow (0 14px 28px) and the 1.03 grow, so the window's edge never cuts it

/** A tvOS-style Settings page: a big glyph on the left, single-line rows on the right.
 *  Only the rows near the window are drawn (the city list can hold every city). */
export function SettingsList ({title, glyph, description, rows, ticks, selected, onSelect, onActivate}: Props) {
	const n = rows.length;
	const visible = Math.floor((VIEW_H - SHADOW_ROOM) / ROW_H);
	const {top, from: first, to: last} = rowWindow(n, selected, visible, Math.floor((visible - 1) / 2), SPARE, true);
	const offset = top * ROW_H;
	// the bottom fade only while more rows wait below; at the end of a long list the last row shows whole
	const more = n > visible && offset < (n - visible) * ROW_H;

	return (
		<div class="slist">
			<h1 class="slist__title">{tr(title)}</h1>
			<div class="slist__side">
				<div class="slist__icon"><Glyph name={glyph} /></div>
				<p class="slist__description">{description || ''}</p>
			</div>
			<div class={`slist__view${more ? ' slist__view--long' : ''}`}>
				<div class="slist__track" style={`height:${n * ROW_H}px;transform:translateY(${-offset}px)`}>
					{rows.slice(first, last).map((r, k) => { const i = first + k; return (
						<div
							key={r.key}
							style={`top:${i * ROW_H}px`}
							class={`slist__row${i === selected ? ' slist__row--focused' : ''}`}
							onMouseEnter={() => onSelect(i)}
							onClick={() => onActivate(i)}
						>
							{ticks && <svg class={`slist__tick${r.ticked ? ' slist__tick--on' : ''}`} viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12.5l4.5 4.5L19 7.5" /></svg>}
							<span class="slist__label">{tr(r.label)}</span>
							{r.value !== undefined && <span class="slist__value">{tr(r.value)}</span>}
							{r.chevron && <svg class="slist__chevron" viewBox="0 0 12 20" aria-hidden="true"><path d="M2 2l8 8-8 8" /></svg>}
						</div>
					); })}
				</div>
			</div>
		</div>
	);
}
