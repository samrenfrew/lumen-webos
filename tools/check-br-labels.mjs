import {readFileSync} from 'node:fs';

const i18n = readFileSync(new URL('../app/src/lib/i18n.ts', import.meta.url), 'utf8');
const requiredLabels = new Map([
	['Control Center', 'Central de Controle'],
	['Theme', 'Tema'],
	['Backlight', 'Brilho OLED']
]);

const languages = i18n.match(/export const LANGS\s*=\s*\[([\s\S]*?)\];/);
const portuguese = languages?.[1].match(/\{[^{}]*\bcode:\s*['"]pt['"][^{}]*\}/)?.[0];
const portugueseLocale = portuguese?.match(/\blocale:\s*['"]([^'"]+)['"]/)?.[1];
const order = i18n.match(/const ORDER\s*=\s*\[([\s\S]*?)\];/);
const portugueseColumn = order ? [...order[1].matchAll(/['"]([^'"]+)['"]/g)].findIndex(([, code]) => code === 'pt') : -1;
const failures = [];

if (portugueseLocale !== 'pt-BR') {
	failures.push(`Portuguese locale must be pt-BR (found ${portugueseLocale ?? 'no Portuguese language entry'})`);
}

if (portugueseColumn < 0) {
	failures.push('Portuguese column is missing from ORDER');
} else {
	for (const [key, expected] of requiredLabels) {
		const escapedKey = key.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
		const entry = i18n.match(new RegExp(`^\\s*"${escapedKey}"\\s*:\\s*\\[([\\s\\S]*?)\\]`, 'm'));
		const translations = entry ? [...entry[1].matchAll(/"((?:\\.|[^"\\])*)"/g)].map(([, value]) => JSON.parse(`"${value}"`)) : [];
		const actual = translations[portugueseColumn];

		if (actual !== expected) {
			failures.push(`${key} must be ${JSON.stringify(expected)} in the Portuguese column (found ${actual === undefined ? 'no entry' : JSON.stringify(actual)})`);
		}
	}
}

if (failures.length > 0) {
	throw new Error(`Brazilian Portuguese display labels are incorrect in app/src/lib/i18n.ts:\n- ${failures.join('\n- ')}`);
}
