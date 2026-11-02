// Control Center: what the TV lets a normal web app do (volume, mute) and what needs the Homebrew Channel's root service
// (picture mode, backlight, Eye Comfort, screen off, restart), through lib/root.ts. Off the TV (the dev preview) the values are kept in memory.

import {isWebOS, lunaCall} from './luna';
import * as root from './root';
import {readCache, writeCache} from './boot';

const AUDIO = 'luna://com.webos.audio/';

/** the TV's own names for the picture modes it offers on an SDR picture */
export const PICTURE_MODES = ['vivid', 'normal', 'eco', 'cinema', 'sports', 'game', 'filmMaker'];
export const PICTURE_LABEL: Record<string, string> = {vivid: 'Vivid', normal: 'Standard', eco: 'Eco', cinema: 'Cinema', sports: 'Sports', game: 'Game', filmMaker: 'FILMMAKER', expert1: 'Expert (bright room)', expert2: 'Expert (dark room)', hdrEffect: 'HDR Effect', hdrStandard: 'HDR Standard', hdrCinema: 'HDR Cinema', hdrGame: 'HDR Game'};
/** the TV's own names for the sound modes (Settings > Sound > Sound Mode) that every model offers */
export const SOUND_MODES = ['standard', 'news', 'movie', 'music', 'sports', 'game', 'aiSound'];
export const SOUND_LABEL: Record<string, string> = {standard: 'Standard', news: 'Clear Voice', movie: 'Cinema', music: 'Music', sports: 'Sports', game: 'Game Optimizer', aiSound: 'AI Sound', aiSoundPlus: 'AI Sound Pro', custom: 'Custom'};

export interface Controls {
	volume: number;
	muted: boolean;
	picture: string;
	sound: string;
	/** 0-100, for the picture mode in use */
	backlight: number;
	eyeComfort: boolean;
}

const mock: Controls = {volume: 18, muted: false, picture: 'normal', sound: 'standard', backlight: 100, eyeComfort: false};

export function readControls (): Promise<Controls> {
	if (!isWebOS()) return Promise.resolve({...mock});
	const out: Controls = {volume: 0, muted: false, picture: 'normal', sound: 'standard', backlight: 100, eyeComfort: false};
	const audio = lunaCall<{volume?: number; muteStatus?: boolean}>(AUDIO + 'getVolume', {}).then((r) => {
		out.volume = Number(r.volume) || 0;
		out.muted = r.muteStatus === true;
	}).catch(() => undefined);
	const picture = root.getSetting('pictureMode').then((v) => { if (v) out.picture = v; }).catch(() => undefined);
	const sound = root.getSetting('soundMode').then((v) => { if (v) out.sound = v; }).catch(() => undefined);
	const backlight = readBacklight().then((v) => { if (v !== null) out.backlight = v; });
	const eye = root.getSetting('eyeComfortMode').then((v) => { out.eyeComfort = v === 'on'; }).catch(() => undefined);
	return Promise.all([audio, picture, sound, backlight, eye]).then(() => out);
}

/** The backlight of the picture mode in use, or null when it can't be read. Read again after a picture mode change: each mode has its own. */
export function readBacklight (): Promise<number | null> {
	if (!isWebOS()) return Promise.resolve(mock.backlight);
	return root.getSetting('backlight').then((v) => {
		const n = Number(v);
		return v !== '' && Number.isInteger(n) && n >= 0 && n <= 100 ? n : null;
	}).catch(() => null);
}

export function setVolume (volume: number): Promise<void> {
	mock.volume = volume;
	if (!isWebOS()) return Promise.resolve();
	return lunaCall(AUDIO + 'setVolume', {volume}).then(() => undefined);
}

export function setMuted (muted: boolean): Promise<void> {
	mock.muted = muted;
	if (!isWebOS()) return Promise.resolve();
	return lunaCall(AUDIO + 'setMuted', {muted}).then(() => undefined);
}

/** Only the TV's known mode names are ever put into the root command. */
export function setPictureMode (mode: string): Promise<void> {
	mock.picture = mode;
	if (!isWebOS()) return Promise.resolve();
	if (PICTURE_MODES.indexOf(mode) === -1) return Promise.reject(new Error('unknown picture mode'));
	return root.setSetting('pictureMode', mode);
}

/** Only the TV's known sound mode names are ever put into the root command. */
export function setSoundMode (mode: string): Promise<void> {
	mock.sound = mode;
	if (!isWebOS()) return Promise.resolve();
	if (SOUND_MODES.indexOf(mode) === -1) return Promise.reject(new Error('unknown sound mode'));
	return root.setSetting('soundMode', mode);
}

/** 0-100, whole numbers only. */
export function setBacklight (level: number): Promise<void> {
	mock.backlight = level;
	if (!isWebOS()) return Promise.resolve();
	if (!Number.isInteger(level) || level < 0 || level > 100) return Promise.reject(new Error('bad backlight level'));
	return root.setSetting('backlight', String(level));
}

/** The backlight before the screensaver lowered it, kept until it is put back, so a launch after a crash can still put it back. */
const SAVED_BACKLIGHT_KEY = 'lumen-screensaver-backlight-v1';
let dimmedHere = false;
/** Settings > Brightness: the level for now, which the screensaver puts back instead of the one it saved (that may be from an earlier part of the day) */
let scheduled: () => number | null = () => null;
export function setBacklightSchedule (fn: () => number | null): void { scheduled = fn; }

/** Lowers the backlight to `level` for the screensaver (never raises it), keeping the old value to put back. */
export function dimForScreensaver (level: number, stillWanted: () => boolean): void {
	readBacklight().then((before) => {
		if (before === null || before <= level || !stillWanted()) return;
		if (readCache<number>(SAVED_BACKLIGHT_KEY) === null) writeCache(SAVED_BACKLIGHT_KEY, before);
		dimmedHere = true;
		return setBacklight(level);
	}).catch(() => undefined);
}

/** Puts back the backlight the screensaver lowered, if it did: the schedule's level for now, else the one it saved. Safe to call at any time. */
export function restoreBacklight (): void {
	const saved = readCache<number>(SAVED_BACKLIGHT_KEY);
	if (saved === null) return;
	const now = scheduled();
	setBacklight(now === null ? saved : now).then(() => { writeCache(SAVED_BACKLIGHT_KEY, null); dimmedHere = false; }).catch(() => undefined);
}

/** At start-up: a backlight left low by a screensaver that never ended (Lumen closed or crashed) is put back, unless this launch dimmed it. */
export function restoreLeftoverBacklight (): void {
	if (!dimmedHere) restoreBacklight();
}

export function setEyeComfort (on: boolean): Promise<void> {
	mock.eyeComfort = on;
	if (!isWebOS()) return Promise.resolve();
	return root.setSetting('eyeComfortMode', on ? 'on' : 'off');
}

/** Screen off, the TV keeps running (a press on the remote wakes it). */
export function screenOff (): Promise<void> {
	if (!isWebOS()) return Promise.reject(new Error('not on a TV'));
	return root.screenOff();
}

export function restartTv (): Promise<void> {
	if (!isWebOS()) return Promise.reject(new Error('not on a TV'));
	return root.reboot();
}
