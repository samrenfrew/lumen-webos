import {render} from 'preact';
import {useCallback, useEffect, useRef, useState} from 'preact/hooks';
import {GuideApp} from './GuideApp';
import {useGuideData} from './useGuideData';
import {applyTokens} from '../theme/tokens';
import {startLog, logError} from '../lib/log';
import '../theme/tokens.css';
import '../theme/guide.css';
import '../theme/lguide.css';

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

function paramsOf (p: unknown): {hidden?: unknown; view?: unknown} {
	try {
		const o = typeof p === 'string' ? JSON.parse(p) : p;
		return o && typeof o === 'object' ? o as {hidden?: unknown; view?: unknown} : {};
	} catch { return {}; }
}
/** watch.sh starts the guide with {"hidden":true} when Live TV comes to the front, so the first LIST shows it at once */
const wantsHidden = (p: unknown) => paramsOf(p).hidden === true;
/** GUIDE launches it with {"view":"full"} (watch.sh), LIST with no params */
const wantsFull = (p: unknown) => paramsOf(p).view === 'full';
const startHidden = (() => { const sys = system(); return !!sys && wantsHidden(sys.launchParams); })();
const startFull = (() => { const sys = system(); return !!sys && wantsFull(sys.launchParams); })();
/** a hidden start waits for the TV to put the window up once (it gets focus), then hides it: hidden before that, LIST never shows it */
let hideOnFirstFocus = startHidden;
/** if focus never comes, hide anyway after this long */
const FIRST_FOCUS_WAIT_MS = 10000;
/** a showing asks the TV again only after the panel has slid in: the root calls' shells otherwise stall its first frames */
const REFRESH_AFTER_MS = 1200;

/** Hide rather than close, so the app stays running and the next LIST shows it at once (a relaunch). Off the TV it closes. */
function hideWindow (): void {
	const sys = system();
	if (sys && sys.hide) sys.hide();
	else window.close();
}

/** Keeps the data between showings. LIST and GUIDE arrive as relaunches. LIST opens on Now and GUIDE on the Full guide; the key that
 *  matches what is up hides it, and the other key switches view. While hidden the TV has already shown the window again, and the
 *  panel mounts afresh and asks the TV again. Nothing is drawn while hidden, so it holds no layers. */
function GuideRoot () {
	const data = useGuideData();
	const [shown, setShown] = useState(!startHidden);
	const [session, setSession] = useState(0);
	const [full, setFull] = useState(startFull);
	const shownRef = useRef(!startHidden);
	const viewRef = useRef(startFull ? 'full' : 'now');
	const hide = useCallback(() => { shownRef.current = false; setShown(false); hideWindow(); }, []);
	useEffect(() => {
		const onRelaunch = (e: Event) => {
			const sys = system();
			const detail = (e as CustomEvent).detail;
			hideOnFirstFocus = false;
			stageReady();
			// the watcher's hidden start while it already runs: the TV has shown the window again, so hide it unless the panel is up
			const params = detail || (sys && sys.launchParams);
			if (wantsHidden(params)) { if (!shownRef.current) hideWindow(); return; }
			const toFull = wantsFull(params);
			if (shownRef.current) {
				// the key for the view that is up hides the guide; the other key switches to its view
				if (toFull === (viewRef.current === 'full')) hide();
				else { setFull(toFull); setSession((n) => n + 1); }
				return;
			}
			shownRef.current = true;
			setFull(toFull);
			setSession((n) => n + 1);
			setShown(true);
			window.setTimeout(data.refresh, REFRESH_AFTER_MS);
		};
		document.addEventListener('webOSRelaunch', onRelaunch);
		return () => document.removeEventListener('webOSRelaunch', onRelaunch);
	}, []);
	return shown ? <GuideApp key={session} data={data} startView={full ? 'full' : 'now'} onView={(v) => { viewRef.current = v; }} onClose={hide} /> : null;
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
		if (__LUMEN_DEV__) logError('lguide', `start hidden: hid ${Math.round(performance.now())} ms after the page started`);
	};
	stageReady();
	if (document.hasFocus()) firstHide();
	else window.addEventListener('focus', firstHide, {once: true});
	window.setTimeout(firstHide, FIRST_FOCUS_WAIT_MS);
}
render(<GuideRoot />, document.getElementById('root')!);
// dev builds: how long from the page starting to the first frame of the panel, in the TV's lumen.log
if (__LUMEN_DEV__ && !startHidden) window.requestAnimationFrame(() => logError('lguide', `open: first frame ${Math.round(performance.now())} ms after the page started`));

// Dev preview only: fit the 1920x1080 panel into the window
if (import.meta.env.DEV) {
	const fit = () => { document.documentElement.style.zoom = String(Math.min(window.innerWidth / 1920, window.innerHeight / 1080)); };
	fit();
	window.addEventListener('resize', fit);
}
