// The user's choices: which apps and which inputs appear on the home rows, in order,
// plus options. Persisted in localStorage (private to this app on the TV).

import {STORAGE_KEY} from '../config/constants';
import {DEFAULT_APP_IDS} from '../config/catalog';
import {WALLPAPERS} from '../config/wallpapers';

/** Settings > Screensaver > Start on the home screen after: the choices, in seconds (0 = never). */
export const IDLE_SECONDS = [0, 30, 60, 120, 300];

export interface UserConfig {
	/** ordered app IDs for the Apps row */
	appIds: string[];
	/** ordered input appIds (com.webos.app.hdmiN) for the Sources row; null → all inputs */
	sourceIds: string[] | null;
	/** read the live app list through the Homebrew Channel (root). Off by default. */
	useHomebrew: boolean;
	/** play a video behind the home screen (aerials and stock clips, streamed from the internet). Off by default. */
	videoWallpaper: boolean;
	/** what the still wallpaper shows, worked out from the fields below by withWallpaper (index.html reads it): pick | immich:<a>|<b> | none */
	wallpaper: string;
	/** Settings > Wallpaper > Wallpaper type: pictures | video | immich | plain */
	wallpaperType: string;
	/** the pictures the home screen picks from (cloud:<id>, local:N), chosen one by one or a category at a time */
	wallpaperPicks: string[];
	/** the Immich sources the home screen picks from (immich:recent, immich:album:<id>, ...) */
	immichPicks: string[];
	/** a new picture from the picks each time Lumen opens; off, it opens on the last one */
	wpNewOnLaunch: boolean;
	/** show AM/PM beside the time when the TV uses a 12-hour clock */
	showAmPm: boolean;
	/** menu language: 'auto' follows the TV, else a code from lib/i18n.ts */
	language: string;
	/** 'auto' follows the TV's 12/24-hour setting, else '12h' or '24h' */
	clockFormat: string;
	/** place the video wallpaper plays ('' = shuffle all) (the older single choice, now videoPlaces) */
	videoClip: string;
	/** the places the video wallpaper shuffles through; empty = every place */
	videoPlaces: string[];
	/** when the dock slides away: off | down | idle | both */
	dockHide: string;
	/** dark glass and a dimmer wallpaper */
	darkMode: string;
	/** the last apps opened, newest first */
	recentIds: string[];
	/** start the screensaver after this many idle minutes; 0 = never */
	/** seconds on the home screen without a key press before the screensaver starts (0 = never) */
	idleSeconds: number;
	/** cloud wallpaper ids marked as favourites (hold OK in the picker) */
	favoriteWallpapers: string[];
	/** pick a new wallpaper every N minutes when a shuffle is chosen; 0 = never */
	wpAutoMinutes: number;
	/** dim the wallpaper: off | night (19:00 to 06:00) | always */
	wallpaperDim: string;
	/** darken the screen while the dock is hidden */
	dockDim: boolean;
	/** the wallpaper's title and credit, or the video's place, at the bottom right while the dock is hidden */
	wallpaperCaptions: boolean;
	/** offer the TV's own apps (Live TV, AirPlay ...) in Settings */
	showAllApps: boolean;
	/** date and weather beside the clock */
	showWeather: boolean;
	/** the weather widget on the home screen, floating above the dock on the left (components/Forecast.tsx) */
	homeForecast: boolean;
	/** its size: small (the next hours) | large (now, the next hours and the chance of rain) */
	homeForecastSize: string;
	/** the weather widget on the screensaver, between the clock and the date */
	saverForecast: boolean;
	saverForecastSize: string;
	/** weather country (a code from config/cities.ts), '' = not chosen */
	weatherCountry: string;
	/** weather place: '' = from the internet connection, else a name from config/cities.ts */
	weatherCity: string;
	/** app folders: apps listed here are shown inside the folder, not on the rows */
	folders: {name: string; appIds: string[]}[];
	/** no transitions or animations */
	reduceMotion: boolean;
	/** no app names under the tiles on the home screen */
	hideNames: boolean;
	/** the dot over apps that are open */
	showRecents: boolean;
	/** the glass bar blurs what is behind it (backdrop-filter); off, it keeps its glass look with no blur, and left and right run smoother */
	dockBlur: boolean;
	/** the glass bar bends the picture at its rim, like Liquid Glass (lib/liquidGlass.ts) */
	liquidDock: boolean;
	/** the Immich sources the ambient screen shows, joined with | (immich:recent|immich:album:<id>, ...) */
	ambient: string;
	/** Lumen's screensaver replaces LG's (Screensaver in Control Center shows Lumen's either way) */
	ambientOn: boolean;
	/** what Lumen's screensaver shows: wallpaper (the pictures picked in Choose wallpaper) | immich (the ambient sources) */
	saverShow: string;
	/** seconds each photo stays on the ambient screen */
	ambientSeconds: number;
	/** how far the screensaver lowers the TV's backlight while it shows: tv (not at all) | dim | low */
	ambientBrightness: string;
	/** Settings > Brightness: tools/tv/watch.sh sets the TV's backlight (and Eye Comfort, unless blEye is keep) at three times of day */
	brightnessOn: boolean;
	/** when each part of the day starts ("HH:MM") and its backlight (0 to 100) */
	blDayAt: string;
	blDay: number;
	blEveningAt: string;
	blEvening: number;
	blNightAt: string;
	blNight: number;
	/** minutes each part of the day takes to move from the last one's backlight to its own; 0 changes it at once */
	blRamp: number;
	/** Eye Comfort through the day: keep (as the TV is set) | off | evening (evening and night) | night */
	blEye: string;
	/** screensaver photos look like a print on the wall: lifted blacks, paper whites, light falling off from the top left */
	ambientPrint: boolean;
	/** how far back the Immich Recent sources reach, in days: 30 | 90 | 180 | 365 */
	immichRecentDays: number;
	/** how strongly the focused tile stands out: subtle | normal | strong */
	focusStrength: string;
	/** apps in the first row (the glass bar): 5, 6 or 7 */
	firstRow: number;
	/** when the video wallpaper plays: always | day (7:00-19:00) | evening (17:00-23:00) | night (19:00-7:00); the still wallpaper otherwise */
	videoHours: string;
	/** dev builds: show the frame-rate counter (lib/fps.ts) */
	fpsCounter: boolean;
	/** dev builds: a live picture of the channel in focus in the Guide (components/LivePreview.tsx) */
	guidePreview: boolean;
	/** dev builds: the focused channel tile on Home's Live TV row plays a live preview */
	homePreview: boolean;
	/** dev builds: the live preview plays its sound */
	guideSound: boolean;
	/** false on a fresh install until the installed apps have been put on the home screen once */
	seeded: boolean;
	/** the built-in tiles (Settings, TV Settings, Inputs) have been put on the home screen once; taken off, they stay off */
	builtinTiles: boolean;
	/** a row of aerial channels under the apps, with what is on now (needs the Homebrew Channel) */
	liveTvRow: boolean;
	/** the Live TV row's channels (channel ids), in order, picked in Settings > Channels; null = the first five by number */
	liveChannels: string[] | null;
	/** the Guide opens on the Full guide (the time grid) rather than Now and Next; the switcher at its top right sets it */
	guideFull: boolean;
}

export const DEFAULT_CONFIG: UserConfig = {
	appIds: DEFAULT_APP_IDS.slice(),
	sourceIds: null,
	useHomebrew: true,
	videoWallpaper: false,
	wallpaper: 'pick',
	wallpaperType: 'pictures',
	wallpaperPicks: [],
	immichPicks: ['immich:recent'],
	wpNewOnLaunch: true,
	showAmPm: true,
	language: 'auto',
	darkMode: 'on',
	dockHide: 'down',
	videoClip: '',
	videoPlaces: [],
	clockFormat: 'auto',
	recentIds: [],
	idleSeconds: 0,
	favoriteWallpapers: [],
	wpAutoMinutes: 0,
	wallpaperDim: 'off',
	dockDim: false,
	wallpaperCaptions: true,
	showAllApps: false,
	showWeather: true,
	homeForecast: false,
	homeForecastSize: 'small',
	saverForecast: false,
	saverForecastSize: 'small',
	weatherCountry: '',
	weatherCity: '',
	folders: [],
	reduceMotion: false,
	hideNames: false,
	showRecents: true,
	dockBlur: false,
	liquidDock: false,
	ambient: 'immich:recent',
	ambientOn: false,
	saverShow: 'wallpaper',
	ambientSeconds: 30,
	ambientBrightness: 'dim',
	ambientPrint: false,
	brightnessOn: false,
	blDayAt: '07:00',
	blDay: 80,
	blEveningAt: '18:00',
	blEvening: 60,
	blNightAt: '22:00',
	blNight: 30,
	blRamp: 0,
	blEye: 'keep',
	immichRecentDays: 180,
	focusStrength: 'normal',
	firstRow: 6,
	videoHours: 'always',
	fpsCounter: false,
	guidePreview: false,
	homePreview: false,
	guideSound: false,
	seeded: false,
	builtinTiles: false,
	liveTvRow: true,
	liveChannels: null,
	guideFull: false
};

export function loadConfig (): UserConfig {
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		return raw ? parseConfig(raw) : freshConfig();
	} catch {
		return freshConfig();
	}
}

function freshConfig (): UserConfig { return {...DEFAULT_CONFIG, wallpaperPicks: starterPicks()}; }

/** Whether this install has settings of its own yet (false on a fresh install, before anything has been saved). */
export function hasSavedConfig (): boolean {
	try { return localStorage.getItem(STORAGE_KEY) !== null; } catch { return false; }
}

/** A config from JSON, with every field checked and anything missing or wrong replaced by its default. Throws on malformed JSON. */
export function parseConfig (raw: string): UserConfig {
	// nightDim is the older on/off form of wallpaperDim
	const parsed = JSON.parse(raw) as Partial<UserConfig> & {nightDim?: boolean};
	return withWallpaper({
		appIds: Array.isArray(parsed.appIds) ? parsed.appIds : DEFAULT_CONFIG.appIds,
		sourceIds: Array.isArray(parsed.sourceIds) ? parsed.sourceIds : null,
		useHomebrew: true,                  // the switch is gone: Lumen is installed through the Homebrew Channel
		videoWallpaper: parsed.videoWallpaper === true,
		wallpaper: typeof parsed.wallpaper === 'string' ? parsed.wallpaper : DEFAULT_CONFIG.wallpaper,
		...wallpaperFields(parsed),
		showAmPm: parsed.showAmPm !== false,
		dockHide: parsed.dockHide === 'off' || parsed.dockHide === 'idle' || parsed.dockHide === 'both' ? parsed.dockHide : 'down',
		darkMode: parsed.darkMode === 'auto' ? 'auto' : parsed.darkMode === 'off' ? 'off' : 'on',
		videoClip: typeof parsed.videoClip === 'string' ? parsed.videoClip : '',
		videoPlaces: Array.isArray(parsed.videoPlaces) ? parsed.videoPlaces : typeof parsed.videoClip === 'string' && parsed.videoClip ? [parsed.videoClip] : [],
		language: typeof parsed.language === 'string' ? parsed.language : 'auto',
		clockFormat: parsed.clockFormat === '12h' || parsed.clockFormat === '24h' ? parsed.clockFormat : 'auto',
		recentIds: Array.isArray(parsed.recentIds) ? parsed.recentIds : [],
		idleSeconds: IDLE_SECONDS.indexOf(Number(parsed.idleSeconds)) !== -1 ? Number(parsed.idleSeconds) : 0,
		favoriteWallpapers: Array.isArray(parsed.favoriteWallpapers) ? parsed.favoriteWallpapers : [],
		wpAutoMinutes: typeof parsed.wpAutoMinutes === 'number' ? parsed.wpAutoMinutes : 0,
		wallpaperDim: parsed.wallpaperDim === 'night' || parsed.wallpaperDim === 'always' ? parsed.wallpaperDim : parsed.nightDim === true ? 'night' : 'off',
		dockDim: parsed.dockDim === true,
		wallpaperCaptions: parsed.wallpaperCaptions !== false,
		showAllApps: parsed.showAllApps === true,
		showWeather: parsed.showWeather !== false,
		homeForecast: parsed.homeForecast === true,
		homeForecastSize: parsed.homeForecastSize === 'large' ? 'large' : 'small',
		saverForecast: parsed.saverForecast === true,
		saverForecastSize: parsed.saverForecastSize === 'large' ? 'large' : 'small',
		weatherCountry: typeof parsed.weatherCountry === 'string' ? parsed.weatherCountry : '',
		weatherCity: typeof parsed.weatherCity === 'string' ? parsed.weatherCity : '',
		folders: Array.isArray(parsed.folders) ? parsed.folders : [],
		reduceMotion: parsed.reduceMotion === true,
		hideNames: parsed.hideNames === true,
		showRecents: parsed.showRecents !== false,
		dockBlur: parsed.dockBlur === true,
		liquidDock: __LUMEN_DEV__ && parsed.liquidDock === true,      // too slow for this TV, so only dev builds offer it
		ambientSeconds: [10, 20, 30, 60, 120, 300].indexOf(Number(parsed.ambientSeconds)) !== -1 ? Number(parsed.ambientSeconds) : 30,
		ambientBrightness: parsed.ambientBrightness === 'tv' || parsed.ambientBrightness === 'low' ? parsed.ambientBrightness : 'dim',
		ambientPrint: parsed.ambientPrint === true,
		brightnessOn: parsed.brightnessOn === true,
		blDayAt: hhmm(parsed.blDayAt, DEFAULT_CONFIG.blDayAt),
		blDay: level(parsed.blDay, DEFAULT_CONFIG.blDay),
		blEveningAt: hhmm(parsed.blEveningAt, DEFAULT_CONFIG.blEveningAt),
		blEvening: level(parsed.blEvening, DEFAULT_CONFIG.blEvening),
		blNightAt: hhmm(parsed.blNightAt, DEFAULT_CONFIG.blNightAt),
		blNight: level(parsed.blNight, DEFAULT_CONFIG.blNight),
		blRamp: [0, 30, 60, 90, 120].indexOf(Number(parsed.blRamp)) !== -1 ? Number(parsed.blRamp) : 0,
		blEye: parsed.blEye === 'off' || parsed.blEye === 'evening' || parsed.blEye === 'night' ? parsed.blEye : 'keep',
		immichRecentDays: [30, 90, 180, 365].indexOf(Number(parsed.immichRecentDays)) !== -1 ? Number(parsed.immichRecentDays) : 180,
		ambient: typeof parsed.ambient === 'string' && parsed.ambient.indexOf('immich:') === 0 ? parsed.ambient : 'immich:recent',
		// before ambientOn, an empty source meant LG's screensaver
		ambientOn: typeof parsed.ambientOn === 'boolean' ? parsed.ambientOn : typeof parsed.ambient === 'string' && parsed.ambient.indexOf('immich:') === 0,
		// before saverShow, Lumen's screensaver could only show Immich
		saverShow: parsed.saverShow === 'wallpaper' || parsed.saverShow === 'immich' ? parsed.saverShow : parsed.ambientOn === true ? 'immich' : 'wallpaper',
		focusStrength: parsed.focusStrength === 'subtle' || parsed.focusStrength === 'strong' ? parsed.focusStrength : 'normal',
		firstRow: parsed.firstRow === 5 || parsed.firstRow === 7 ? parsed.firstRow : 6,
		videoHours: parsed.videoHours === 'day' || parsed.videoHours === 'evening' || parsed.videoHours === 'night' ? parsed.videoHours : 'always',
		fpsCounter: parsed.fpsCounter === true,
		guidePreview: __LUMEN_DEV__ && parsed.guidePreview === true,
		homePreview: __LUMEN_DEV__ && parsed.homePreview === true,
		guideSound: __LUMEN_DEV__ && parsed.guideSound === true,
		seeded: parsed.seeded !== false,
		builtinTiles: parsed.builtinTiles === true,
		liveTvRow: parsed.liveTvRow !== false,
		liveChannels: Array.isArray(parsed.liveChannels) ? parsed.liveChannels.filter((x) => typeof x === 'string') : null,
		guideFull: parsed.guideFull === true
	});
}

const hhmm = (v: unknown, fallback: string) => typeof v === 'string' && /^([01]\d|2[0-3]):[0-5]\d$/.test(v) ? v : fallback;
const level = (v: unknown, fallback: number) => typeof v === 'number' && Number.isInteger(v) && v >= 0 && v <= 100 ? v : fallback;

/** Landscapes and Art: what the pictures start from on a new install */
export function starterPicks (): string[] {
	return WALLPAPERS.filter((w) => w.category === 'Landscapes' || w.category === 'Art').map((w) => `cloud:${w.id}`);
}

/** The wallpaper type and picks, from a config that has them or, once, from the older single choice it replaces. */
function wallpaperFields (p: Partial<UserConfig>): Pick<UserConfig, 'wallpaperType' | 'wallpaperPicks' | 'immichPicks' | 'wpNewOnLaunch'> {
	const strings = (v: unknown) => Array.isArray(v) ? (v as unknown[]).filter((x): x is string => typeof x === 'string') : null;
	const type = p.wallpaperType === 'video' || p.wallpaperType === 'immich' || p.wallpaperType === 'plain' || p.wallpaperType === 'pictures' ? p.wallpaperType : '';
	if (type) return {wallpaperType: type, wallpaperPicks: strings(p.wallpaperPicks) || starterPicks(), immichPicks: strings(p.immichPicks) || ['immich:recent'], wpNewOnLaunch: p.wpNewOnLaunch !== false};
	const old = typeof p.wallpaper === 'string' ? p.wallpaper : '';
	const favs = strings(p.favoriteWallpapers) || [];
	const picks = old === 'shuffle:Favorites' && favs.length > 0 ? favs.map((id) => `cloud:${id}`)
		: old.indexOf('shuffle:') === 0 ? WALLPAPERS.filter((w) => w.category === old.slice(8)).map((w) => `cloud:${w.id}`)
		: old.indexOf('cloud:') === 0 || old.indexOf('local:') === 0 ? [old]
		: starterPicks();
	return {
		wallpaperType: p.videoWallpaper === true ? 'video' : old.indexOf('immich:') === 0 ? 'immich' : old === 'none' ? 'plain' : 'pictures',
		wallpaperPicks: picks.length > 0 ? picks : starterPicks(),
		immichPicks: old.indexOf('immich:') === 0 ? [old] : ['immich:recent'],
		// a fixed picture stays put; a shuffle was a new one each launch
		wpNewOnLaunch: old.indexOf('cloud:') !== 0 && old.indexOf('local:') !== 0
	};
}

/** The config with `wallpaper` and `videoWallpaper` worked out from the wallpaper type and picks. Video keeps the pictures as its still
 *  wallpaper, shown outside the video hours. */
export function withWallpaper (c: UserConfig): UserConfig {
	const wallpaper = c.wallpaperType === 'plain' ? 'none'
		: c.wallpaperType === 'immich' ? (c.immichPicks.length > 0 ? c.immichPicks.join('|') : 'immich:recent')
		: 'pick';
	return wallpaper === c.wallpaper && c.videoWallpaper === (c.wallpaperType === 'video') ? c : {...c, wallpaper, videoWallpaper: c.wallpaperType === 'video'};
}

export function saveConfig (config: UserConfig): void {
	try { localStorage.setItem(STORAGE_KEY, JSON.stringify(config)); }
	catch { /* nothing sensible to do on a TV */ }
}

/** Toggle an ID in an ordered list: remove if present, else append. */
export function toggleId (ids: string[], id: string): string[] {
	return ids.indexOf(id) === -1 ? ids.concat(id) : ids.filter((x) => x !== id);
}
