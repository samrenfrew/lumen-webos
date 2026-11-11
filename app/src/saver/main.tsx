import {render} from 'preact';
import {useEffect, useRef, useState} from 'preact/hooks';
import {Ambient} from '../components/Ambient';
import {applyTokens} from '../theme/tokens';
import {startLog, logError} from '../lib/log';
import {readBackup} from '../lib/backup';
import {loadConfig, saveConfig, type UserConfig} from '../lib/storage';
import {isDark, setClockMode, setLang} from '../lib/i18n';
import {restoreBacklight, setBacklightSchedule} from '../lib/tvcontrols';
import {scheduledBacklight} from '../lib/brightness';
import {AUTO_PLACE, peekWeather, readWeather, type Weather} from '../lib/weather';
import {CITIES, type City} from '../config/cities';
import {copyFontIn, setAmbientShowing} from '../lib/root';
import {isWebOS} from '../lib/luna';
import {setWhere} from '../lib/where';
import '../theme/tokens.css';
import '../theme/home.css';
import '../theme/forecast.css';
import '../theme/ambient.css';
import '../theme/lsaver.css';

applyTokens();
startLog();
// SF Pro from the app's folder: put there by a private build, or copied in from the TV's copy (as Lumen does, lib/root.ts copyFontIn)
if ('FontFace' in window) {
	const url = new URL('fonts/sf.woff2', location.href).href;
	const sf = () => new FontFace('SF Private', `url(${url})`, {weight: '1 1000'}).load().then((face) => { document.fonts.add(face); document.documentElement.classList.add('sf'); });
	sf().catch(() => { copyFontIn(decodeURIComponent(location.pathname).replace(/\/[^/]*$/, '')).then((ok) => ok ? sf() : undefined).catch(() => undefined); });
}

interface WebOSSystem { stageReady?: () => void }
const system = () => (window as unknown as {webOSSystem?: WebOSSystem}).webOSSystem;
/** tells the TV the page can be shown; without it, and with nothing painted, the TV waits 8 s before it puts the window up */
function stageReady (): void {
	const sys = system();
	try { if (sys && sys.stageReady) sys.stageReady(); } catch { /* an older TV without it shows the window anyway */ }
}

/** if no photo is ready by then, fade in anyway (on black, with the clock) */
const SHOW_ANYWAY_MS = 8000;
/** how long Lumen's settings file may take before the copy kept here from last time is used */
const CONFIG_WAIT_MS = 3000;
/** the backlight and the watcher's flag are put back before the app closes, but no longer than this */
const CLOSE_WAIT_MS = 3000;
/** as AMBIENT_BACKLIGHT in app.tsx: Settings > Screensaver > Brightness */
const BACKLIGHT: Record<string, number> = {dim: 30, low: 10};

/** Lumen's settings: the file Lumen writes after each change (lib/backup.ts), else the copy kept here from the last start, else the defaults.
 *  Kept here too, so lib/immich.ts (which reads loadConfig) sees the same values. */
function lumenSettings (): Promise<UserConfig> {
	// loadConfig gives the defaults when nothing is kept yet
	const kept = () => loadConfig();
	if (!isWebOS()) return Promise.resolve(kept());
	const read = readBackup().then((c) => { if (c) saveConfig(c); return c || kept(); });
	const late = new Promise<UserConfig>((resolve) => window.setTimeout(() => resolve(kept()), CONFIG_WAIT_MS));
	return Promise.race([read, late]);
}

/** The classes on <html> the screensaver's stylesheet reads, as useHtmlClasses sets them in Lumen (night dimming, reduce motion, dark). */
function applyClasses (c: UserConfig): void {
	const root = document.documentElement.classList;
	const h = new Date().getHours();
	root.toggle('night', c.wallpaperDim === 'always' || (c.wallpaperDim === 'night' && (h >= 19 || h < 6)));
	root.toggle('reduce-motion', c.reduceMotion);
	const dark = isDark(c.darkMode);
	root.toggle('dark', dark);
	root.toggle('light', !dark);
}

const cityOf = (c: UserConfig): City | null => (c.weatherCity === AUTO_PLACE ? null : CITIES.filter((x) => x.name === c.weatherCity)[0] || null);
const wantsWeather = (c: UserConfig) => !!c.weatherCity && (c.showWeather || c.saverForecast) && (c.weatherCity === AUTO_PLACE || !!cityOf(c));

/** Closes the app once the backlight is back and the watcher knows the screensaver is gone. */
function closeApp (): void {
	const cap = new Promise<void>((resolve) => window.setTimeout(resolve, CLOSE_WAIT_MS));
	const flag = isWebOS() ? setAmbientShowing(false).catch(() => undefined) : Promise.resolve();
	Promise.race([Promise.all([restoreBacklight(), flag]), cap]).then(() => window.close());
}

/** Lumen Screensaver (docs/plans/side-apps.md): Lumen's screensaver in an overlay window, over the app in front, so that app is left as it
 *  was. watch.sh launches it when the TV's screensaver starts over another app. The page stays clear until the first photo is ready, then
 *  fades in; any key fades it out and closes the app. It is not kept running. */
function SaverRoot () {
	const [config, setConfig] = useState<UserConfig | null>(null);
	const [weather, setWeather] = useState<Weather | null>(null);
	const [shown, setShown] = useState(false);
	const [gone, setGone] = useState(false);
	const leaving = useRef(false);
	/** once the screensaver is mounted, it takes the keys (and fades out first) */
	const mounted = useRef(false);
	const leave = () => {
		if (leaving.current) return;
		leaving.current = true;
		setGone(true);
		closeApp();
	};

	useEffect(() => {
		setWhere('lumen screensaver', false);
		lumenSettings().then((c) => {
			setLang(c.language);
			setClockMode(c.clockFormat);
			applyClasses(c);
			setBacklightSchedule(() => scheduledBacklight(c));
			if (wantsWeather(c)) {
				const city = cityOf(c);
				setWeather(peekWeather(city));
				readWeather(city).then(setWeather).catch(() => undefined);
			}
			mounted.current = true;
			setConfig(c);
		});
		const anyway = window.setTimeout(() => setShown(true), SHOW_ANYWAY_MS);
		// a key before the screensaver is mounted ends it too; once mounted, Ambient takes the keys
		const early = (e: KeyboardEvent) => { if (mounted.current) return; e.preventDefault(); leave(); };
		window.addEventListener('keydown', early, true);
		return () => { window.clearTimeout(anyway); window.removeEventListener('keydown', early, true); };
	}, []);
	if (gone || !config) return null;
	const large = config.saverForecast && config.saverForecastSize === 'large';
	return (
		<div class={`lsaver${shown ? ' lsaver--on' : ''}`}>
			<Ambient
				show={config.saverShow}
				source={config.ambient}
				pictures={config.wallpaperPicks}
				seconds={config.ambientSeconds}
				backlight={BACKLIGHT[config.ambientBrightness]}
				weather={weather}
				weatherLine={config.showWeather && !large}
				forecast={config.saverForecast ? config.saverForecastSize as 'small' | 'large' : undefined}
				showPeriod={config.showAmPm}
				print={config.ambientPrint}
				opening={null}
				onShown={() => setShown(true)}
				onExit={leave}
			/>
		</div>
	);
}

// the TV takes the first contentful paint as the sign the page is ready; with none it waits 8 s, so paint an invisible dot
const dot = document.createElement('div');
dot.textContent = '.';
dot.style.cssText = 'position:fixed;left:0;top:0;font-size:2px;color:rgba(255,255,255,0.004);pointer-events:none';
document.body.appendChild(dot);
stageReady();
// a second launch while it is up (the watcher, or Quick Settings' button) leaves it as it is
document.addEventListener('webOSRelaunch', () => stageReady());
render(<SaverRoot />, document.getElementById('root')!);
if (__LUMEN_DEV__) window.requestAnimationFrame(() => logError('lsaver', `open: first frame ${Math.round(performance.now())} ms after the page started`));

// Dev preview only: fit the 1920x1080 page into the window
if (import.meta.env.DEV) {
	const fit = () => { document.documentElement.style.zoom = String(Math.min(window.innerWidth / 1920, window.innerHeight / 1080)); };
	fit();
	window.addEventListener('resize', fit);
}
