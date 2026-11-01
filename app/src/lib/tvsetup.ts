// Settings > Home and Power, Settings > Privacy and Settings > System > TV scripts: what the TV-side scripts report, read in one root call
// (lib/root.ts tvStatus). The scripts are tools/tv/home-setup.sh and tools/tv/privacy.sh; docs/home-and-power-on.md and docs/privacy.md.

import {isWebOS} from './luna';
import * as root from './root';

export interface TvState {
	/** the installed scripts against the ones in this build: ok, old (differ), missing, or unshipped (this build has none) */
	scripts: string;
	/** the app the TV's Home setting names now */
	home: string;
	/** ok, or why Native Home cannot be set: no-lumen, lumen-category, lghome-category */
	native: string;
	/** the app the power-on copy opens, off, or foreign (a copy someone else wrote) */
	powerOn: string;
	/** ok, or why not: no-bootd, no-table, no-node */
	powerOnSupport: string;
	/** what bootd read at this boot: an app id, off, or unknown */
	powerOnLoaded: string;
	/** withdrawable agreements still on; null when unreadable */
	agreements: number | null;
	/** a backup of the agreements exists, so they can be restored */
	backup: boolean;
	/** on, off, or '' when the TV has no such setting */
	lmt: string;
	livePlus: string;
	hosts: boolean;
	hostsWritable: boolean;
	services: boolean;
	servicesRunning: number;
	updates: boolean;
	node: boolean;
}

export function parseTvState (text: string): TvState {
	const f: Record<string, string> = {};
	text.split('\n').forEach((line) => {
		const eq = line.indexOf('=');
		if (eq > 0) f[line.slice(0, eq).trim()] = line.slice(eq + 1).trim();
	});
	const n = (v: string | undefined) => v === undefined || v === '' || isNaN(Number(v)) ? null : Number(v);
	return {
		scripts: f.scripts || '',
		home: f.home || '',
		native: f.native || '',
		powerOn: f.poweron || '',
		powerOnSupport: f.poweron_support || '',
		powerOnLoaded: f.poweron_loaded || 'unknown',
		agreements: n(f.agreements),
		backup: f.backup === 'yes',
		lmt: f.lmt || '',
		livePlus: (f.liveplus || '').toLowerCase(),
		hosts: f.hosts === 'on',
		hostsWritable: f.hosts_writable === 'yes',
		services: f.services === 'on',
		servicesRunning: n(f.services_running) || 0,
		updates: f.updates === 'on',
		node: f.node === 'yes'
	};
}

/** The folder this build is installed in (the page's own folder on the TV). */
export function appDir (): string {
	return decodeURIComponent(location.pathname).replace(/\/[^/]*$/, '');
}

/** The TV's state, or null off the TV or when the call fails. */
export function readTvState (): Promise<TvState | null> {
	if (!isWebOS()) return Promise.resolve(null);
	return root.tvStatus(appDir()).then(parseTvState, () => null);
}

/** The scripts the rows below need are installed and match this build. */
export function scriptsReady (s: TvState | null): boolean {
	return !!s && s.scripts === 'ok';
}

/** Power-on is chosen, but bootd has not read it yet (it reads its file only at a full restart). */
export function powerOnPending (s: TvState | null): boolean {
	if (!s || s.powerOnLoaded === 'unknown' || s.powerOn === 'foreign') return false;
	return s.powerOn !== s.powerOnLoaded;
}

const NATIVE_WHY: Record<string, string> = {
	'no-lumen': 'No Lumen build is installed where the TV scripts look for it.',
	'lumen-category': 'The TV does not list this build as a Home app. Install a build with "appCategory": ["home"].',
	'lghome-category': "LG Home is not listed as a Home app on this TV, so the way back is not known. Left alone."
};
const POWER_WHY: Record<string, string> = {
	'no-bootd': "This TV's start-up service does not read a replacement file, so this cannot work here.",
	'no-table': "This TV's start-up file has no table of power-on apps, so this cannot work here.",
	'no-node': 'This TV has no node to write the start-up file safely.'
};

/** Why Native Home cannot be set, or '' when it can. */
export function nativeWhy (s: TvState): string { return s.native === 'ok' ? '' : NATIVE_WHY[s.native] || 'The TV did not say.'; }
/** Why the power-on app cannot be set, or '' when it can. */
export function powerOnWhy (s: TvState): string {
	if (s.powerOn === 'foreign') return 'Another tool has written the start-up file. Lumen leaves it alone.';
	return s.powerOnSupport === 'ok' ? '' : POWER_WHY[s.powerOnSupport] || 'The TV did not say.';
}
