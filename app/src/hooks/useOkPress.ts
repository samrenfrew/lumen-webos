import {useEffect, useRef} from 'preact/hooks';
import {KEY, TIMING} from '../config/constants';

interface Handlers {
	/** a short OK press: on key down where a hold does nothing, else on release before the hold threshold */
	onPress: () => void;
	/** OK held for TIMING.longPressMs (fires while still held; the release is then ignored). Without it, every press acts on key down. */
	onLongPress?: () => void;
	/** asked at key down: can a hold do something where the focus is now? False acts at once, with no wait for the release. Default: true. */
	canHold?: () => boolean;
}

/** Distinguishes a short OK press from a long one. Where a hold can do something, a short press acts on key *up*, so a
 *  hold never triggers the short action first; elsewhere it acts on key down, which is 100 to 250 ms sooner. A key-up without
 *  a matching key-down (e.g. the OK that opened this screen) is ignored. Auto-repeat key-downs are ignored. */
export function useOkPress (handlers: Handlers, active = true): void {
	// Latest handlers in a ref, so a re-render mid-press doesn't reset the hold timer.
	const latest = useRef(handlers);
	latest.current = handlers;

	useEffect(() => {
		if (!active) return;
		let armed = false;
		let fired = false;
		let timer = 0;

		const onKeyDown = (e: KeyboardEvent) => {
			if (e.keyCode !== KEY.enter) return;
			e.preventDefault();
			if (armed || e.repeat) return;
			const {onLongPress, canHold} = latest.current;
			if (!onLongPress || (canHold && !canHold())) { latest.current.onPress(); return; }
			armed = true;
			fired = false;
			timer = window.setTimeout(() => {
				fired = true;
				const hold = latest.current.onLongPress;
				if (hold) hold();
			}, TIMING.longPressMs);
		};

		const onKeyUp = (e: KeyboardEvent) => {
			if (e.keyCode !== KEY.enter) return;
			e.preventDefault();
			if (!armed) return;
			window.clearTimeout(timer);
			armed = false;
			if (!fired) latest.current.onPress();
		};

		window.addEventListener('keydown', onKeyDown);
		window.addEventListener('keyup', onKeyUp);
		return () => {
			window.clearTimeout(timer);
			window.removeEventListener('keydown', onKeyDown);
			window.removeEventListener('keyup', onKeyUp);
		};
	}, [active]);
}
