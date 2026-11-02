// Settings > Brightness: the TV's backlight, and Eye Comfort if chosen, at three times of day. Lumen keeps the schedule and writes it to
// /var/lib/homehook/backlight-schedule; tools/tv/watch.sh applies it as root, so it runs while Lumen is closed. While Lumen's screensaver
// shows, watch.sh holds the backlight back, and the screensaver puts back the level for the time it ends (lib/tvcontrols.ts restoreBacklight).

import {isWebOS} from './luna';
import {writeFile} from './root';
import type {UserConfig} from './storage';

interface Step { at: string; backlight: number; eye: 'on' | 'off' | 'keep' }

function steps (c: UserConfig): Step[] {
	const eye = (on: boolean): Step['eye'] => c.blEye === 'keep' ? 'keep' : on ? 'on' : 'off';
	return [
		{at: c.blDayAt, backlight: c.blDay, eye: eye(false)},
		{at: c.blEveningAt, backlight: c.blEvening, eye: eye(c.blEye === 'evening')},
		{at: c.blNightAt, backlight: c.blNight, eye: eye(c.blEye === 'evening' || c.blEye === 'night')}
	].sort((a, b) => a.at < b.at ? -1 : a.at > b.at ? 1 : 0);
}

/** The backlight the schedule wants at `when`, or null with the schedule off. Before the day's first step, the last one (from yesterday) holds. */
export function scheduledBacklight (c: UserConfig, when = new Date()): number | null {
	if (!c.brightnessOn) return null;
	const list = steps(c);
	const now = `${String(when.getHours()).padStart(2, '0')}:${String(when.getMinutes()).padStart(2, '0')}`;
	let cur = list[list.length - 1];
	list.forEach((s) => { if (s.at <= now) cur = s; });
	return cur.backlight;
}

/** The file watch.sh reads: one "HH:MM level on|off|keep" line per step, or nothing with the schedule off. */
export function writeBrightnessSchedule (c: UserConfig): Promise<void> {
	if (!isWebOS()) return Promise.resolve();
	const text = c.brightnessOn ? steps(c).map((s) => `${s.at} ${s.backlight} ${s.eye}\n`).join('') : '';
	return writeFile('backlight-schedule', text);
}
