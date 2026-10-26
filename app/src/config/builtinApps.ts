// Tiles on the home screen that are not installed apps, as tvOS puts its own Settings in the grid. They are listed in Settings > Apps
// like any app, so they can be moved, hidden or put in a folder.
import {packIcon, type AppEntry} from '../lib/apps';

export const BUILTIN = {settings: 'lumen:settings', inputs: 'lumen:inputs', tvSettings: 'com.palm.app.settings'} as const;

export const BUILTIN_APPS: AppEntry[] = [
	{id: BUILTIN.settings, title: 'Settings', color: '#8a8d99', glyph: 'gear'},
	{id: BUILTIN.tvSettings, title: 'TV Settings', color: '#4d4e6c', glyph: 'picture'},
	{id: BUILTIN.inputs, title: 'Inputs', color: '#3a64c8', glyph: 'inputs'}
];

/** The installed apps plus the built-in tiles (an installed TV Settings keeps its own icon). A tile in the icon pack wins over the glyph. */
export function withBuiltins (apps: AppEntry[]): AppEntry[] {
	return apps.concat(BUILTIN_APPS.filter((b) => !apps.some((a) => a.id === b.id)).map((b) => {
		const icon = packIcon(b.id, b.title);
		return icon ? {...b, iconUrl: icon, glyph: undefined} : b;
	}));
}
