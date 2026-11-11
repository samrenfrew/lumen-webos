import {render} from 'preact';
import {useCallback, useEffect, useRef, useState} from 'preact/hooks';
import {ControlCenter, prefetchControls, type CenterAction, type ControlLayout} from '../components/ControlCenter';
import {applyTokens} from '../theme/tokens';
import {startLog, logError} from '../lib/log';
import {appExists, launchApp} from '../lib/launch';
import {setWhere} from '../lib/where';
import '../theme/tokens.css';
import '../theme/panels.css';
import '../theme/lquick.css';

applyTokens();
startLog();
if ('FontFace' in window) {
	const url = new URL('fonts/sf.woff2', location.href).href;
	new FontFace('SF Private', `url(${url})`, {weight: '1 1000'}).load().then((face) => { document.fonts.add(face); }, () => undefined);
}
interface WebOSSystem { hide?: () => void; stageReady?: () => void; launchParams?: string }
const system = () => (window as unknown as {webOSSystem?: WebOSSystem}).webOSSystem;
/** tells the TV the page can be shown; after a hidden start it otherwise waits 8 s on every show */
function stageReady (): void {
	const sys = system();
	try { if (sys && sys.stageReady) sys.stageReady(); } catch { /* an older TV without it shows the window anyway */ }
}

/** watch.sh starts it with {"hidden":true} when an app other than Lumen comes to the front, so the first gear press shows it at once */
function wantsHidden (p: unknown): boolean {
	try {
		const o = typeof p === 'string' ? JSON.parse(p) : p;
		return !!o && typeof o === 'object' && (o as {hidden?: unknown}).hidden === true;
	} catch { return false; }
}
const startHidden = (() => { const sys = system(); return !!sys && wantsHidden(sys.launchParams); })();
/** a hidden start waits for the TV to put the window up once (it gets focus), then hides it: hidden before that, the gear never shows it */
let hideOnFirstFocus = startHidden;
/** if focus never comes, hide anyway after this long */
const FIRST_FOCUS_WAIT_MS = 10000;
/** the panel closes itself after this long without a key */
const IDLE_MS = 30000;
const TOAST_MS = 3000;

/** Lumen's Control Center without the tiles that are Lumen's own settings (Theme, Clock) or that need Lumen under them (LG Home). */
const LAYOUT: ControlLayout = [
	[{id: 'sound', span: 2}, {id: 'picture', span: 2}],
	[{id: 'backlight', span: 3}, {id: 'eye', span: 1}],
	[{id: 'settings', span: 1}, {id: 'art', span: 1}, {id: 'screenoff', span: 1}, {id: 'restart', span: 1}]
];
/** LG's full Settings app, which the gear button no longer reaches by itself */
const LG_SETTINGS = 'com.palm.app.settings';
const LUMEN = 'com.samrenfrew.lumen';
/** Lumen Screensaver (docs/plans/side-apps.md): the photos over the app underneath, so it is left as it was */
const SAVER = 'com.samrenfrew.lumen.saver';
/** the screensaver over the app underneath, or Lumen's own when Lumen Screensaver is not installed */
const openScreensaver = () => appExists(SAVER).then((yes) => (yes ? launchApp(SAVER) : launchApp(LUMEN, {view: 'ambient'})));

/** Hide rather than close, so the app stays running and the next gear press shows it at once (a relaunch). Off the TV it closes. */
function hideWindow (): void {
	const sys = system();
	if (sys && sys.hide) sys.hide();
	else window.close();
}

/** Lumen Quick Settings (docs/plans/quick-settings.md): Lumen's Control Center in an overlay window, over whatever app is in front. The gear
 *  button arrives as a relaunch: it shows the panel, or hides it when it is up. Back hides it, as does IDLE_MS without a key. Each
 *  showing mounts the panel afresh, and it reads the TV's values for the app underneath. */
function QuickRoot () {
	const [shown, setShown] = useState(!startHidden);
	const [session, setSession] = useState(0);
	const [toast, setToast] = useState('');
	const shownRef = useRef(!startHidden);
	const hide = useCallback(() => { shownRef.current = false; setShown(false); setToast(''); hideWindow(); }, []);
	useEffect(() => {
		const onRelaunch = (e: Event) => {
			const sys = system();
			hideOnFirstFocus = false;
			stageReady();
			// the watcher's hidden start while it already runs: the TV has shown the window again, so hide it unless the panel is up
			if (wantsHidden((e as CustomEvent).detail || (sys && sys.launchParams))) { if (!shownRef.current) hideWindow(); return; }
			if (shownRef.current) { hide(); return; }
			shownRef.current = true;
			setSession((n) => n + 1);
			setShown(true);
		};
		document.addEventListener('webOSRelaunch', onRelaunch);
		return () => document.removeEventListener('webOSRelaunch', onRelaunch);
	}, []);
	// any key keeps it open for another IDLE_MS
	useEffect(() => {
		if (!shown) return undefined;
		let idle = window.setTimeout(hide, IDLE_MS);
		const onKey = () => { window.clearTimeout(idle); idle = window.setTimeout(hide, IDLE_MS); };
		window.addEventListener('keydown', onKey, true);
		return () => { window.clearTimeout(idle); window.removeEventListener('keydown', onKey, true); };
	}, [shown, session]);
	useEffect(() => {
		if (!toast) return undefined;
		const id = window.setTimeout(() => setToast(''), TOAST_MS);
		return () => window.clearTimeout(id);
	}, [toast]);
	useEffect(() => setWhere('lumen quick settings', shown), [shown]);
	const act = (a: CenterAction) => {
		hide();
		const go = a === 'settings' ? launchApp(LG_SETTINGS) : a === 'art' ? openScreensaver() : Promise.resolve();
		go.catch((e: Error) => logError('lquick', `${a} failed: ${e.message}`));
	};
	if (!shown) return null;
	return (
		<div class="lquick" key={session}>
			<div class="lquick__shade" />
			<ControlCenter layout={LAYOUT} onClose={hide} onCloseUp={() => undefined} onCloseDown={() => undefined} onError={setToast} onAction={act} darkMode="on" clockFormat="auto" onDarkMode={() => undefined} onClockFormat={() => undefined} />
			{toast && <div class="lquick__toast">{toast}</div>}
		</div>
	);
}

if (startHidden) {
	// the TV takes the first contentful paint as the sign the page is ready; with none it waits 8 s on every show, so paint an invisible dot
	const dot = document.createElement('div');
	dot.textContent = '.';
	dot.style.cssText = 'position:fixed;left:0;top:0;font-size:2px;color:rgba(255,255,255,0.004);pointer-events:none';
	document.body.appendChild(dot);
	const firstHide = () => {
		if (!hideOnFirstFocus) return;
		hideOnFirstFocus = false;
		dot.remove();
		hideWindow();
		// the TV's values, so the first showing opens on them
		prefetchControls();
		if (__LUMEN_DEV__) logError('lquick', `start hidden: hid ${Math.round(performance.now())} ms after the page started`);
	};
	stageReady();
	if (document.hasFocus()) firstHide();
	else window.addEventListener('focus', firstHide, {once: true});
	window.setTimeout(firstHide, FIRST_FOCUS_WAIT_MS);
}
render(<QuickRoot />, document.getElementById('root')!);
if (__LUMEN_DEV__ && !startHidden) window.requestAnimationFrame(() => logError('lquick', `open: first frame ${Math.round(performance.now())} ms after the page started`));

// Dev preview only: fit the 1920x1080 page into the window
if (import.meta.env.DEV) {
	const fit = () => { document.documentElement.style.zoom = String(Math.min(window.innerWidth / 1920, window.innerHeight / 1080)); };
	fit();
	window.addEventListener('resize', fit);
}
