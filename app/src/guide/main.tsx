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
interface WebOSSystem { hide?: () => void }

/** Hide rather than close, so the app stays running and the next LIST shows it at once (a relaunch). Off the TV it closes. */
function hideWindow (): void {
	const sys = (window as unknown as {webOSSystem?: WebOSSystem}).webOSSystem;
	if (sys && sys.hide) sys.hide();
	else window.close();
}

/** Keeps the data between showings. LIST arrives as a relaunch: while shown it hides the guide; while hidden the TV has already shown
 *  the window again, and the panel mounts afresh on Now and asks the TV again. Nothing is drawn while hidden, so it holds no layers. */
function GuideRoot () {
	const data = useGuideData();
	const [shown, setShown] = useState(true);
	const [session, setSession] = useState(0);
	const shownRef = useRef(true);
	const hide = useCallback(() => { shownRef.current = false; setShown(false); hideWindow(); }, []);
	useEffect(() => {
		const onRelaunch = () => {
			if (shownRef.current) { hide(); return; }
			shownRef.current = true;
			setSession((n) => n + 1);
			setShown(true);
			data.refresh();
		};
		document.addEventListener('webOSRelaunch', onRelaunch);
		return () => document.removeEventListener('webOSRelaunch', onRelaunch);
	}, []);
	return shown ? <GuideApp key={session} data={data} onClose={hide} /> : null;
}

render(<GuideRoot />, document.getElementById('root')!);
// dev builds: how long from the page starting to the first frame of the panel, in the TV's lumen.log
if (__LUMEN_DEV__) window.requestAnimationFrame(() => logError('lguide', `open: first frame ${Math.round(performance.now())} ms after the page started`));

// Dev preview only: fit the 1920x1080 panel into the window
if (import.meta.env.DEV) {
	const fit = () => { document.documentElement.style.zoom = String(Math.min(window.innerWidth / 1920, window.innerHeight / 1080)); };
	fit();
	window.addEventListener('resize', fit);
}
