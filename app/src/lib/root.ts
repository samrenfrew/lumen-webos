// Every command Lumen runs as root, through the Homebrew Channel's exec service. Nothing else in the app builds a shell command: each export
// below is one fixed command, and any value that goes into one is checked against a list or a pattern first, then shell-quoted.

import {LUNA} from '../config/constants';
import {isWebOS, lunaCall} from './luna';

/** The TV-side folder shared with tools/tv/watch.sh (flag files, the log, the settings backup). */
export const HOMEHOOK = '/var/lib/homehook';

/** Quote a value for a POSIX shell. */
export function shellQuote (value: string): string {
	return `'${value.replace(/'/g, `'\\''`)}'`;
}

/** Plain absolute paths only: quotes, spaces, control characters and shell syntax are refused rather than passed on. */
const SAFE_PATH = /^\/[A-Za-z0-9._\/+@-]+$/;
const BASE64 = /^[A-Za-z0-9+\/]*={0,2}$/;

/** The settings Lumen reads or changes, and the values it may set. A key not listed here never reaches the shell. */
const SETTINGS: Record<string, {category: string; values?: string[]}> = {
	pictureMode: {category: 'picture', values: ['vivid', 'normal', 'eco', 'cinema', 'sports', 'game', 'filmMaker']},
	soundMode: {category: 'sound', values: ['standard', 'news', 'movie', 'music', 'sports', 'game', 'aiSound']},
	/** for the picture mode in use (the TV keeps one per mode and input) */
	backlight: {category: 'picture', values: Array.from({length: 101}, (_, i) => String(i))},
	eyeComfortMode: {category: 'picture', values: ['on', 'off']}
};

/** Files under HOMEHOOK that Lumen may write or read. */
const FILES = ['lumen-config.json', 'recent', 'immich.json'] as const;
export type HomehookFile = typeof FILES[number];
const LOG = `${HOMEHOOK}/lumen.log`;
const LOG_MAX_BYTES = 64 * 1024;

let timed: ((tag: string, start: number, end: number) => void) | null = null;
/** lib/log.ts's boot timeline: told when each command started and ended. */
export function setRootTimer (fn: (tag: string, start: number, end: number) => void): void { timed = fn; }

/** Set once the Homebrew Channel's service turns out to be missing (a TV in Developer Mode, not rooted): every later command fails at once
 *  instead of going over the bus, so the start-up burst and the Live TV row's minute-by-minute refresh cost nothing. Reset by a relaunch. */
let noService = false;
/** the bus's answer when the Homebrew Channel itself is missing; a command's own error naming another service must not match */
const NO_SERVICE = /Service does not exist: org\.webosbrew\.hbchannel\.service\b/;

/** The one place a command is sent. Not exported: callers use the fixed commands below. */
function run (command: string, timeoutMs = 6000): Promise<string> {
	if (!isWebOS()) return Promise.reject(new Error('not on a TV'));
	if (noService) return Promise.reject(new Error('Homebrew Channel service not installed'));
	const start = performance.now();
	const call = lunaCall<{stdoutString?: string}>(LUNA.homebrewExec, {command}, timeoutMs)
		.then((r) => r.stdoutString || '', (e: Error) => { if (NO_SERVICE.test(e && e.message)) noService = true; throw e; });
	if (timed) {
		// a Luna method's name ("getChannelList"), else the command's first word ("base64", "curl")
		const m = /luna:\/\/[\w.]+\/(?:[\w]+\/)*(\w+)/.exec(command);
		const tag = m ? m[1] : (/(curl|base64|touch|rm|mkdir)/.exec(command) || ['shell'])[0];
		const done = () => { if (timed) timed(tag, start, performance.now()); };
		call.then(done, done);
	}
	return call;
}

function luna (uri: string, params: object): string {
	return `luna-send -n 1 ${shellQuote(uri)} ${shellQuote(JSON.stringify(params))}`;
}

export function listLaunchPoints (): Promise<string> {
	return run(luna('luna://com.webos.applicationManager/listLaunchPoints', {}), 8000);
}

const APP_ID = /^[A-Za-z0-9._-]+$/;

/** The ids of the apps the TV has open, Lumen too. Root because the app manager's running list is not offered to apps. */
export function runningApps (): Promise<string[]> {
	return run(luna('luna://com.webos.applicationManager/running', {})).then((text) => {
		const ids = (JSON.parse(text).running || []) as {id?: string}[];
		return ids.map((a) => a.id || '').filter((id) => APP_ID.test(id));
	});
}

/** Never offered for uninstalling: Lumen itself, LG Home and system apps, and the Homebrew Channel, which every root command runs through. */
const KEEP_INSTALLED = /^(com\.samrenfrew\.lumen|com\.ammar027\.lumen|com\.webos\.|com\.palm\.|com\.lge\.|org\.webosbrew\.hbchannel$)/;

export function canUninstall (id: string): boolean {
	return APP_ID.test(id) && !KEEP_INSTALLED.test(id);
}

/** Removes an app: the developer-mode remover first (Homebrew Channel apps), then the store's. True once the TV no longer knows the app. */
export function uninstallApp (id: string): Promise<boolean> {
	if (!canUninstall(id)) return Promise.reject(new Error(`refused: uninstall ${id}`));
	const present = `${luna('luna://com.webos.applicationManager/getAppInfo', {id})} 2>/dev/null | grep -q '"returnValue":true'`;
	const remove = (service: string) => `if ${present}; then ${luna(`luna://com.webos.appInstallService/${service}`, {id, subscribe: false})} >/dev/null 2>&1; sleep 2; fi`;
	return run(`${remove('dev/remove')}; ${remove('remove')}; if ${present}; then echo kept; else echo gone; fi`, 30000).then((text) => text.indexOf('gone') !== -1);
}

/** Icons as "<index> <base64>" lines, one per path; paths that are not plain absolute paths are skipped (their line is missing). */
export function readIcons (paths: string[]): Promise<string> {
	const script = paths
		.map((p, i) => SAFE_PATH.test(p) ? `printf '%d ' ${i}; base64 ${shellQuote(p)} 2>/dev/null | tr -d '\\n'; echo` : '')
		.filter(Boolean)
		.join('; ');
	return script ? run(script, 15000) : Promise.resolve('');
}

// ---------- Live TV (docs/live-tv-build.md) ----------

/** "1_41_2_102_16522_17472_9018": seven numbers. */
const CHANNEL_ID = /^\d+(_\d+){6}$/;
/** db8's paging key from the last reply ("1mo403xJBZ3rD34pMoS5+F") */
const DB8_PAGE = /^[A-Za-z0-9+\/=_.-]{1,128}$/;
/** the now and next programmes of every channel */
const PROGRAM_PF = 'com.webos.service.legacybroadcast.programPF:4';
const PF_FIELDS = ['signalChannelId', 'programName', 'startTime', 'endTime', 'isPresent'];

/** The aerial channel list (about 188 channels), as the TV's JSON reply. */
export function channelList (): Promise<string> {
	return run(luna('luna://com.webos.service.iepg/getChannelList', {dataType: 0, sort: 0, channelMode: 'Terrestrial', attribute: [{filterType: 'Invisible', value: false}]}), 12000);
}

/** One page (up to 500 rows) of the now and next programmes, as the TV's JSON reply. `slim` asks only for the fields Lumen reads; without
 *  it every row carries its audio, rating and description blocks (about 1.5 KB a row). */
export function nowNext (page?: string, slim = true): Promise<string> {
	if (page !== undefined && !DB8_PAGE.test(page)) return Promise.reject(new Error('refused: db8 page'));
	const query: Record<string, unknown> = {from: PROGRAM_PF, limit: 500};
	if (slim) query.select = PF_FIELDS;
	if (page) query.page = page;
	return run(luna('luna://com.webos.epgdb/find', {query}), 12000);
}

/** the full schedule (the Full guide); about 11,900 rows when Live TV has run lately */
const PROGRAM_SCH = 'com.webos.service.legacybroadcast.programSCH:4';
const SCH_FIELDS = ['signalChannelId', 'programName', 'startTime', 'endTime', 'isSubtitle', 'isAD'];

/** One page (up to 500 rows) of the whole schedule, as the TV's JSON reply. No time filter: the kind has no index on the times. */
export function schedule (page?: string): Promise<string> {
	if (page !== undefined && !DB8_PAGE.test(page)) return Promise.reject(new Error('refused: db8 page'));
	const query: Record<string, unknown> = {from: PROGRAM_SCH, select: SCH_FIELDS, limit: 500};
	if (page) query.page = page;
	return run(luna('luna://com.webos.epgdb/find', {query}), 12000);
}

/** "2026,10,18,06,30,00": the guide service's local-time stamp */
const LOCAL_TIME = /^\d{4}(,\d{2}){5}$/;
/** channels per guide service call, as LG's own Guide asks */
export const EVENT_BATCH = 10;

/** Every programme on up to EVENT_BATCH channels between two local times, from the TV's guide service (what LG's Guide reads, which can
 *  also draw on the online Freeview Play guide), as the TV's JSON reply. The call is copied from LG's Guide (docs/live-tv.md Step 2d). */
export function eventList (channelIds: string[], localStart: string, localEnd: string): Promise<string> {
	if (channelIds.length === 0 || channelIds.length > EVENT_BATCH || !channelIds.every((id) => CHANNEL_ID.test(id))) return Promise.reject(new Error('refused: channel ids'));
	if (!LOCAL_TIME.test(localStart) || !LOCAL_TIME.test(localEnd)) return Promise.reject(new Error('refused: guide time'));
	return run(luna('luna://com.webos.service.iepg/getEventList', {bFVPChannelId: true, channelId: channelIds, dataType: 0, fepgType: 'Forward', localStartTime: localStart, localEndTime: localEnd, sort: 0}), 12000);
}

/** Opens Live TV on a channel (from Home too). */
export function openChannel (channelId: string): Promise<void> {
	if (!CHANNEL_ID.test(channelId)) return Promise.reject(new Error('refused: channel id'));
	return run(luna('luna://com.webos.service.apiadapter/tv/openChannel', {channelId})).then((text) => {
		if (text.indexOf('"returnValue":false') !== -1) throw new Error('the TV refused the channel');
	});
}

/** The value of one known setting, or '' when the TV has none. */
export function getSetting (key: string): Promise<string> {
	const s = SETTINGS[key];
	if (!s) return Promise.reject(new Error('unknown setting ' + key));
	return run(luna('luna://com.webos.settingsservice/getSystemSettings', {category: s.category, keys: [key]})).then((text) => {
		const m = JSON.parse(text) as {settings?: Record<string, unknown>};
		const v = m.settings && m.settings[key];
		return v === undefined || v === null ? '' : String(v);
	});
}

/** Rejects for a key or value not in SETTINGS, and when the TV answers returnValue:false. */
export function setSetting (key: string, value: string): Promise<void> {
	const s = SETTINGS[key];
	if (!s || (s.values && s.values.indexOf(value) === -1)) return Promise.reject(new Error(`refused: ${key}=${value}`));
	return run(luna('luna://com.webos.settingsservice/setSystemSettings', {category: s.category, settings: {[key]: value}})).then((text) => {
		if (text.indexOf('"returnValue":false') !== -1) throw new Error(`the TV refused ${key}=${value}`);
	});
}

export function screenOff (): Promise<void> {
	return run(luna('luna://com.webos.service.tvpower/power/turnOffScreen', {})).then(() => undefined);
}

// every tvpower reboot reason comes back in Active Standby (docs/spikes.md); the shell's reboot comes back on. Detached so the call returns.
export function reboot (): Promise<void> {
	return run('(sleep 1; reboot) >/dev/null 2>&1 &').then(() => undefined);
}

/** Seconds left on the sleep timer, 0 when none. webOS 25 denies tvReservationAgent to apps ("Denied method call"), hence root. */
export function sleepRemainSeconds (): Promise<number> {
	return run(luna('luna://com.webos.service.tvReservationAgent/timer/sleep/getRemainTime', {})).then((text) => {
		const m = JSON.parse(text) as {sleepTimeSecond?: number};
		return Number(m.sleepTimeSecond) || 0;
	});
}

/** tvpower/turnOnScreenSaver is a system-only call, hence root. */
export function startScreensaver (): Promise<void> {
	return run(luna('luna://com.webos.service.tvpower/power/turnOnScreenSaver', {})).then(() => undefined);
}

/** Tells watch.sh the video wallpaper is on, so LG's screensaver is ended while Lumen is in front. */
export function setVideoFlag (on: boolean): Promise<void> {
	return run(on ? `mkdir -p ${HOMEHOOK} && touch ${HOMEHOOK}/video-wallpaper` : `rm -f ${HOMEHOOK}/video-wallpaper`).then(() => undefined);
}

/** Tells watch.sh to replace LG's screensaver with Lumen's ambient screen (your photos). */
export function setAmbientFlag (on: boolean): Promise<void> {
	return run(on ? `mkdir -p ${HOMEHOOK} && touch ${HOMEHOOK}/ambient-screensaver` : `rm -f ${HOMEHOOK}/ambient-screensaver`).then(() => undefined);
}

/** Tells watch.sh to end LG's screensaver as soon as it starts, in every app (Settings > Screensaver > Off). */
export function setSaverOffFlag (on: boolean): Promise<void> {
	return run(on ? `mkdir -p ${HOMEHOOK} && touch ${HOMEHOOK}/no-screensaver` : `rm -f ${HOMEHOOK}/no-screensaver`).then(() => undefined);
}

/** While the ambient screen is up, watch.sh only ends LG's screensaver (which starts again after its timer, as nothing is pressed) instead of
 *  opening Lumen again. */
export function setAmbientShowing (on: boolean): Promise<void> {
	return run(on ? 'touch /tmp/lumen-ambient-on' : 'rm -f /tmp/lumen-ambient-on', 4000).then(() => undefined);
}

/** With the Home watcher running, LG Home would bounce straight back to Lumen: this flag lets it open once. */
export function allowLgHome (): Promise<void> {
	return run('touch /tmp/allow_lg_home', 4000).then(() => undefined);
}

/** An Immich server's address (scheme, host and port only) and an API key, as checked before either reaches the shell. */
const IMMICH_BASE = /^https?:\/\/[A-Za-z0-9.-]+(:\d{1,5})?$/;
const IMMICH_KEY = /^[A-Za-z0-9_-]{16,128}$/;
/** The only Immich paths Lumen asks for. */
const IMMICH_PATHS = [
	/^\/api\/server\/version$/,
	/^\/api\/albums$/,
	/^\/api\/search\/random$/,
	/^\/api\/memories\?for=\d{4}-\d{2}-\d{2}$/,
	/^\/api\/assets\/[0-9a-f-]{36}\/thumbnail\?size=thumbnail$/,
	/^\/api\/faces\?id=[0-9a-f-]{36}$/
];

/** An address and key immichRequest would accept. */
export function immichServerOk (base: string, key: string): {base: boolean; key: boolean} {
	return {base: IMMICH_BASE.test(base), key: IMMICH_KEY.test(key)};
}

/** One request to an Immich server, made with curl on the TV: Immich sends no CORS headers outside its dev mode, so the app's own fetch would
 *  be refused. A JSON body travels as base64. `as: 'base64'` returns the response body base64-encoded (for a thumbnail). */
export function immichRequest (base: string, key: string, path: string, opts: {body?: object; as?: 'text' | 'base64'} = {}): Promise<string> {
	if (!IMMICH_BASE.test(base) || !IMMICH_KEY.test(key) || !IMMICH_PATHS.some((re) => re.test(path))) return Promise.reject(new Error('refused: immich request'));
	const curl = `curl -sf -m 15 -H ${shellQuote('x-api-key: ' + key)} -H 'accept: application/json'`;
	const url = shellQuote(base + path);
	const command = opts.body
		? `echo ${toBase64(JSON.stringify(opts.body))} | base64 -d | ${curl} -H 'content-type: application/json' --data-binary @- ${url}`
		: `${curl} ${url}`;
	return run(opts.as === 'base64' ? `${command} | base64 | tr -d '\\n'` : command, 20000);
}

/** The ids in every stack on an Immich server, as '<primaryAssetId>' or '=<id>' lines (the stacks' own ids are among the second kind).
 *  Only the ids leave the TV, since the full answer carries every photo in every stack. Needs the API key to have stack.read. */
export function immichStackIds (base: string, key: string): Promise<string> {
	if (!IMMICH_BASE.test(base) || !IMMICH_KEY.test(key)) return Promise.reject(new Error('refused: immich request'));
	const curl = `curl -sf -m 15 -H ${shellQuote('x-api-key: ' + key)} -H 'accept: application/json'`;
	return run(`${curl} ${shellQuote(base + '/api/stacks')} | grep -oE '"(primaryAssetId|id)":"[0-9a-f-]{36}"' | sed -e 's/^"primaryAssetId":"//' -e 's/^"id":"/=/' -e 's/"$//'`, 20000);
}

/** Lumen's own install folder: the only place the app may read files from, and the only one copyFontIn writes to. */
const APP_DIR = /^\/media\/developer\/apps\/usr\/palm\/applications\/com\.samrenfrew\.lumen(\.dev)?$/;

/** Copies the SF font that tools/push_sf_font.sh put on the TV into the app's folder (each Channel update replaces the folder). True when copied. */
export function copyFontIn (appDir: string): Promise<boolean> {
	if (!APP_DIR.test(appDir)) return Promise.reject(new Error('refused: app folder'));
	return run(`[ -f ${HOMEHOOK}/fonts/sf.woff2 ] && mkdir -p ${appDir}/fonts && cp ${HOMEHOOK}/fonts/sf.woff2 ${appDir}/fonts/sf.woff2 && echo ok; true`).then((out) => out.trim() === 'ok');
}

/** Writes text to one of FILES. The text travels as base64, so nothing in it is ever read by the shell. */
export function writeFile (name: HomehookFile, text: string): Promise<void> {
	if (FILES.indexOf(name) === -1) return Promise.reject(new Error('unknown file ' + name));
	const b64 = toBase64(text);
	return run(`mkdir -p ${HOMEHOOK} && echo ${b64} | base64 -d > ${HOMEHOOK}/${name}.tmp && mv ${HOMEHOOK}/${name}.tmp ${HOMEHOOK}/${name}`).then(() => undefined);
}

/** The contents of one of FILES, or '' when it does not exist. */
export function readFile (name: HomehookFile): Promise<string> {
	if (FILES.indexOf(name) === -1) return Promise.reject(new Error('unknown file ' + name));
	return run(`[ -f ${HOMEHOOK}/${name} ] && base64 ${HOMEHOOK}/${name} | tr -d '\\n'; true`).then((out) => fromBase64(out.trim()));
}

/** Appends lines to the log and keeps only its last LOG_MAX_BYTES. Calls the bridge directly, without the error hook, so a failing
 *  root service cannot loop by logging its own failure to write the log. */
export function appendLog (text: string): Promise<void> {
	if (!isWebOS()) return Promise.reject(new Error('not on a TV'));
	const b64 = toBase64(text);
	const command = `mkdir -p ${HOMEHOOK} && echo ${b64} | base64 -d >> ${LOG} && if [ $(wc -c < ${LOG}) -gt ${LOG_MAX_BYTES} ]; then tail -c ${LOG_MAX_BYTES / 2} ${LOG} > ${LOG}.tmp && mv ${LOG}.tmp ${LOG}; fi`;
	return lunaCall(LUNA.homebrewExec, {command}, 6000, true).then(() => undefined);
}

function toBase64 (text: string): string {
	const b64 = btoa(unescape(encodeURIComponent(text)));
	if (!BASE64.test(b64)) throw new Error('bad base64');   // cannot happen; the check is what keeps the command safe if btoa ever changes
	return b64;
}

function fromBase64 (b64: string): string {
	if (!b64) return '';
	if (!BASE64.test(b64)) throw new Error('bad base64 from the TV');
	return decodeURIComponent(escape(atob(b64)));
}
