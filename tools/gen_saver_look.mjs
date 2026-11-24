import {readFileSync, writeFileSync, mkdirSync, rmSync} from 'node:fs';
import {dirname, join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {Resvg} from '@resvg/resvg-js';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');
const OUT = join(ROOT, 'native/saver/assets');
const ICON_SIZES = [30, 32, 36, 50];
const LABELS = ['Clear', 'Partly cloudy', 'Cloudy', 'Fog', 'Rain', 'Snow', 'Showers', 'Storm', 'High', 'Low', 'Rain from'];

const iconSource = readFileSync(join(ROOT, 'app/src/components/WeatherIcon.tsx'), 'utf8');
const pathConst = (name) => iconSource.match(new RegExp(`const ${name} = '([^']+)'`))[1];
const kindPath = (kind) => {
	const m = iconSource.match(new RegExp(`\\n\\t${kind}: ([A-Z]+|'[^']+')`))[1];
	return m.startsWith("'") ? m.slice(1, -1) : pathConst(m);
};
const KINDS = ['clear', 'partly', 'cloudy', 'fog', 'rain', 'snow', 'storm'];
const NIGHT = {clear: pathConst('MOON'), partly: pathConst('PARTLY_NIGHT')};

const iconDir = join(OUT, 'icons');
rmSync(iconDir, {recursive: true, force: true});
mkdirSync(iconDir, {recursive: true});
for (const kind of KINDS) {
	for (const day of [true, false]) {
		const d = day ? kindPath(kind) : NIGHT[kind] || kindPath(kind);
		const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="${d}" fill="none" stroke="#fff" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg>`;
		for (const size of ICON_SIZES) {
			const png = new Resvg(svg, {fitTo: {mode: 'width', value: size}, background: 'rgba(0,0,0,0)'}).render().asPng();
			writeFileSync(join(iconDir, `${kind}-${day ? 'day' : 'night'}-${size}.png`), png);
		}
	}
}

const i18n = readFileSync(join(ROOT, 'app/src/lib/i18n.ts'), 'utf8');
const LANGS = [...i18n.matchAll(/\{code: '(\w+)', name: '[^']*', locale: '([\w-]+)'\}/g)].map((m) => ({code: m[1], locale: m[2]}));
const ORDER = JSON.parse(i18n.match(/const ORDER = (\[[^\]]*\])/)[1]);
const row = (key) => JSON.parse(i18n.match(new RegExp(`\\n\\t"${key}": (\\[[^\\]]*\\])`))[1]);

const OPTIONS = {weekday: 'long', month: 'long', day: 'numeric'};
const dateText = (locale, d) => d.toLocaleDateString(locale, OPTIONS).replace(',', '');
const partOf = (fmt, d, type) => fmt.formatToParts(d).filter((p) => p.type === type)[0].value;

const stringDir = join(OUT, 'strings');
rmSync(stringDir, {recursive: true, force: true});
mkdirSync(stringDir, {recursive: true});
for (const {code, locale} of LANGS) {
	const fmt = new Intl.DateTimeFormat(locale, OPTIONS);
	const sample = new Date(2026, 10, 24);
	const template = fmt.formatToParts(sample).map((p) => ({weekday: '%W', month: '%M', day: '%D'})[p.type] || p.value).join('').replace(',', '');
	const weekdays = [0, 1, 2, 3, 4, 5, 6].map((i) => partOf(fmt, new Date(2026, 10, 22 + i), 'weekday'));
	const months = [...Array(12).keys()].map((m) => partOf(fmt, new Date(2026, m, 15), 'month'));
	for (let m = 0; m < 12; m++) {
		for (const day of [1, 9, 23]) {
			const d = new Date(2026, m, day);
			const built = template.replace('%W', weekdays[d.getDay()]).replace('%M', months[m]).replace('%D', String(day));
			if (built !== dateText(locale, d)) throw new Error(`${code}: "${built}" is not "${dateText(locale, d)}"`);
		}
	}
	const at = ORDER.indexOf(code);
	const lines = [`date=${template}`, ...weekdays.map((w, i) => `weekday${i}=${w}`), ...months.map((m, i) => `month${i}=${m}`),
		...LABELS.map((key) => `${key}=${at >= 0 && row(key)[at] ? row(key)[at] : key}`)];
	writeFileSync(join(stringDir, `${code}.txt`), lines.join('\n') + '\n');
}
console.log(`icons: ${KINDS.length * 2 * ICON_SIZES.length} in ${iconDir}; strings: ${LANGS.map((l) => l.code).join(' ')} in ${stringDir}`);
