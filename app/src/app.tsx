import {useCallback, useEffect, useMemo, useRef, useState} from 'preact/hooks';
import {Home, type HomeRow} from './screens/Home';
import {Settings, type SettingsTab, type WallpaperTab} from './screens/Settings';
import {Guide} from './screens/Guide';
import type {WallpaperEntry} from './components/WallpaperPicker';
import type {OptionItem} from './components/SettingsList';
import {Toast, type ToastMessage} from './components/Toast';
import type {TileModel} from './components/Tile';
import {BUILTIN, BUILTIN_APPS, withBuiltins} from './config/builtinApps';
import {cachedApps, listApps, sameApps, saveAppsCache, seedAppIds, type AppEntry} from './lib/apps';
import {afterFirstFrame, readCache, writeCache} from './lib/boot';
import {listInputs, MOCK_INPUTS, type InputSource} from './lib/inputs';
import {launchApp, launchLgHome} from './lib/launch';
import {isWebOS} from './lib/luna';
import {hidePointerOnForeground} from './lib/pointer';
import {DEFAULT_CONFIG, IDLE_SECONDS, hasSavedConfig, loadConfig, saveConfig, toggleId, withWallpaper, type UserConfig} from './lib/storage';
import {readBackup, scheduleBackup} from './lib/backup';
import {onRelaunch, startParams, type LaunchParams} from './lib/launchParams';
import {appTile, inputTile, pickInOrder} from './lib/tiles';
import {TIMING} from './config/constants';
import {LOCAL_WALLPAPERS, WALLPAPERS, WALLPAPER_CATEGORIES, cloudUrl} from './config/wallpapers';
import {setLiquidDock} from './lib/liquidGlass';
import {resized, setPicks, setWallpaper, shownWallpaper} from './lib/wallpaper';
import {restoreLeftoverBacklight, setBacklightSchedule} from './lib/tvcontrols';
import {scheduledBacklight, writeBrightnessSchedule} from './lib/brightness';
import {albums as immichAlbums, cachedServer, IMMICH_SOURCES, loadServer, saveServer, testServer, type ImmichServer} from './lib/immich';
import {Ambient, prepareAmbient} from './components/Ambient';
import {setVideoChoice, setVideoPaused, setVideoWallpaper, VIDEO_PLACES} from './lib/wallpaperVideo';
import {clearSaverOffFlag, setAmbientFlag, setVideoFlag, startScreensaver} from './lib/system';
import {setFpsCounter} from './lib/fps';
import {setWhere} from './lib/where';
import {onMinute} from './lib/minute';
import {useHtmlClasses} from './hooks/useHtmlClasses';
import {ControlCenter, prefetchControls, type CenterAction} from './components/ControlCenter';
import {CITIES, COUNTRIES} from './config/cities';
import {AUTO_PLACE, peekWeather, readWeather, type Weather} from './lib/weather';
import {useLiveTv} from './hooks/useLiveTv';
import {channelColor, nowNextAt, rowChannels, type Channel} from './lib/livetv';
import {homeSetup, installTvScripts, openChannel, privacy, reboot, runningApps, uninstallApp, type HomeAction, type PrivacyAction} from './lib/root';
import {behind, installSide, readSideApps, removeSide, updateSides, SIDE_APP_TITLE, type SideAppId, type SideApps} from './lib/sideapps';
import {appDir, nativeWhy, powerOnPending, powerOnWhy, readTvState, scriptsReady, type TvState} from './lib/tvsetup';
import {APP_ID} from './config/constants';
import {bootMark, navMark} from './lib/log';
import {LANGS, setClockMode, setLang} from './lib/i18n';

type Screen = 'home' | 'settings' | 'ambient' | 'guide';

const ROW = {apps: 'apps', sources: 'sources', misc: 'misc', dock: 'dock', live: 'livetv', channels: 'channels'} as const;
/** the Live TV row: the Guide tile first, then its channels' tiles (channel:<channelId>) */
const LIVE = {prefix: 'channel:', guide: 'livetv:guide'} as const;
/** the width of a channel tile's now/next block (.chan__info in theme/live-tv.css): the row scrolls on so the focused tile always has this to its right */
const LIVE_INFO_W = 660;
/** how much higher the Live TV row sits when focused, so its now/next block fits on screen */
const LIVE_BELOW = 150;
const MISC = {lgHome: 'lg-home', settings: 'settings'} as const;
const OPTION = {liveTv: 'live-tv-row', ambient: 'ambient', ambientOn: 'ambient-on', saverShow: 'saver-show', ambientSeconds: 'ambient-seconds', ambientBrightness: 'ambient-brightness', ambientPrint: 'ambient-print', fps: 'fps-counter', guidePreview: 'guide-preview', guideSound: 'guide-sound', homePreview: 'home-preview', restore: 'restore-backup', focus: 'focus-strength', firstRow: 'first-row', videoHours: 'video-hours', dockBlur: 'dock-blur', liquidDock: 'liquid-dock', reduceMotion: 'reduce-motion', hideNames: 'hide-names', showRecents: 'show-recents', dockHide: 'dock-hide', showAmPm: 'show-ampm', showAll: 'show-all-apps', weather: 'show-weather', homeCast: 'home-forecast', saverCast: 'saver-forecast', immichUrl: 'immich-url', immichKey: 'immich-key', immichTest: 'immich-test', city: 'weather-city', country: 'weather-country', idle: 'idle-screensaver', wpAuto: 'wp-auto', immichDays: 'immich-recent-days', wpType: 'wp-type', wpNew: 'wp-new', nightDim: 'night-dim', dockDim: 'dock-dim', captions: 'wallpaper-captions', darkMode: 'dark-mode', language: 'language', clock: 'clock-format', tvScripts: 'tv-scripts', tvHome: 'tv-home', tvPowerOn: 'tv-power-on', tvRestart: 'tv-restart', pvAgreements: 'privacy-agreements', pvLmt: 'privacy-lmt', pvLivePlus: 'privacy-liveplus', pvHosts: 'privacy-hosts', pvServices: 'privacy-services', pvUpdates: 'privacy-updates', blOn: 'brightness-on', blDayAt: 'brightness-day-at', blDay: 'brightness-day', blEveningAt: 'brightness-evening-at', blEvening: 'brightness-evening', blNightAt: 'brightness-night-at', blNight: 'brightness-night', blRamp: 'brightness-ramp', blEye: 'brightness-eye', sideGuide: 'side-guide', sideQuick: 'side-quick', sideSaver: 'side-saver', sideUpdate: 'side-update'} as const;
const WP_AUTO_CHOICES = [0, 0.5, 1, 5, 15, 30, 60];   // minutes
/** when the video wallpaper plays (Settings > Wallpaper > Video wallpaper hours): [first hour, hour it stops]; always = no limit */
const VIDEO_HOURS: Record<string, [number, number]> = {day: [7, 19], evening: [17, 23], night: [19, 7]};
function inVideoHours (mode: string, now = new Date()): boolean {
	const span = VIDEO_HOURS[mode];
	if (!span) return true;
	const h = now.getHours();
	return span[0] < span[1] ? h >= span[0] && h < span[1] : h >= span[0] || h < span[1];
}
/** tile sizes for 5, 6 or 7 apps a row: always about 1670px wide, with 40px of glass round them (the bar about 91% of the screen), still 5:3 */
const TILES_FOR: Record<number, {size: number; height: number; gap: number}> = {5: {size: 288, height: 173, gap: 58}, 6: {size: 240, height: 144, gap: 46}, 7: {size: 206, height: 124, gap: 38}};
/** the backlight the screensaver lowers the TV to, by Settings > Screensaver > Brightness */
const AMBIENT_BACKLIGHT: Record<string, number> = {dim: 30, low: 10};
const PREPARE_SAVER_MS = 8000;   // after the home screen shows, so the screensaver's photo loads clear of the boot and the way back from an app
const RETURN_READ_MS = 1500;     // after Lumen comes back, the inputs and open apps are read once the redraw is over (docs/keep-alive.md)
const RECENT_MAX = 8;
/** what the last launch showed, drawn at once on a cold start (lib/boot.ts) */
const INPUTS_CACHE_KEY = 'lumen-boot-inputs-v1';

/** The top bar: Control Center, Settings and Inputs. Control Center comes first, so it has the focus the first time Up reaches the top bar. */
const TOP_ITEMS: TileModel[] = [
	{key: 'controls', label: 'Control Center', glyph: 'sliders'},
	{key: MISC.settings, label: 'Settings', glyph: 'gear'},
	{key: 'inputs', label: 'Inputs', glyph: 'inputs'}
];


const LUMEN_IDS = ['com.samrenfrew.lumen', 'com.samrenfrew.lumen.dev'];
/** Settings > Brightness: the start times on offer, every half hour from `from` to `to` o'clock (past 24 wraps to the next morning) */
const halfHours = (from: number, to: number) => {
	const out: string[] = [];
	for (let m = from * 60; m <= to * 60; m += 30) out.push(`${String(Math.floor(m / 60) % 24).padStart(2, '0')}:${m % 60 === 0 ? '00' : '30'}`);
	return out;
};
const BL_LEVELS = Array.from({length: 21}, (_, i) => i * 5);
const NEEDS_SCRIPTS = 'Install the TV scripts first, in Settings > System > TV scripts. ';

/** Settings > Home and Power and Settings > Privacy: one row per TV-script setting, each showing what the TV reports. */
function tvOptions (tv: TvState | null, busy: string): OptionItem[] {
	const ready = scriptsReady(tv);
	/** the value shown: Working while it changes, Checking before the first read, and the script's state after */
	const shown = (id: string, value: string) => busy === id ? 'Working' : !tv ? (isWebOS() ? 'Checking' : 'Not on a TV') : !ready ? 'Needs TV scripts' : value;
	const pre = tv && !ready ? NEEDS_SCRIPTS : '';
	const onOff = (v: string) => v === 'on' ? 'On' : v === 'off' ? 'Off' : 'Not on this TV';
	const rows: OptionItem[] = [];
	const lumenHome = !!tv && LUMEN_IDS.indexOf(tv.home) >= 0;
	const nWhy = tv ? nativeWhy(tv) : '';
	rows.push({
		id: OPTION.tvHome, group: 'tv', label: 'Home button opens',
		description: pre + (nWhy && !lumenHome ? nWhy : "Which app LG's own Home routes open: the Home button, Exit, and Back out of an app. Lumen's LG Home button still opens LG Home. LG Input Hook can keep its Home mapping."),
		value: lumenHome,
		valueText: shown(OPTION.tvHome, lumenHome ? 'Lumen' : nWhy ? 'Not available' : 'LG Home')
	});
	const pWhy = tv ? powerOnWhy(tv) : '';
	const powerLumen = !!tv && LUMEN_IDS.indexOf(tv.powerOn) >= 0;
	const pending = powerOnPending(tv);
	rows.push({
		id: OPTION.tvPowerOn, group: 'tv', label: 'When the TV turns on',
		description: pre + (pWhy && !powerLumen ? pWhy : 'Lumen: the power button and a start after a power cut open Lumen first, before Live TV is drawn. Casting, Wake-on-LAN and HDMI-CEC still go where they ask. Takes effect after the TV restarts.'),
		value: powerLumen,
		valueText: shown(OPTION.tvPowerOn, (powerLumen ? 'Lumen' : pWhy ? 'Not available' : 'Last input') + (pending ? ' (after a restart)' : ''))
	});
	if (pending) rows.push({
		id: OPTION.tvRestart, group: 'tv', label: 'Restart the TV now',
		description: 'The TV reads what it opens at power-on only when it starts. This restarts it now.',
		value: false, valueText: shown(OPTION.tvRestart, 'Restart')
	});

	const a = tv ? tv.agreements : null;
	rows.push({
		id: OPTION.pvAgreements, group: 'privacy', label: 'LG data agreements',
		description: pre + (tv && !tv.node ? 'This TV has no node to change them safely. ' : '') + "Withdraws LG's optional agreements: screen recognition, ads based on what you watch, personalised ads, ad cookies, sharing with data partners, extra usage data, remote diagnostics and marketing. Voice and the terms of use stay. OK again puts back what was there before.",
		value: a === 0,
		valueText: shown(OPTION.pvAgreements, a === null ? 'Unknown' : a === 0 ? 'Withdrawn' : `${a} given`)
	});
	rows.push({
		id: OPTION.pvLmt, group: 'privacy', label: 'Limit ad tracking',
		description: pre + "The TV's own setting: apps are asked not to use its advertising ID to profile you.",
		value: !!tv && tv.lmt === 'on', valueText: shown(OPTION.pvLmt, onOff(tv ? tv.lmt : ''))
	});
	rows.push({
		id: OPTION.pvLivePlus, group: 'privacy', label: 'Live Plus',
		description: pre + "LG's screen recognition: it samples what is on screen, from apps and HDMI alike, to work out what you watch. Off is the private choice.",
		value: !!tv && tv.livePlus === 'on', valueText: shown(OPTION.pvLivePlus, onOff(tv ? tv.livePlus : ''))
	});
	rows.push({
		id: OPTION.pvHosts, group: 'privacy', label: 'Block LG ad servers',
		description: pre + (tv && !tv.hostsWritable ? "This TV's /etc/hosts cannot be written, so nothing can be blocked. " : '') + "Blocks LG's ad, tracking and screen recognition servers on the TV itself. The Content Store, app updates and streaming apps keep working.",
		value: !!tv && tv.hosts, valueText: shown(OPTION.pvHosts, tv && tv.hosts ? 'On' : 'Off')
	});
	rows.push({
		id: OPTION.pvServices, group: 'privacy', label: 'Stop LG tracking services',
		description: pre + "Stops LG's log uploaders, suggestion and screen recognition services, and stops them again at each start. Frees about 10 MB. LG's ad service stays, as some streaming apps wait for it." + (tv && tv.services && tv.servicesRunning ? ` ${tv.servicesRunning} still running.` : ''),
		value: !!tv && tv.services, valueText: shown(OPTION.pvServices, tv && tv.services ? 'On' : 'Off')
	});
	rows.push({
		id: OPTION.pvUpdates, group: 'privacy', label: 'Block TV software updates',
		description: pre + 'A software update can remove root, and with it Lumen\'s TV scripts. This is the Homebrew Channel\'s own switch, and takes effect after the TV restarts.',
		value: !!tv && tv.updates, valueText: shown(OPTION.pvUpdates, tv && tv.updates ? 'On' : 'Off')
	});
	return rows;
}

/** Settings > Side apps: each row's side app (docs/plans/side-apps.md) */
const SIDE_ROWS: {option: string; id: SideAppId; description: string}[] = [
	{option: OPTION.sideGuide, id: 'com.samrenfrew.lumen.guide', description: 'A channel guide over Live TV: LIST opens Now and Next, GUIDE the full guide. Off removes it from the TV.'},
	{option: OPTION.sideQuick, id: 'com.samrenfrew.lumen.quick', description: 'Sound, picture and backlight over any app, on the gear button. In Lumen the gear opens Control Center. Off removes it, and the gear is LG\'s again.'},
	{option: OPTION.sideSaver, id: 'com.samrenfrew.lumen.saver', description: 'Your screensaver photos over the app you are watching, so after a key the app is where you left it. Off removes it, and the screensaver opens Lumen as before.'}
];

/** Settings > Side apps: a row per side app (on when installed), then Update while any is on. */
function sideOptions (side: SideApps | null, busy: string): OptionItem[] {
	const working = (id: string) => busy === id;
	const rows: OptionItem[] = SIDE_ROWS.map(({option, id, description}) => {
		const have = side && side.installed[id];
		return {
			id: option, group: 'sideapps', label: SIDE_APP_TITLE[id], description, value: !!have,
			valueText: working(option) ? 'Working' : !side ? 'Checking' : have ? `On, ${have}` : 'Off'
		};
	});
	if (side && SIDE_ROWS.some((r) => side.installed[r.id])) {
		const n = behind(side).length;
		rows.push({
			id: OPTION.sideUpdate, group: 'sideapps', label: 'Update side apps',
			description: 'Gets the newest version of each side app that is on, from Lumen\'s releases on GitHub (the ones the Homebrew Channel lists), and installs it.',
			value: n > 0,
			valueText: working(OPTION.sideUpdate) ? 'Working' : !side.latest ? (side.offline ? 'Could not check' : 'Checking') : n ? `Update ${n} app${n > 1 ? 's' : ''}` : 'Up to date'
		});
	}
	return rows;
}

/** OK on a TV-script row. False when the row is not one of them. */
function onTvOption (id: string, tv: TvState | null, run: (id: string, job: () => Promise<string>) => void, toast: (text: string, kind?: 'info' | 'error') => void): boolean {
	if (id === OPTION.tvScripts) {
		if (!tv) { toast(isWebOS() ? 'Still checking the TV' : 'Only on the TV'); return true; }
		if (tv.scripts === 'unshipped') { toast('This build has no TV scripts in it', 'error'); return true; }
		run(id, () => installTvScripts(appDir()).then((ok) => ok ? 'TV scripts installed' : 'Could not install the TV scripts'));
		return true;
	}
	const home: Record<string, HomeAction> = {};
	const pv: Record<string, PrivacyAction> = {};
	if (id === OPTION.tvHome) home[id] = tv && LUMEN_IDS.indexOf(tv.home) >= 0 ? 'nativeOff' : 'nativeOn';
	else if (id === OPTION.tvPowerOn) home[id] = tv && LUMEN_IDS.indexOf(tv.powerOn) >= 0 ? 'powerOnOff' : 'powerOnOn';
	else if (id === OPTION.pvAgreements) pv[id] = tv && tv.agreements === 0 ? 'agreementsRestore' : 'agreementsOff';
	else if (id === OPTION.pvLmt) pv[id] = tv && tv.lmt === 'on' ? 'lmtOff' : 'lmtOn';
	else if (id === OPTION.pvLivePlus) pv[id] = tv && tv.livePlus === 'on' ? 'livePlusOff' : 'livePlusOn';
	else if (id === OPTION.pvHosts) pv[id] = tv && tv.hosts ? 'hostsOff' : 'hostsOn';
	else if (id === OPTION.pvServices) pv[id] = tv && tv.services ? 'servicesOff' : 'servicesOn';
	else if (id === OPTION.pvUpdates) pv[id] = tv && tv.updates ? 'updatesOff' : 'updatesOn';
	else if (id !== OPTION.tvRestart) return false;
	if (!tv) { toast(isWebOS() ? 'Still checking the TV' : 'Only on the TV'); return true; }
	if (!scriptsReady(tv)) { toast(NEEDS_SCRIPTS.trim()); return true; }
	if (id === OPTION.tvRestart) { run(id, () => reboot().then(() => 'Restarting the TV')); return true; }
	if (id === OPTION.tvHome && home[id] === 'nativeOn' && nativeWhy(tv)) { toast(nativeWhy(tv), 'error'); return true; }
	if (id === OPTION.tvPowerOn && home[id] === 'powerOnOn' && powerOnWhy(tv)) { toast(powerOnWhy(tv), 'error'); return true; }
	// what the settings service does with a key it never had is untried, so a setting the TV lacks is never written
	if ((id === OPTION.pvLmt && !tv.lmt) || (id === OPTION.pvLivePlus && !tv.livePlus)) { toast('This TV has no such setting'); return true; }
	if (id === OPTION.pvAgreements && tv.agreements === 0 && !tv.backup) { toast('Already withdrawn. LG\'s own menus can give them again.'); return true; }
	if (home[id]) run(id, () => homeSetup(home[id], APP_ID));
	else run(id, () => privacy(pv[id]));
	return true;
}

export function App () {
	// opened by watch.sh for the TV's screensaver: the first frame is the screensaver, so the home screen never shows first
	const [screen, setScreen] = useState<Screen>(() => import.meta.env.DEV && /[?&]settings/.test(location.search) ? 'settings' : startParams().view === 'ambient' && loadConfig().ambientOn ? 'ambient' : 'home');
	// Immich (Settings > Immich): whether a server is set up on the TV, and its albums; read again each time Settings opens
	const [immich, setImmich] = useState<{ready: boolean; albums: {id: string; title: string}[]}>({ready: !!cachedServer(), albums: []});
	// Settings > Immich: the address and key being typed, and how the last Test connection went
	const [immichDraft, setImmichDraft] = useState<ImmichServer>(() => cachedServer() || {url: '', key: ''});
	const [immichTest, setImmichTest] = useState<'' | 'testing' | 'failed'>('');
	useEffect(() => {
		if (screen !== 'settings') return;
		loadServer().then((srv) => {
			if (!srv) { setImmich({ready: false, albums: []}); return; }
			setImmichDraft(srv);
			immichAlbums(srv).then((a) => setImmich({ready: true, albums: a}), () => setImmich({ready: true, albums: []}));
		});
	}, [screen]);
	const [config, setConfig] = useState<UserConfig>(loadConfig);
	// set before anything renders, so every label below reads the chosen language and clock
	setLang(config.language);
	setClockMode(config.clockFormat);
	// a cold start draws what the last launch showed (lib/boot.ts); the TV is asked again once the first frame is up
	const [apps, setApps] = useState<AppEntry[]>(() => cachedApps(config.useHomebrew, config.showAllApps) || []);
	const [inputs, setInputs] = useState<InputSource[]>(() => (isWebOS() && readCache<InputSource[]>(INPUTS_CACHE_KEY)) || []);
	const [toast, setToast] = useState<ToastMessage | null>(null);
	const [controls, setControls] = useState(false);
	/** each press of the Guide button opens the Guide afresh, so it starts on Live TV's channel even when it was left open behind Live TV */
	const [guideOpens, setGuideOpens] = useState(0);
	const configRef = useRef(config);
	configRef.current = config;
	const toastTimer = useRef(0);
	// a fresh install first looks for the settings backup on the TV (lib/backup.ts); nothing is seeded until that has answered
	const [restoring, setRestoring] = useState(() => !hasSavedConfig());
	const [focusRow, setFocusRow] = useState<{id: string; seq: number} | undefined>(undefined);
	const live = useLiveTv(config.liveTvRow, screen === 'guide', config.liveChannels);

	const showToast = useCallback((text: string, kind: ToastMessage['kind'] = 'info') => {
		window.clearTimeout(toastTimer.current);
		setToast({text, kind});
		toastTimer.current = window.setTimeout(() => setToast(null), TIMING.toastMs);
	}, []);

	// Magic Remote scroll wheel = Up/Down (rows on the home screen, lists and grids elsewhere)
	useEffect(() => {
		let last = 0;
		const onWheel = (e: WheelEvent) => {
			const now = Date.now();
			if (now - last < 220 || Math.abs(e.deltaY) < 4) return;
			last = now;
			const code = e.deltaY > 0 ? 40 : 38;
			window.dispatchEvent(new KeyboardEvent('keydown', {keyCode: code, which: code, bubbles: true}));
		};
		window.addEventListener('wheel', onWheel, {passive: true});
		return () => window.removeEventListener('wheel', onWheel);
	}, []);

	useEffect(() => {
		if (!restoring) return;
		readBackup().then((saved) => {
			if (saved && !hasSavedConfig()) { updateConfig(saved); showToast('Settings restored from the TV'); }
			setRestoring(false);
		});
	}, []);

	// launch parameters: {"view":"recent"} opens on the app grid (it opened the Recent row, which has gone: every app is in the grid)
	useEffect(() => {
		let seq = 0;
		const apply = (p: LaunchParams) => {
			if (p.view === 'recent') setFocusRow({id: ROW.dock, seq: ++seq});
			// watch.sh opens Lumen with {"view":"ambient"} when the TV's screensaver starts (Settings > Screensaver), with the app it came up over in from
			if (p.view === 'ambient' && configRef.current.ambientOn) { ambientOpening.current = null; saverFrom.current = p.from || ''; setScreen('ambient'); }
			// {"view":"guide"} is the remote's Guide button, rebound with LG Input Hook; the Guide only has data with the Live TV row on
			if (p.view === 'guide' && configRef.current.liveTvRow) { setScreen('guide'); setControls(false); setGuideOpens((n) => n + 1); }
			// {"view":"controls"} is the remote's gear button, bound by watch.sh while Lumen is in front: it opens Control Center on home, or closes it
			if (p.view === 'controls') { saverFrom.current = ''; setScreen('home'); setControls((c) => !c); }
			// a plain relaunch is the remote's Home button: back to the home screen from Settings, the Guide, the screensaver or Control Center
			if (!p.view) { saverFrom.current = ''; setScreen('home'); setControls(false); }
		};
		apply(startParams());
		return onRelaunch(apply);
	}, []);

	// Start in remote (5-way) mode: hide the Magic Remote pointer whenever we come to the
	// front. Shaking the remote still brings it back.
	useEffect(() => hidePointerOnForeground(), []);

	// Load apps at start, whenever the Homebrew option changes, and each time Settings opens; refresh inputs whenever
	// the app comes back to the foreground (labels or connections may have changed).
	// With a list from the last launch on screen, the first read waits for the first frame; a changed option reads at once.
	const appsShown = useRef(apps);
	appsShown.current = apps;
	const firstAppsRead = useRef(true);
	const readApps = useRef<() => void>(() => undefined);
	useEffect(() => {
		const useHomebrew = config.useHomebrew;
		const showAll = config.showAllApps;
		const read = () => listApps(useHomebrew, showAll)
			.then(({apps: list, source}) => {
				bootMark('apps');
				if (!sameApps(list, appsShown.current)) setApps(list);
				saveAppsCache(useHomebrew, showAll, list);
				if (useHomebrew && source !== 'homebrew') showToast('Homebrew Channel not available, using the built-in app list', 'error');
			})
			.catch(() => showToast('Could not read the app list', 'error'));
		readApps.current = read;
		const wait = firstAppsRead.current && apps.length > 0;
		firstAppsRead.current = false;
		if (!wait) { read(); return undefined; }
		return afterFirstFrame(read);
	}, [config.useHomebrew, config.showAllApps, showToast]);
	// Lumen stays running for days, so an app installed meanwhile shows in Settings only if the list is read again there
	useEffect(() => { if (screen === 'settings') readApps.current(); }, [screen]);

	useEffect(() => {
		const refresh = () => {
			if (!isWebOS()) { setInputs(MOCK_INPUTS); return; }
			listInputs().then((list) => {
				writeCache(INPUTS_CACHE_KEY, list);
				setInputs((now) => (JSON.stringify(now) === JSON.stringify(list) ? now : list));
			}).catch(() => showToast('Could not read the inputs', 'error'));
		};
		const cancel = isWebOS() ? afterFirstFrame(refresh) : (refresh(), () => undefined);
		let timer = 0;
		const onVisibility = () => { window.clearTimeout(timer); if (!document.hidden) timer = window.setTimeout(refresh, RETURN_READ_MS); };
		document.addEventListener('visibilitychange', onVisibility);
		return () => { cancel(); window.clearTimeout(timer); document.removeEventListener('visibilitychange', onVisibility); };
	}, [showToast]);

	// the apps still open in the background (docs/keep-alive.md), read again 5 s after Lumen comes to the front: the TV may close some just then
	const [running, setRunning] = useState<string[]>([]);
	useEffect(() => {
		if (!isWebOS()) { setRunning(['youtube.leanback.v4', 'com.beb.plxnative']); return undefined; }
		let first = 0;
		let again = 0;
		const read = () => runningApps().then((ids) => ids.sort()).then((ids) => setRunning((now) => (now.join() === ids.join() ? now : ids))).catch(() => undefined);
		const readSoon = (wait: number) => {
			window.clearTimeout(first);
			window.clearTimeout(again);
			if (document.hidden) return;
			first = window.setTimeout(read, wait);
			again = window.setTimeout(read, 5000);
		};
		const cancel = afterFirstFrame(() => readSoon(0));
		const onVisibility = () => readSoon(RETURN_READ_MS);
		document.addEventListener('visibilitychange', onVisibility);
		return () => { cancel(); window.clearTimeout(first); window.clearTimeout(again); document.removeEventListener('visibilitychange', onVisibility); };
	}, []);

	useEffect(() => { setPicks(config.wallpaperPicks, config.wpNewOnLaunch); setWallpaper(config.wallpaper); }, [config.wallpaper]);
	useEffect(() => setPicks(config.wallpaperPicks, config.wpNewOnLaunch), [config.wallpaperPicks, config.wpNewOnLaunch]);

	// the weather under the clock at the top left
	const weatherCity = (c: UserConfig) => (c.weatherCity === AUTO_PLACE ? null : CITIES.filter((x) => x.name === c.weatherCity)[0] || null);
	// the last reading is shown at once on a cold start, however old, so the pill does not change size when the new one comes in
	const wantWeather = (c: UserConfig) => c.showWeather || c.homeForecast || c.saverForecast;
	const [weather, setWeather] = useState<Weather | null>(() => (wantWeather(config) && config.weatherCity && (config.weatherCity === AUTO_PLACE || weatherCity(config)) ? peekWeather(weatherCity(config)) : null));
	useEffect(() => {
		// no place chosen, no weather (and no request to anyone): Auto, which asks ipwho.is, has to be picked on purpose
		if (!wantWeather(config) || !config.weatherCity) { setWeather(null); return; }
		const city = weatherCity(config);
		if (!city && config.weatherCity !== AUTO_PLACE) { setWeather(null); return; }
		const load = () => readWeather(city).then(setWeather).catch(() => setWeather(null));
		const cancel = afterFirstFrame(load);
		const id = window.setInterval(load, 30 * 60000);
		return () => { cancel(); window.clearInterval(id); };
	}, [config.showWeather, config.homeForecast, config.saverForecast, config.weatherCity]);

	// a new wallpaper every N minutes, from the picks or the Immich sources
	useEffect(() => {
		if (!config.wpAutoMinutes || (config.wallpaper !== 'pick' && config.wallpaper.indexOf('immich:') !== 0)) return;
		const id = window.setInterval(() => { if (!document.hidden) setWallpaper(config.wallpaper); }, config.wpAutoMinutes * 60000);
		return () => window.clearInterval(id);
	}, [config.wpAutoMinutes, config.wallpaper]);

	// a backlight left low by a screensaver that never ended goes back up
	useEffect(() => { setBacklightSchedule(() => scheduledBacklight(configRef.current)); return afterFirstFrame(restoreLeftoverBacklight, 3000); }, []);
	// Settings > Brightness: watch.sh reads the schedule from a file, written after each change and once per launch (it may have been reinstalled)
	useEffect(() => afterFirstFrame(() => { writeBrightnessSchedule(configRef.current).catch(() => undefined); }, 3000),
		[config.brightnessOn, config.blDayAt, config.blDay, config.blEveningAt, config.blEvening, config.blNightAt, config.blNight, config.blRamp, config.blEye]);

	useHtmlClasses(config);
	// the Control Center's values are read once a return has caught up, so the reads do not slow the redraw
	useEffect(() => {
		let timer = 0;
		const onVisibility = () => {
			window.clearTimeout(timer);
			if (!document.hidden) timer = window.setTimeout(prefetchControls, 4000);
		};
		onVisibility();
		document.addEventListener('visibilitychange', onVisibility);
		return () => { window.clearTimeout(timer); document.removeEventListener('visibilitychange', onVisibility); };
	}, []);
	useEffect(() => setFpsCounter(__LUMEN_DEV__ && config.fpsCounter), [config.fpsCounter]);
	useEffect(() => setWhere(screen === 'guide' && config.guideFull ? 'full guide' : screen, controls), [screen, controls, config.guideFull]);
	useEffect(() => setLiquidDock(config.liquidDock), [config.liquidDock]);
	useEffect(() => setVideoChoice(config.videoPlaces), [config.videoPlaces]);
	// the video wallpaper plays only within its hours (checked each minute); the still wallpaper shows the rest of the time
	const [videoTime, setVideoTime] = useState(() => inVideoHours(config.videoHours));
	useEffect(() => {
		const check = () => setVideoTime(inVideoHours(config.videoHours));
		check();
		return onMinute(check);
	}, [config.videoHours]);
	const videoOn = config.videoWallpaper && videoTime;
	const videoOnRef = useRef(videoOn);
	videoOnRef.current = videoOn;
	// the wallpaper the screensaver opens on when started from the home screen; none when the TV's screensaver opens it
	const ambientOpening = useRef<ReturnType<typeof shownWallpaper>>(null);
	// the app the TV's screensaver came up over: a key that ends the screensaver goes back to it (a paused film), not to the home screen
	const saverFrom = useRef('');
	useEffect(() => setVideoWallpaper(videoOn), [videoOn]);
	useEffect(() => { setAmbientFlag(config.ambientOn); }, [config.ambientOn]);
	useEffect(() => { clearSaverOffFlag(); }, []);
	useEffect(() => { setVideoPaused(screen !== 'home'); document.documentElement.classList.toggle('in-settings', screen === 'settings' || screen === 'guide'); }, [screen]);
	const lastScreen = useRef(screen);
	useEffect(() => { if (lastScreen.current !== screen) navMark(`${lastScreen.current}>${screen}`); lastScreen.current = screen; }, [screen]);

	const updateConfig = (c: UserConfig) => { const next = withWallpaper(c); configRef.current = next; setConfig(next); saveConfig(next); scheduleBackup(next); };

	// first run: put the installed apps on the home screen (the catalogue's favourites first)
	useEffect(() => {
		const c = configRef.current;
		if (restoring || apps.length === 0) return;
		const builtins = (ids: string[]) => ids.concat(BUILTIN_APPS.map((b) => b.id).filter((id) => ids.indexOf(id) === -1));
		if (!c.seeded) updateConfig({...c, seeded: true, builtinTiles: true, appIds: builtins(seedAppIds(apps, DEFAULT_CONFIG.appIds))});
		// an install from before the built-in tiles: add them once at the end
		else if (!c.builtinTiles) updateConfig({...c, builtinTiles: true, appIds: builtins(c.appIds)});
	}, [apps, restoring]);
	const allApps = useMemo(() => withBuiltins(apps), [apps]);

	/** Lumen's screensaver straight away, without LG's showing first; LG's own when Lumen's is not chosen (unless forced, from Control Center). */
	const showScreensaver = (lumen = configRef.current.ambientOn) => {
		if (lumen) { ambientOpening.current = videoOnRef.current ? null : shownWallpaper(); saverFrom.current = ''; setScreen('ambient'); return; }
		startScreensaver().catch(() => showToast("Couldn't start the screensaver (it needs the Homebrew Channel)", 'error'));
	};
	/** A key ended the screensaver: back to the app it came up over, else the home screen. The Home button is a plain relaunch and always goes home. */
	const leaveAmbient = () => {
		const from = saverFrom.current;
		saverFrom.current = '';
		setScreen('home');
		if (from) launchApp(from).catch(() => undefined);
	};

	// idle screensaver: after the chosen time on the home screen without a key press or pointer move, while the launcher is in front
	useEffect(() => {
		if (!config.ambientOn || !config.idleSeconds || videoOn || screen !== 'home' || controls) return;   // no screensaver of our own while the video plays
		let timer = 0;
		const arm = () => {
			window.clearTimeout(timer);
			timer = window.setTimeout(() => { if (!document.hidden) showScreensaver(); }, config.idleSeconds * 1000);
		};
		arm();
		const events = ['keydown', 'mousemove', 'click'];
		events.forEach((e) => window.addEventListener(e, arm));
		return () => { window.clearTimeout(timer); events.forEach((e) => window.removeEventListener(e, arm)); };
	}, [config.ambientOn, config.idleSeconds, videoOn, screen, controls]);
	// the screensaver's first photo is loaded while the home screen is up, so the idle timer and Control Center's Art mode both open on it at once
	useEffect(() => {
		if (screen !== 'home') return;
		const timer = window.setTimeout(() => { if (!document.hidden) prepareAmbient(config.saverShow, config.ambient, config.wallpaperPicks); }, PREPARE_SAVER_MS);
		return () => window.clearTimeout(timer);
	}, [screen, config.saverShow, config.ambient, config.wallpaperPicks]);
	useEffect(() => { setVideoFlag(videoOn); }, [videoOn]);

	// ----- derived rows -----
	const shownApps = useMemo(() => pickInOrder(allApps, config.appIds, (a) => a.id), [allApps, config.appIds]);
	const shownInputs = useMemo(
		() => (config.sourceIds ? pickInOrder(inputs, config.sourceIds, (s) => s.appId) : inputs),
		[inputs, config.sourceIds]
	);


	const folderTiles = useMemo(
		() => config.folders.map((f) => ({name: f.name, items: pickInOrder(allApps, f.appIds, (a) => a.id).map(appTile)})).filter((f) => f.items.length > 0),
		[allApps, config.folders]
	);
	const folderMap = useMemo(() => {
		const map: Record<string, TileModel[]> = {};
		folderTiles.forEach((f) => { map[f.name] = f.items; });
		return map;
	}, [folderTiles]);
	/** what the home rows show: the apps that are not in a folder, then one tile per folder */
	const homeTiles = useMemo(() => {
		const inFolder = config.folders.reduce((all, f) => all.concat(f.appIds), [] as string[]);
		const open = (id: string) => running.indexOf(id) !== -1;
		return shownApps.filter((a) => inFolder.indexOf(a.id) === -1).map((a): TileModel => ({...appTile(a), running: open(a.id)}))
			.concat(folderTiles.map((f): TileModel => ({key: 'folder:' + f.name, label: f.name, color: '#3b4256', children: f.items.map((t) => t.imageUrl), running: f.items.some((t) => open(t.key))})));
	}, [shownApps, folderTiles, config.folders, running]);

	// the first row (in the glass bar) holds config.firstRow apps; the rest make a grid of rows as wide below it, like the tvOS home screen
	// (the grid rows are 'dock', 'dock:1', 'dock:2'...)
	const firstRow = config.firstRow;
	/** the Live TV row: the Guide tile, then the channels picked in Settings > Channels (or the first few), each with what is on now */
	// dev builds: the focused channel tile plays a live preview while Home is in front (components/LivePreview.tsx)
	const homePreview = __LUMEN_DEV__ && config.homePreview && screen === 'home' && !controls ? {sound: config.guideSound} : undefined;
	const homePreviewKey = homePreview ? String(homePreview.sound) : '';
	const liveTiles = useMemo((): TileModel[] => {
		if (!config.liveTvRow || !live.channels.length) return [];
		const guide: TileModel = {key: LIVE.guide, label: 'Guide', glyph: 'guide', color: 'rgba(255, 255, 255, 0.16)'};
		return [guide].concat(rowChannels(live.channels, config.liveChannels).map((c): TileModel => ({
			key: LIVE.prefix + c.id,
			label: c.name,
			live: {channel: c, number: c.number, name: c.name, logo: c.logoPath && live.logos[c.logoPath], at: live.now, loading: !!live.pending[c.signal], preview: homePreview, ...nowNextAt(live.listings[c.signal], live.now)}
		})));
	}, [config.liveTvRow, config.liveChannels, live.channels, live.listings, live.logos, live.now, live.pending, config.language, homePreviewKey]);
	/** the furthest along the row a focused channel may sit with its now/next block still on the row (Row's slot) */
	const liveSlot = useMemo(() => {
		const t = TILES_FOR[firstRow] || TILES_FOR[6];
		return Math.max(0, Math.floor((firstRow * (t.size + t.gap) - t.gap - LIVE_INFO_W) / (t.size + t.gap)));
	}, [firstRow]);
	const rows: HomeRow[] = useMemo(() => {
		const list: HomeRow[] = [{id: ROW.apps, label: 'Apps', items: homeTiles.slice(0, firstRow), emptyText: 'No apps selected — add some in Settings'}];
		for (let at = firstRow, n = 0; at < homeTiles.length; at += firstRow, n++) list.push({id: n ? `${ROW.dock}:${n}` : ROW.dock, label: 'More apps', items: homeTiles.slice(at, at + firstRow)});
		if (liveTiles.length) list.push({id: ROW.live, label: 'Live TV', titled: true, className: 'row--live', below: LIVE_BELOW, slot: liveSlot, items: liveTiles});
		return list;
	}, [homeTiles, shownInputs, config.language, firstRow, liveTiles, liveSlot]);

	// Settings' tabs are only built while Settings is open
	const inSettings = screen === 'settings';

	// Settings > Home and Power, Privacy and System > TV scripts: the TV's state, read when Settings opens and after each change (lib/tvsetup.ts)
	const [tv, setTv] = useState<TvState | null>(null);
	const [tvBusy, setTvBusy] = useState('');
	const tvBusyRef = useRef('');
	useEffect(() => { if (inSettings) readTvState().then(setTv); }, [inSettings]);
	/** Runs one TV-script change: the row shows Working until it is done, then the script's last line shows as a toast and the state is read again. */
	const runTv = useCallback((id: string, job: () => Promise<string>) => {
		if (tvBusyRef.current) return;
		tvBusyRef.current = id;
		setTvBusy(id);
		const done = () => readTvState().then((s) => { setTv(s); tvBusyRef.current = ''; setTvBusy(''); });
		job().then((out) => {
			const lines = out.split('\n').map((l) => l.trim()).filter(Boolean);
			const last = lines[lines.length - 1] || 'Done';
			showToast(last.length > 160 ? last.slice(0, 157) + '...' : last, /refused|could not|cannot|not ours|no node|failed|left .* on/i.test(last) ? 'error' : 'info');
		}, (e: unknown) => showToast(e instanceof Error ? e.message : String(e), 'error')).then(done, done);
	}, [showToast]);
	// Settings > Side apps: what is installed and the newest releases, read when Settings opens and after each change (lib/sideapps.ts)
	const [side, setSide] = useState<SideApps | null>(null);
	const readSide = useCallback(() => { readSideApps().then(setSide).catch(() => undefined); }, []);
	useEffect(() => { if (inSettings) readSide(); }, [inSettings]);
	const tabs: SettingsTab[] = useMemo(() => {
		if (!inSettings) return [];
		const list: SettingsTab[] = [
			{id: ROW.apps, label: 'Apps', glyph: 'grid', noun: 'apps', items: allApps.map(appTile), shown: shownApps.map(appTile)},
			{id: ROW.sources, label: 'Sources', glyph: 'inputs', noun: 'sources', items: inputs.map(inputTile), shown: shownInputs.map(inputTile)}
		];
		// the Live TV row's channels: every channel, the row's ticked, in the row's order
		if (config.liveTvRow && live.channels.length) {
			const tile = (c: Channel): TileModel => {
				const logo = c.logoPath && live.logos[c.logoPath];
				return {key: c.id, label: `${c.number} · ${c.name}`, imageUrl: logo || undefined, plain: true, color: `linear-gradient(160deg, rgba(255, 255, 255, 0.1) 0%, rgba(0, 0, 0, 0.28) 100%) ${channelColor(c.name) || '#22263a'}`, wordmark: logo ? undefined : c.name};
			};
			list.push({id: ROW.channels, label: 'Channels', glyph: 'guide', noun: 'channels', items: live.channels.map(tile), shown: rowChannels(live.channels, config.liveChannels).map(tile)});
		}
		return list;
	}, [inSettings, allApps, inputs, shownApps, shownInputs, config.language, config.liveTvRow, config.liveChannels, live.channels, live.logos]);

	/** options that pick from a list: the labels shown in the pop-up list, the stored values, and which one is set now */
	type Choice = {labels: string[]; values: (string | number)[]; now: string | number};
	const choices: Record<string, Choice> = useMemo(() => {
		const country = COUNTRIES.filter((c) => c.code === config.weatherCountry)[0];
		const cities = (country ? country.cities : CITIES).map((c) => c.name);
		return {
			[OPTION.country]: {labels: ['Any'].concat(COUNTRIES.map((c) => c.name)), values: [''].concat(COUNTRIES.map((c) => c.code)), now: config.weatherCountry},
			[OPTION.city]: {labels: ['Not set', 'Auto'].concat(cities), values: ['', AUTO_PLACE].concat(cities), now: config.weatherCity},
			// Immich is a choice only once it is connected (Settings > Immich)
			[OPTION.wpType]: immich.ready
				? {labels: ['Pictures', 'Video', 'Immich photos', 'Plain'], values: ['pictures', 'video', 'immich', 'plain'], now: config.wallpaperType}
				: {labels: ['Pictures', 'Video', 'Plain'], values: ['pictures', 'video', 'plain'], now: config.wallpaperType},
			[OPTION.immichDays]: {labels: ['Last month', 'Last 3 months', 'Last 6 months', 'Last year'], values: [30, 90, 180, 365], now: config.immichRecentDays},
			[OPTION.wpAuto]: {labels: ['Off', '30 sec', '1 min', '5 min', '15 min', '30 min', '60 min'], values: WP_AUTO_CHOICES, now: config.wpAutoMinutes},
			[OPTION.dockHide]: {labels: ['Off', 'Down arrow', 'After idle', 'Both'], values: ['off', 'down', 'idle', 'both'], now: config.dockHide},
			[OPTION.ambientOn]: {labels: ['Enabled', 'Disabled'], values: ['on', 'off'], now: config.ambientOn ? 'on' : 'off'},
			[OPTION.saverShow]: {labels: ['Wallpaper pictures', 'Immich photos'], values: ['wallpaper', 'immich'], now: config.saverShow},
			[OPTION.idle]: {labels: ['Never', '30 seconds', '1 minute', '2 minutes', '5 minutes'], values: IDLE_SECONDS, now: config.idleSeconds},
			[OPTION.darkMode]: {labels: ['Light', 'Dark', 'Auto'], values: ['off', 'on', 'auto'], now: config.darkMode},
			[OPTION.language]: {labels: ['Auto'].concat(LANGS.map((l) => l.name)), values: ['auto'].concat(LANGS.map((l) => l.code)), now: config.language},
			[OPTION.clock]: {labels: ['Auto', '12-hour', '24-hour'], values: ['auto', '12h', '24h'], now: config.clockFormat},
			[OPTION.homeCast]: {labels: ['Off', 'Small', 'Large'], values: ['off', 'small', 'large'], now: config.homeForecast ? config.homeForecastSize : 'off'},
			[OPTION.saverCast]: {labels: ['Off', 'Small', 'Large'], values: ['off', 'small', 'large'], now: config.saverForecast ? config.saverForecastSize : 'off'},
			[OPTION.focus]: {labels: ['Subtle', 'Normal', 'Strong'], values: ['subtle', 'normal', 'strong'], now: config.focusStrength},
			[OPTION.firstRow]: {labels: ['5', '6', '7'], values: [5, 6, 7], now: config.firstRow},
			[OPTION.ambient]: {labels: IMMICH_SOURCES.map((x) => x.title).concat(immich.albums.map((a) => a.title)), values: IMMICH_SOURCES.map((x) => x.key).concat(immich.albums.map((a) => `immich:album:${a.id}`)), now: config.ambient},
			[OPTION.ambientSeconds]: {labels: ['10 seconds', '20 seconds', '30 seconds', '1 minute', '2 minutes', '5 minutes'], values: [10, 20, 30, 60, 120, 300], now: config.ambientSeconds},
			[OPTION.ambientBrightness]: {labels: ['As the TV is set', 'Dim', 'Very dim'], values: ['tv', 'dim', 'low'], now: config.ambientBrightness},
			[OPTION.nightDim]: {labels: ['Off', 'At night', 'Always'], values: ['off', 'night', 'always'], now: config.wallpaperDim},
			[OPTION.videoHours]: {labels: ['Always', 'Daytime (7 am to 7 pm)', 'Evenings (5 pm to 11 pm)', 'Night (7 pm to 7 am)'], values: ['always', 'day', 'evening', 'night'], now: config.videoHours},
			[OPTION.guidePreview]: {labels: ['Off', 'On'], values: ['off', 'on'], now: config.guidePreview ? 'on' : 'off'},
			[OPTION.homePreview]: {labels: ['Off', 'On'], values: ['off', 'on'], now: config.homePreview ? 'on' : 'off'},
			[OPTION.guideSound]: {labels: ['Muted', 'On'], values: ['off', 'on'], now: config.guideSound ? 'on' : 'off'},
			[OPTION.blOn]: {labels: ['Off', 'On'], values: ['off', 'on'], now: config.brightnessOn ? 'on' : 'off'},
			[OPTION.blDayAt]: {labels: halfHours(5, 12), values: halfHours(5, 12), now: config.blDayAt},
			[OPTION.blEveningAt]: {labels: halfHours(15, 22), values: halfHours(15, 22), now: config.blEveningAt},
			[OPTION.blNightAt]: {labels: halfHours(19, 26), values: halfHours(19, 26), now: config.blNightAt},
			[OPTION.blDay]: {labels: BL_LEVELS.map(String), values: BL_LEVELS, now: config.blDay},
			[OPTION.blEvening]: {labels: BL_LEVELS.map(String), values: BL_LEVELS, now: config.blEvening},
			[OPTION.blNight]: {labels: BL_LEVELS.map(String), values: BL_LEVELS, now: config.blNight},
			[OPTION.blRamp]: {labels: ['At once', 'Over 30 minutes', 'Over 1 hour', 'Over 1½ hours', 'Over 2 hours'], values: [0, 30, 60, 90, 120], now: config.blRamp},
			[OPTION.blEye]: {labels: ['As the TV is set', 'Off all day', 'Evening and night', 'Night only'], values: ['keep', 'off', 'evening', 'night'], now: config.blEye}
		};
	}, [config, immich]);
	/** the list fields of an option: its choices, which is set, and the text shown on the right */
	const listed = (id: string, fallback = '') => {
		const c = choices[id];
		const at = c.values.indexOf(c.now);
		return {choices: c.labels, choice: Math.max(0, at), valueText: at >= 0 ? c.labels[at] : fallback || c.labels[0]};
	};
	/** the screensaver's sources keep several (immich:recent|immich:favorites), which lib/immich.ts mixes */
	const ambientPicks = () => {
		const c = choices[OPTION.ambient];
		const picks = config.ambient.split('|').map((k) => c.values.indexOf(k)).filter((i) => i >= 0);
		return {choices: c.labels, choice: picks.length > 0 ? picks[0] : 0, picks, valueText: picks.length === 1 ? c.labels[picks[0]] : `${picks.length} sources`};
	};

	// each page lists its options in this order (Settings.tsx GROUPS sets the order of the pages)
	const options: OptionItem[] = useMemo(() => {
		const stills = config.wallpaperType === 'pictures' || config.wallpaperType === 'immich';
		const saverImmich = immich.ready && config.saverShow === 'immich';
		return [
			// ----- Home Screen -----
			{
				id: OPTION.liveTv,
				group: 'home' as const,
				label: 'Live TV row',
				description: 'A row of TV channels under your apps, showing what is on now and next, with the Guide at the end. OK on a channel opens Live TV on it. Settings > Channels picks the channels.',
				value: config.liveTvRow
			},
			{
				id: OPTION.firstRow,
				group: 'home' as const,
				label: 'Apps per row',
				description: 'How many apps a row holds: the dock, and each row of the grid below it. With 7 the tiles are a little smaller, with 5 a little bigger.',
				value: true,
				...listed(OPTION.firstRow)
			},
			{
				id: OPTION.hideNames,
				group: 'home' as const,
				label: 'Hide app names',
				description: 'No names under the apps on the home screen, not even the one in focus. Settings still shows them.',
				value: config.hideNames
			},
			{
				id: OPTION.showRecents,
				group: 'home' as const,
				label: 'Show recents indicator',
				description: 'A dot over the apps that are open.',
				value: config.showRecents
			},
			{
				id: OPTION.dockHide,
				group: 'home' as const,
				label: 'Hide the dock',
				description: 'Down arrow: Down on the last row slides the dock away. After idle: it hides itself after 30 seconds without a key press. Any key brings it back.',
				value: config.dockHide !== 'off',
				...listed(OPTION.dockHide)
			},
			...(config.dockHide !== 'off' ? [{
				id: OPTION.dockDim,
				group: 'home' as const,
				label: 'Dim when the dock is hidden',
				description: 'With the dock away, the wallpaper fades darker. It brightens again when the dock comes back.',
				value: config.dockDim
			}, {
				id: OPTION.captions,
				group: 'home' as const,
				label: 'Wallpaper captions',
				description: 'With the dock away, the picture\'s title and artist, or the video\'s place, show at the bottom right.',
				value: config.wallpaperCaptions
			}] : []),
			{
				id: OPTION.weather,
				group: 'home' as const,
				label: 'Date and weather',
				description: 'Today\'s date and the temperature at the top left. The temperature needs a Weather city in Settings > Language and Region; it comes from open-meteo.com.',
				value: config.showWeather
			},
			{
				id: OPTION.homeCast,
				group: 'home' as const,
				label: 'Weather widget',
				description: 'The next few hours of weather, above the dock on the left. Small: the next 6 hours. Large: the weather now with today\'s high and low, and the next 7 hours with the chance of rain. Needs a Weather city.',
				value: config.homeForecast,
				...listed(OPTION.homeCast)
			},

			// ----- Live TV (dev builds while the live preview is tried) -----
			...(__LUMEN_DEV__ && config.liveTvRow ? [{
				id: OPTION.guidePreview,
				group: 'livetv' as const,
				label: 'Preview in the Guide',
				description: 'A live picture of the channel in focus in the Guide. It starts when you stop on a channel for a moment, so flicking through does not tune each one.',
				value: config.guidePreview,
				...listed(OPTION.guidePreview)
			}, {
				id: OPTION.homePreview,
				group: 'livetv' as const,
				label: 'Preview on the Live TV row',
				description: 'On the home screen, the channel in focus on the Live TV row turns into its live picture after a moment.',
				value: config.homePreview,
				...listed(OPTION.homePreview)
			}, ...(config.guidePreview || config.homePreview ? [{
				id: OPTION.guideSound,
				group: 'livetv' as const,
				label: 'Sound',
				description: 'Play the channel\'s sound with the preview, or keep it silent.',
				value: config.guideSound,
				...listed(OPTION.guideSound)
			}] : [])] : []),

			// ----- Wallpaper -----
			{
				id: OPTION.wpType,
				group: 'wallpaper' as const,
				label: 'Wallpaper type',
				description: immich.ready ? 'Pictures from the library, a video, your Immich photos, or a plain background. Choose wallpaper then picks which ones.' : 'Pictures from the library, a video, or a plain background. Choose wallpaper then picks which ones. Connect Immich in Settings > Immich to use your own photos.',
				value: true,
				...listed(OPTION.wpType)
			},
			...(stills ? [{
				id: OPTION.wpAuto,
				group: 'wallpaper' as const,
				label: 'Change wallpaper automatically',
				description: 'Shows another of your pictures every 30 seconds to 60 minutes.',
				value: config.wpAutoMinutes > 0,
				...listed(OPTION.wpAuto)
			}, {
				id: OPTION.wpNew,
				group: 'wallpaper' as const,
				label: 'New picture on startup',
				description: 'Opens on another of your pictures each time. Off, it opens on the last one it showed.',
				value: config.wpNewOnLaunch
			}] : []),
			...(config.wallpaperType === 'video' ? [{
				id: OPTION.videoHours,
				group: 'wallpaper' as const,
				label: 'Video wallpaper hours',
				description: 'When the video wallpaper plays. Outside these hours the still wallpaper shows instead, and nothing streams.',
				value: config.videoHours !== 'always',
				...listed(OPTION.videoHours)
			}] : []),
			{
				id: OPTION.nightDim,
				group: 'wallpaper' as const,
				label: 'Dim the wallpaper',
				description: 'A darker wallpaper, easier on the eyes in a dark room. At night: between 7 pm and 6 am.',
				value: config.wallpaperDim !== 'off',
				...listed(OPTION.nightDim)
			},

			// ----- Screensaver -----
			{
				id: OPTION.ambientOn,
				group: 'screensaver' as const,
				label: 'Screensaver',
				description: 'Enabled: this app\'s screensaver replaces the TV\'s, a slow crossfade of pictures with the clock; any key returns to the home screen. Disabled: the TV\'s default LG screensaver runs. The TV\'s own auto power-off still applies.',
				value: config.ambientOn,
				...listed(OPTION.ambientOn)
			},
			// Disabled: LG's screensaver runs, so none of this app's screensaver options apply
			...(config.ambientOn ? [
				{
					id: OPTION.idle,
					group: 'screensaver' as const,
					label: 'Start on the home screen after',
					description: 'Starts the screensaver when the home screen has been left alone this long. The TV\'s own screensaver timer still covers everything else.',
					value: config.idleSeconds > 0,
					...listed(OPTION.idle)
				},
				...(immich.ready ? [{
					id: OPTION.saverShow,
					group: 'screensaver' as const,
					label: 'Show',
					description: 'What the screensaver shows. Wallpaper pictures: the pictures picked in Wallpaper > Choose wallpaper. Immich photos: photos from your Immich server.',
					value: true,
					...listed(OPTION.saverShow)
				}] : []),
				...(saverImmich ? [{
					id: OPTION.ambient,
					group: 'screensaver' as const,
					label: 'Immich photos',
					description: 'Which Immich photos the screensaver shows. Pick several and it mixes them.',
					value: true,
					...ambientPicks()
				}] : []),
				{
					id: OPTION.ambientSeconds,
					group: 'screensaver' as const,
					label: 'Time on each picture',
					description: 'How long each picture stays on the screensaver before the next one fades in.',
					value: true,
					...listed(OPTION.ambientSeconds)
				}, {
					id: OPTION.ambientBrightness,
					group: 'screensaver' as const,
					label: 'Brightness',
					description: 'Turns the TV\'s backlight down while the screensaver shows, and back up when you press a key.',
					value: config.ambientBrightness !== 'tv',
					...listed(OPTION.ambientBrightness)
				}, {
					id: OPTION.ambientPrint,
					group: 'screensaver' as const,
					label: 'Print look',
					description: 'Makes the pictures look like prints on a wall rather than a glowing screen: softer blacks, paper-white whites and a gentle fall of light.',
					value: config.ambientPrint
				}, {
					id: OPTION.saverCast,
					group: 'screensaver' as const,
					label: 'Weather widget',
					description: 'The next few hours of weather on the screensaver, at the bottom between the clock and the date. Small: the next 6 hours. Large: the weather now with today\'s high and low, and the next 7 hours with the chance of rain. Needs a Weather city.',
					value: config.saverForecast,
					...listed(OPTION.saverCast)
				}
			] : []),

			// ----- Brightness -----
			{
				id: OPTION.blOn,
				group: 'brightness' as const,
				label: 'Brightness schedule',
				description: 'Sets the TV\'s brightness at the start of the day, the evening and the night, and Eye Comfort if you choose. It works while Lumen is closed, and needs the TV scripts (Settings > System). A change by hand stays until the next part of the day. Turn off anything else that sets the brightness on a timer, such as Home Assistant, so the two do not fight.',
				value: config.brightnessOn,
				...listed(OPTION.blOn)
			},
			...(config.brightnessOn ? [
				{id: OPTION.blDayAt, group: 'brightness' as const, label: 'Day starts', description: 'When the day brightness is set.', value: true, ...listed(OPTION.blDayAt)},
				{id: OPTION.blDay, group: 'brightness' as const, label: 'Day brightness', description: 'The TV\'s backlight through the day, from 0 to 100.', value: true, ...listed(OPTION.blDay)},
				{id: OPTION.blEveningAt, group: 'brightness' as const, label: 'Evening starts', description: 'When the evening brightness is set.', value: true, ...listed(OPTION.blEveningAt)},
				{id: OPTION.blEvening, group: 'brightness' as const, label: 'Evening brightness', description: 'The TV\'s backlight through the evening, from 0 to 100.', value: true, ...listed(OPTION.blEvening)},
				{id: OPTION.blNightAt, group: 'brightness' as const, label: 'Night starts', description: 'When the night brightness is set. It holds until the day starts again.', value: true, ...listed(OPTION.blNightAt)},
				{id: OPTION.blNight, group: 'brightness' as const, label: 'Night brightness', description: 'The TV\'s backlight through the night, from 0 to 100.', value: true, ...listed(OPTION.blNight)},
				{id: OPTION.blRamp, group: 'brightness' as const, label: 'Change', description: 'How each part of the day moves to its brightness. At once: it jumps when that part starts. Over a time: it starts at the last part\'s brightness and changes a little each minute until it gets there. While it changes, a change by hand lasts only a minute.', value: config.blRamp > 0, ...listed(OPTION.blRamp)},
				{id: OPTION.blEye, group: 'brightness' as const, label: 'Eye Comfort', description: 'Eye Comfort makes the picture warmer. As the TV is set: the schedule leaves it alone. Otherwise it is turned on and off with the parts of the day.', value: config.blEye !== 'keep' && config.blEye !== 'off', ...listed(OPTION.blEye)}
			] : []),

			// ----- Appearance -----
			{
				id: OPTION.darkMode,
				group: 'appearance' as const,
				label: 'Appearance',
				description: 'Light: bright glass with dark text. Dark: dark glass and a dimmer wallpaper or video. Auto is dark from 7 pm to 6 am.',
				value: config.darkMode !== 'off',
				...listed(OPTION.darkMode)
			},
			{
				id: OPTION.focus,
				group: 'appearance' as const,
				label: 'Focus highlight',
				description: 'How much the app in focus stands out: how far it grows and how strong its shadow and shine are.',
				value: config.focusStrength !== 'normal',
				...listed(OPTION.focus)
			},
			{
				id: OPTION.reduceMotion,
				group: 'appearance' as const,
				label: 'Reduce motion',
				description: 'Focus, rows and panels change at once, without sliding, fading or zooming.',
				value: config.reduceMotion
			},
			{
				id: OPTION.dockBlur,
				group: 'appearance' as const,
				label: 'Dock blur',
				description: 'The dock blurs whatever is behind it. Off: the same glass look with no blur, and moving left and right is smoother.',
				value: config.dockBlur
			},

			// ----- Photos -----
			{
				id: OPTION.immichUrl,
				group: 'photos' as const,
				label: 'Server address',
				description: 'Your Immich server as you would open it in a browser at home: scheme, address and port, for example http://192.168.1.10:2283.',
				value: immichDraft.url !== '',
				valueText: immichDraft.url || 'Not set',
				text: {value: immichDraft.url, placeholder: 'http://192.168.1.10:2283'}
			},
			{
				id: OPTION.immichKey,
				group: 'photos' as const,
				label: 'API key',
				description: 'In Immich: Account Settings > API Keys > New API Key, with only album.read, asset.read, asset.view, memory.read, face.read and stack.read. Read-only, so it can only show photos.',
				value: immichDraft.key !== '',
				valueText: immichDraft.key ? '\u2022\u2022\u2022\u2022 ' + immichDraft.key.slice(-4) : 'Not set',
				text: {value: immichDraft.key, placeholder: 'The key from Immich'}
			},
			{
				id: OPTION.immichTest,
				group: 'photos' as const,
				label: 'Test connection',
				description: 'Checks the address and the key with your server. When it passes, Lumen saves them on the TV and your Immich photos can be the wallpaper and the screensaver.',
				value: immich.ready,
				valueText: immichTest === 'testing' ? 'Testing' : immichTest === 'failed' ? 'Failed' : immich.ready ? 'Connected' : 'Not connected'
			},
			...(immich.ready ? [{
				id: OPTION.immichDays,
				group: 'photos' as const,
				label: 'Recent photos from',
				description: 'How far back the Recent and Recent, with people sources reach, for the wallpaper and the screensaver.',
				value: true,
				...listed(OPTION.immichDays)
			}] : []),

			// ----- Language and Region -----
			{
				id: OPTION.language,
				group: 'region' as const,
				label: 'Language',
				description: "Language of the menus, dates and weather. Auto follows the TV. The long descriptions on this screen stay in English.",
				value: config.language !== 'auto',
				...listed(OPTION.language)
			},
			{
				id: OPTION.clock,
				group: 'region' as const,
				label: 'Clock',
				description: "Auto follows the TV's own 12/24-hour setting. Choose 12-hour or 24-hour to override it.",
				value: config.clockFormat !== 'auto',
				...listed(OPTION.clock)
			},
			...(config.clockFormat !== '24h' ? [{
				id: OPTION.showAmPm,
				group: 'region' as const,
				label: 'AM/PM on the clock',
				description: 'Show AM or PM beside the time when the clock is 12-hour.',
				value: config.showAmPm
			}] : []),
			{
				id: OPTION.country,
				group: 'region' as const,
				label: 'Weather country',
				description: 'Pick your country, then the city below. Until a city is chosen no weather is shown and nothing is fetched.',
				value: config.weatherCountry !== '',
				...listed(OPTION.country)
			},
			{
				id: OPTION.city,
				group: 'region' as const,
				label: 'Weather city',
				description: 'Not set: no weather. Auto finds your area from your internet connection, which sends your IP address to ipwho.is and can be a few towns off. Or pick a city.',
				value: config.weatherCity !== '',
				...listed(OPTION.city, config.weatherCity)
			},

			// ----- Home and Power, Privacy (the TV scripts: tools/tv/home-setup.sh, tools/tv/privacy.sh) -----
			...tvOptions(tv, tvBusy),

			// ----- Side apps (docs/plans/side-apps.md) -----
			...sideOptions(side, tvBusy),

			// ----- System -----
			{
				id: OPTION.tvScripts,
				group: 'system' as const,
				label: 'TV scripts',
				description: "The scripts Lumen runs on the TV as root: the Home button watcher, Home and Power, and Privacy. They ship inside Lumen; this copies this build's into place and restarts them. Needs the Homebrew Channel with root.",
				value: !!tv && tv.scripts === 'ok',
				valueText: tvBusy === OPTION.tvScripts ? 'Working' : !tv ? (isWebOS() ? 'Checking' : 'Not on a TV') : tv.scripts === 'ok' ? 'Up to date' : tv.scripts === 'old' ? 'Update' : tv.scripts === 'missing' ? 'Install' : 'Not in this build'
			},
			{
				id: OPTION.showAll,
				group: 'system' as const,
				label: 'Show system apps',
				description: 'Offer the TV\'s own apps (Live TV, LG Channels, AirPlay, Screen Share, Game Optimizer, TV Settings) in Settings > Apps, so you can add them to the home screen.',
				value: config.showAllApps
			},
			{
				id: OPTION.restore,
				group: 'system' as const,
				label: 'Restore settings from the TV',
				description: 'Lumen keeps a copy of its settings on the TV (/var/lib/homehook/lumen-config.json) after every change. This puts that copy back, replacing the settings here.',
				value: false,
				valueText: 'Restore'
			},
			...(__LUMEN_DEV__ ? [{
				id: OPTION.fps,
				group: 'system' as const,
				label: 'Frame-rate counter (dev builds)',
				description: 'Frames drawn in the last second and the longest frame, in the top-left corner. Only in dev builds.',
				value: config.fpsCounter
			}, {
				id: OPTION.liquidDock,
				group: 'system' as const,
				label: 'Liquid glass dock (dev builds)',
				description: 'The dock bends the picture at its edges, like Liquid Glass on Apple TV. Too slow for everyday use on this TV, so only dev builds offer it.',
				value: config.liquidDock
			}] : [])
		];
	}, [config, choices, immich.ready, immichDraft, immichTest, tv, tvBusy, side]);

	const wallpaper: WallpaperTab = useMemo(() => {
		if (!inSettings) return {categories: [], entries: [], onPick: () => undefined};
		const entries: WallpaperEntry[] = [];
		const type = config.wallpaperType;
		if (type === 'video') {
			const tint = (i: number) => `linear-gradient(150deg, ${["#e9a15b,#6d3d4a", "#d8b46a,#3e5a78", "#9fc2d6,#2d4a6b", "#7fc79a,#1e4a52", "#e07f8a,#4a2d5e", "#c9a0dc,#2f2f6b", "#f0c16a,#8a4a2a", "#8ab0e0,#1d2f5a", "#d6a07a,#4a3430", "#86c5c0,#27495a"][i % 10]})`;
			entries.push({key: 'video:', title: 'All places', category: 'Video', tint: tint(0), on: config.videoPlaces.length === 0});
			VIDEO_PLACES.forEach((p, i) => entries.push({key: `video:${p}`, title: p, category: 'Video', tint: tint(i + 1), on: config.videoPlaces.indexOf(p) >= 0}));
			return {
				categories: ['Video'],
				entries,
				summary: 'Pick the places the video wallpaper plays.',
				onPick: (key: string) => {
					const place = key.slice(6);
					updateConfig({...config, videoPlaces: !place ? [] : config.videoPlaces.indexOf(place) >= 0 ? config.videoPlaces.filter((x) => x !== place) : config.videoPlaces.concat(place)});
				}
			};
		}
		if (type === 'immich') {
			if (immich.ready) {
				IMMICH_SOURCES.forEach((src) => entries.push({key: src.key, title: src.title, category: 'Immich', on: config.immichPicks.indexOf(src.key) >= 0}));
				immich.albums.forEach((a) => entries.push({key: `immich:album:${a.id}`, title: a.title, category: 'Immich', on: config.immichPicks.indexOf(`immich:album:${a.id}`) >= 0}));
			} else {
				entries.push({key: 'immich:setup', title: 'Set up Immich', category: 'Immich', label: 'Set up'});
			}
			return {
				categories: ['Immich'],
				entries,
				summary: 'Pick which of your Immich photos the home screen shows.',
				onPick: (key: string) => {
					if (key === 'immich:setup') { showToast('Set up Immich first, in Settings > Immich'); return; }
					const has = config.immichPicks.indexOf(key) >= 0;
					if (has && config.immichPicks.length === 1) { showToast('Keep at least one source'); return; }
					updateConfig({...config, immichPicks: has ? config.immichPicks.filter((x) => x !== key) : config.immichPicks.concat(key)});
				}
			};
		}
		if (type === 'plain') return {categories: [], entries: [], onPick: () => undefined};
		const picked = config.wallpaperPicks;
		const keysIn: Record<string, string[]> = {Defaults: LOCAL_WALLPAPERS.map((_, i) => `local:${i}`)};
		WALLPAPERS.forEach((w) => { (keysIn[w.category] || (keysIn[w.category] = [])).push(`cloud:${w.id}`); });
		const categories = WALLPAPER_CATEGORIES.slice(0, 2).concat('Defaults', WALLPAPER_CATEGORIES.slice(2));      // Landscapes and Art first
		categories.forEach((c) => {
			const keys = keysIn[c] || [];
			const all = keys.length > 0 && keys.every((k) => picked.indexOf(k) >= 0);
			entries.push({key: `all:${c}`, title: all ? 'Deselect all' : 'Select all', category: c, label: 'All', on: all});
		});
		LOCAL_WALLPAPERS.forEach((w, i) => entries.push({key: `local:${i}`, title: w.title, category: 'Defaults', thumb: w.file, on: picked.indexOf(`local:${i}`) >= 0}));
		WALLPAPERS.forEach((w) => entries.push({key: `cloud:${w.id}`, title: w.title, category: w.category, thumb: resized(cloudUrl(w), 360), on: picked.indexOf(`cloud:${w.id}`) >= 0}));
		return {
			categories,
			entries,
			// with nothing picked, the default picture shows (lib/wallpaper.ts falls back to it); it is not one of the choices
			summary: picked.length === 0 ? 'Nothing picked: the default picture shows.' : `${picked.length} ${picked.length === 1 ? 'picture' : 'pictures'} picked.`,
			onPick: (key: string) => {
				if (key.indexOf('all:') === 0) {
					const keys = keysIn[key.slice(4)] || [];
					const all = keys.every((k) => picked.indexOf(k) >= 0);
					updateConfig({...config, wallpaperPicks: all ? picked.filter((k) => keys.indexOf(k) === -1) : picked.concat(keys.filter((k) => picked.indexOf(k) === -1))});
					return;
				}
				const has = picked.indexOf(key) >= 0;
				updateConfig({...config, wallpaperPicks: has ? picked.filter((k) => k !== key) : picked.concat(key)});
			}
		};
	}, [config, immich, inSettings]);

	// ----- actions -----
	const open = (id: string, label: string) => {
		launchApp(id).catch(() => showToast(`Couldn't open ${label}`, 'error'));
	};

	const watchChannel = useCallback((c: Channel) => {
		openChannel(c.id).catch(() => showToast(`Couldn't open ${c.name}`, 'error'));
	}, [showToast]);

	const onActivate = useCallback((_rowId: string, item: TileModel) => {
		if (item.key === LIVE.guide) { setScreen('guide'); return; }
		if (item.key.indexOf(LIVE.prefix) === 0) { openChannel(item.key.slice(LIVE.prefix.length)).catch(() => showToast(`Couldn't open ${item.label}`, 'error')); return; }
		if (item.key === MISC.settings || item.key === BUILTIN.settings) { setScreen('settings'); return; }
		if (item.key === 'screensaver') { startScreensaver().catch(() => showToast("Couldn't start the screensaver (it needs the Homebrew Channel)", 'error')); return; }
		if (item.key === MISC.lgHome) { launchLgHome().catch(() => showToast("Couldn't open LG Home", 'error')); return; }
		if (item.key === 'controls') { setControls(true); return; }
		if (_rowId === ROW.apps || _rowId.indexOf(ROW.dock) === 0 || _rowId === 'folder') {
			const c = configRef.current;
			updateConfig({...c, recentIds: [item.key].concat(c.recentIds.filter((x) => x !== item.key)).slice(0, RECENT_MAX)});
		}
		open(item.key, item.label);
	}, [showToast]);

	/** hold OK on an app: move it one place along the home rows, or take it off the home screen */
	const onEdit = useCallback((action: 'move' | 'remove' | 'uninstall' | 'folder-add' | 'folder-out', _rowId: string, key: string, arg: number | string = 0) => {
		const c = configRef.current;
		const strip = (folders: UserConfig['folders']) => folders.map((f) => ({...f, appIds: f.appIds.filter((id) => id !== key)}));
		if (action === 'uninstall') {
			const title = (allApps.filter((a) => a.id === key)[0] || {title: 'The app'}).title;
			showToast(`Uninstalling ${title}…`);
			uninstallApp(key).then((gone) => {
				if (!gone) { showToast(`${title} is still installed`, 'error'); return; }
				const now = configRef.current;
				updateConfig({...now, appIds: now.appIds.filter((id) => id !== key), folders: strip(now.folders).filter((f) => f.appIds.length > 0)});
				setApps((list) => list.filter((a) => a.id !== key));
				showToast(`${title} uninstalled`);
			}, () => showToast(`Couldn't uninstall ${title}`, 'error'));
			return;
		}
		if (action === 'folder-add') {
			const folders = strip(c.folders);
			const target = folders.filter((f) => f.name === String(arg))[0];
			if (target) target.appIds.push(key); else folders.push({name: String(arg), appIds: [key]});
			updateConfig({...c, folders: folders.filter((f) => f.appIds.length > 0)});
			return;
		}
		if (action === 'folder-out') { updateConfig({...c, folders: strip(c.folders).filter((f) => f.appIds.length > 0)}); return; }
		const inFolder = c.folders.reduce((all, f) => all.concat(f.appIds), [] as string[]);
		const ids = c.appIds.filter((id) => allApps.some((a) => a.id === id) && inFolder.indexOf(id) === -1);   // what the rows show, in order
		const kept = c.appIds.filter((id) => ids.indexOf(id) === -1);                                          // apps in folders keep their place
		const at = ids.indexOf(key);
		if (at < 0) return;
		if (action === 'remove') { updateConfig({...c, appIds: ids.filter((id) => id !== key).concat(kept)}); return; }
		// a row up or down past either end stops at the first or last place
		const to = Math.max(0, Math.min(ids.length - 1, at + Number(arg)));
		if (to === at) return;
		const next = ids.slice();
		next.splice(at, 1);
		next.splice(to, 0, key);
		updateConfig({...c, appIds: next.concat(kept)});
	}, [allApps]);

	const onToggle = useCallback((tabId: string, key: string) => {
		if (tabId === ROW.channels) {
			updateConfig({...config, liveChannels: toggleId(rowChannels(live.channels, config.liveChannels).map((c) => c.id), key)});
		} else if (tabId === ROW.apps) {
			updateConfig({...config, appIds: toggleId(config.appIds, key)});
		} else {
			const current = config.sourceIds || inputs.map((s) => s.appId);
			updateConfig({...config, sourceIds: toggleId(current, key)});
		}
	}, [config, inputs, live.channels]);

	/** Settings > Immich > Test connection: on a pass the address and key are saved on the TV and the Immich options open up */
	const testImmich = useCallback(() => {
		setImmichTest('testing');
		testServer(immichDraft).then((result) => {
			if (typeof result === 'string') { setImmichTest('failed'); showToast(result, 'error'); return; }
			return saveServer(result.server).then(() => {
				setImmichDraft(result.server);
				setImmich({ready: true, albums: result.albums});
				setImmichTest('');
				showToast('Connected to Immich');
			});
		}).catch(() => { setImmichTest('failed'); showToast('Couldn\'t save the Immich settings on the TV', 'error'); });
	}, [immichDraft, showToast]);

	/** text typed on a Settings text page */
	const onText = useCallback((optionId: string, value: string) => {
		if (optionId === OPTION.immichUrl) setImmichDraft((d) => ({...d, url: value.trim()}));
		else if (optionId === OPTION.immichKey) setImmichDraft((d) => ({...d, key: value.trim()}));
		setImmichTest('');
	}, []);

	const onOption = useCallback((optionId: string) => {
		if (onTvOption(optionId, tv, runTv, showToast)) return;
		// a side app's row installs or removes it; Update brings each one that is on up to the newest release
		const sideRow = SIDE_ROWS.filter((r) => r.option === optionId)[0];
		if (sideRow || optionId === OPTION.sideUpdate) {
			if (!side) { showToast('Still checking the side apps'); return; }
			const then = (line: string) => { readSide(); return line; };
			if (!sideRow) runTv(optionId, () => updateSides(side).then(then));
			else runTv(optionId, () => (side.installed[sideRow.id] ? removeSide(sideRow.id) : installSide(side, sideRow.id)).then(then));
			return;
		}
		if (optionId === OPTION.restore) {
			readBackup().then((saved) => {
				if (!saved) { showToast('No settings backup found on the TV', 'error'); return; }
				updateConfig({...saved, seeded: true});
				showToast('Settings restored from the TV');
			});
			return;
		}
		if (optionId === OPTION.immichTest) { testImmich(); return; }
		if (optionId === OPTION.liveTv) updateConfig({...config, liveTvRow: !config.liveTvRow});
		else if (optionId === OPTION.showAmPm) updateConfig({...config, showAmPm: !config.showAmPm});
		else if (optionId === OPTION.showAll) updateConfig({...config, showAllApps: !config.showAllApps});
		else if (optionId === OPTION.weather) updateConfig({...config, showWeather: !config.showWeather});
		else if (optionId === OPTION.fps) updateConfig({...config, fpsCounter: !config.fpsCounter});
		else if (optionId === OPTION.hideNames) updateConfig({...config, hideNames: !config.hideNames});
		else if (optionId === OPTION.showRecents) updateConfig({...config, showRecents: !config.showRecents});
		else if (optionId === OPTION.reduceMotion) updateConfig({...config, reduceMotion: !config.reduceMotion});
		else if (optionId === OPTION.dockBlur) updateConfig({...config, dockBlur: !config.dockBlur});
		else if (optionId === OPTION.liquidDock) updateConfig({...config, liquidDock: !config.liquidDock});
		else if (optionId === OPTION.dockDim) updateConfig({...config, dockDim: !config.dockDim});
		else if (optionId === OPTION.captions) updateConfig({...config, wallpaperCaptions: !config.wallpaperCaptions});
		else if (optionId === OPTION.ambientPrint) updateConfig({...config, ambientPrint: !config.ambientPrint});
		else if (optionId === OPTION.wpNew) updateConfig({...config, wpNewOnLaunch: !config.wpNewOnLaunch});
	}, [config, immich.ready, testImmich, tv, runTv, showToast, side, readSide]);

	/** a value picked from an option's pop-up list */
	const onChoose = useCallback((optionId: string, index: number) => {
		const c = configRef.current;
		const value = choices[optionId] && choices[optionId].values[index];
		if (value === undefined) return;
		const v = String(value);
		if (optionId === OPTION.country) {
			const first = COUNTRIES.filter((x) => x.code === v)[0];
			// a new country starts with no city (no weather until one is picked); the same country keeps its city
			updateConfig({...c, weatherCountry: v, weatherCity: v === c.weatherCountry ? c.weatherCity : first ? '' : c.weatherCity});
		} else if (optionId === OPTION.city) {
			const owner = COUNTRIES.filter((x) => x.cities.some((y) => y.name === v))[0];
			updateConfig({...c, weatherCity: v, weatherCountry: c.weatherCountry || (owner ? owner.code : '')});
		}
		else if (optionId === OPTION.immichDays) updateConfig({...c, immichRecentDays: Number(value)});
		else if (optionId === OPTION.wpAuto) updateConfig({...c, wpAutoMinutes: Number(value)});
		else if (optionId === OPTION.wpType) updateConfig({...c, wallpaperType: v});
		else if (optionId === OPTION.guidePreview) updateConfig({...c, guidePreview: v === 'on'});
		else if (optionId === OPTION.homePreview) updateConfig({...c, homePreview: v === 'on'});
		else if (optionId === OPTION.guideSound) updateConfig({...c, guideSound: v === 'on'});
		else if (optionId === OPTION.dockHide) updateConfig({...c, dockHide: v});
		else if (optionId === OPTION.ambientOn) {
			updateConfig({...c, ambientOn: v === 'on'});
		}
		else if (optionId === OPTION.saverShow) updateConfig({...c, saverShow: v});
		else if (optionId === OPTION.idle) updateConfig({...c, idleSeconds: Number(value)});
		else if (optionId === OPTION.darkMode) updateConfig({...c, darkMode: v});
		else if (optionId === OPTION.language) updateConfig({...c, language: v});
		else if (optionId === OPTION.clock) updateConfig({...c, clockFormat: v});
		else if (optionId === OPTION.focus) updateConfig({...c, focusStrength: v});
		else if (optionId === OPTION.homeCast) updateConfig(v === 'off' ? {...c, homeForecast: false} : {...c, homeForecast: true, homeForecastSize: v});
		else if (optionId === OPTION.saverCast) updateConfig(v === 'off' ? {...c, saverForecast: false} : {...c, saverForecast: true, saverForecastSize: v});
		else if (optionId === OPTION.firstRow) updateConfig({...c, firstRow: Number(value)});
		else if (optionId === OPTION.videoHours) updateConfig({...c, videoHours: v});
		else if (optionId === OPTION.nightDim) updateConfig({...c, wallpaperDim: v});
		else if (optionId === OPTION.ambient) {
			const picks = c.ambient.split('|');
			const has = picks.indexOf(v) >= 0;
			if (has && picks.length === 1) { showToast('Keep at least one source'); return; }
			updateConfig({...c, ambient: (has ? picks.filter((k) => k !== v) : picks.concat(v)).join('|')});
		}
		else if (optionId === OPTION.ambientSeconds) updateConfig({...c, ambientSeconds: Number(value)});
		else if (optionId === OPTION.ambientBrightness) updateConfig({...c, ambientBrightness: v});
		else if (optionId === OPTION.blOn) updateConfig({...c, brightnessOn: v === 'on'});
		else if (optionId === OPTION.blDayAt) updateConfig({...c, blDayAt: v});
		else if (optionId === OPTION.blEveningAt) updateConfig({...c, blEveningAt: v});
		else if (optionId === OPTION.blNightAt) updateConfig({...c, blNightAt: v});
		else if (optionId === OPTION.blDay) updateConfig({...c, blDay: Number(value)});
		else if (optionId === OPTION.blEvening) updateConfig({...c, blEvening: Number(value)});
		else if (optionId === OPTION.blNight) updateConfig({...c, blNight: Number(value)});
		else if (optionId === OPTION.blRamp) updateConfig({...c, blRamp: Number(value)});
		else if (optionId === OPTION.blEye) updateConfig({...c, blEye: v});
	}, [choices]);

	/** Replace a row's order with `keys` (the full list of what's shown). */
	const onReorder = useCallback((tabId: string, keys: string[]) => {
		if (tabId === ROW.apps) updateConfig({...config, appIds: keys});
		else if (tabId === ROW.channels) updateConfig({...config, liveChannels: keys});
		else updateConfig({...config, sourceIds: keys});
	}, [config]);

	return (
		<>
			{/* Home stays mounted under Settings so it keeps its focus position. */}
			<Home rows={rows} tiles={TILES_FOR[firstRow] || TILES_FOR[6]} perRow={firstRow} topItems={TOP_ITEMS} inputs={shownInputs.map(inputTile)} active={screen === 'home' && !controls} onActivate={onActivate} showPeriod={config.showAmPm} onEdit={onEdit} folders={folderMap} weather={weather} weatherLine={config.showWeather && !(config.homeForecast && config.homeForecastSize === 'large')} forecast={config.homeForecast ? config.homeForecastSize as 'small' | 'large' : undefined} dockHide={config.dockHide} dockDim={config.dockDim} captions={config.wallpaperCaptions} focusRow={focusRow} />
			{screen === 'ambient' && <Ambient show={config.saverShow} source={config.ambient} pictures={config.wallpaperPicks} seconds={config.ambientSeconds} backlight={AMBIENT_BACKLIGHT[config.ambientBrightness]} weather={weather} weatherLine={config.showWeather && !(config.saverForecast && config.saverForecastSize === 'large')} forecast={config.saverForecast ? config.saverForecastSize as 'small' | 'large' : undefined} showPeriod={config.showAmPm} print={config.ambientPrint} opening={ambientOpening.current} onExit={leaveAmbient} />}
			{screen === 'guide' && <Guide key={guideOpens} channels={live.channels} listings={live.listings} logos={live.logos} now={live.now} schedule={live.schedule} pending={live.pending} full={config.guideFull} preview={__LUMEN_DEV__ && config.guidePreview} sound={config.guideSound} onMode={(full) => { if (configRef.current.guideFull !== full) updateConfig({...configRef.current, guideFull: full}); }} active={!controls} onOpen={watchChannel} onClose={() => setScreen('home')} />}
			{screen === 'settings' && <Settings tabs={tabs} wallpaper={wallpaper} options={options} active onToggle={onToggle} onReorder={onReorder} onOption={onOption} onChoose={onChoose} onText={onText} onClose={() => setScreen('home')} />}
			{controls && <ControlCenter
				darkMode={config.darkMode}
				clockFormat={config.clockFormat}
				onDarkMode={(darkMode) => updateConfig({...configRef.current, darkMode})}
				onClockFormat={(clockFormat) => updateConfig({...configRef.current, clockFormat})}
				onClose={() => setControls(false)}
				onCloseDown={() => { setControls(false); setFocusRow({id: ROW.apps, seq: Date.now()}); }}
				onError={(text) => showToast(text, 'error')}
				onAction={(a: CenterAction) => { setControls(false); if (a === 'art') showScreensaver(true); else onActivate('menu', {key: MISC.lgHome, label: 'LG Home'}); }}
			/>}
			<Toast message={toast} />
		</>
	);
}
