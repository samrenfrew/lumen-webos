import {readFileSync, writeFileSync} from 'node:fs';

const root = new URL('..', import.meta.url);
const lines = [];
for (const file of ['app/src/config/catalog.ts', 'app/src/config/systemApps.ts']) {
	const source = readFileSync(new URL(file, root), 'utf8');
	for (const m of source.matchAll(/^\t\{id: '([^']+)', title: '([^']+)'(?:, icon: '([^']+)')?(?:, color: '(#[0-9a-fA-F]{6})')?\},?$/gm)) {
		if (!lines.some((line) => line.startsWith(m[1] + '\t'))) lines.push([m[1], m[2], m[3] || '', m[4] || ''].join('\t'));
	}
}
if (lines.length < 10) throw new Error(`only ${lines.length} apps found`);
writeFileSync(new URL('native/lumen/assets/catalog.tsv', root), lines.join('\n') + '\n');
console.log(`${lines.length} apps`);
