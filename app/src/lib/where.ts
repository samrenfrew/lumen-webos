// Dev builds only: window.lumen.where() tells an agent on DevTools which screen is up and what the remote has focused. It reads the page,
// since every focus mark in the app is a `--focused` class (tile--focused, pill--focused, slist__row--focused...), so no screen reports
// its own state and a new screen is covered as long as it follows that naming.

export interface Focused {
	/** the focused part's class, without --focused (tile, pill, guide__row, cc__mod...) */
	part: string;
	/** its visible text, squashed to one line */
	label: string;
	/** its place among its siblings of the same part, from 0 */
	index: number;
	/** the heading of the row or section it sits in, if it has one */
	in?: string;
}

export interface Where {
	screen: string;
	/** Control Center is open over the screen */
	controls: boolean;
	/** in page order: an open panel or menu comes after the screen under it, so the last entry is what OK presses */
	focused: Focused[];
}

let state = {screen: 'home', controls: false};

// innerText, not textContent, so a name and its subtitle in separate blocks stay apart ("Sound · Standard")
const text = (el: Element | null | undefined) => (el && (el as HTMLElement).innerText || '').trim().replace(/\s*\n\s*/g, ' · ').replace(/\s+/g, ' ').slice(0, 80);

function describe (el: Element): Focused {
	const cls = Array.prototype.find.call(el.classList, (c: string) => /--focused$/.test(c)) as string;
	const part = cls.replace(/--focused$/, '');
	const siblings = el.parentElement ? Array.prototype.filter.call(el.parentElement.children, (s: Element) => s.classList.contains(part)) : [el];
	const section = el.parentElement && el.parentElement.closest('section, [class$="__page"]');
	const heading = section && section.querySelector('h1, h2, .row__label');
	const out: Focused = {part, label: text(el), index: siblings.indexOf(el)};
	if (heading && !el.contains(heading)) out.in = text(heading);
	return out;
}

export function where (): Where {
	const marks = Array.prototype.filter.call(document.querySelectorAll('[class*="--focused"]'),
		(el: Element) => Array.prototype.some.call(el.classList, (c: string) => /--focused$/.test(c))) as Element[];
	// a part counts once it is on screen: closed screens can keep their state in the DOM while hidden
	const shown = marks.filter((el) => (el as HTMLElement).offsetParent !== null || getComputedStyle(el).position === 'fixed');
	return {...state, focused: shown.map(describe)};
}

/** Called by the app when the screen or Control Center changes. Does nothing outside dev builds. */
export function setWhere (screen: string, controls: boolean): void {
	if (!__LUMEN_DEV__) return;
	state = {screen, controls};
	// other dev hooks (lumen.preview) may have made window.lumen first
	const w = window as unknown as {lumen?: {where?: () => Where}};
	w.lumen = w.lumen || {};
	w.lumen.where = where;
}
