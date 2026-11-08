import {t as tr} from '../lib/i18n';
import {Glyph, type GlyphName} from './Glyph';

export interface RailItem { key: string; label: string; glyph: GlyphName }

/** The guides' rail (theme/guide.css .rail): icons at the left edge of the screen; open, it widens over the list with names. `on` is the view
 *  shown, `at` the item focused while it is open. Keys are the screen's: Left opens it, Up/Down move, OK picks, Right closes. */
export function Rail ({items, on, at, open}: {items: RailItem[]; on: number; at: number; open: boolean}) {
	return (
		<div class={`rail${open ? ' rail--open' : ''}`}>
			<div class="rail__plate" />
			{items.map((r, k) => (
				<div key={r.key} class={`rail__item${open && k === at ? ' rail__item--focused' : ''}${k === on ? ' rail__item--on' : ''}`}>
					<Glyph name={r.glyph} />
					<span class="rail__label">{tr(r.label)}</span>
				</div>
			))}
		</div>
	);
}
