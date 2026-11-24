import {readFileSync, writeFileSync} from 'node:fs';

const root = new URL('..', import.meta.url);
const source = readFileSync(new URL('app/src/config/wallpapers.ts', root), 'utf8');

function cloudUrl(id, ext, u) {
	if (!u) return `https://i.redd.it/${id}.${ext === 'j' ? 'jpeg' : 'png'}`;
	if (u.startsWith('w:')) return `https://thumb.wikimedia.org/wikipedia/commons/thumb/${u.slice(2)}/1920px-${u.slice(u.lastIndexOf('/') + 1)}`;
	if (u.startsWith('c:')) return `https://openaccess-cdn.clevelandart.org/${u.slice(2)}/${u.slice(2)}_print.jpg`;
	return 'https://' + u;
}

const lines = [];
for (const m of source.matchAll(/^\t\{id: "([^"]+)", ext: "([jp])".*?(?:url: "([^"]*)")?(?:, credit: "[^"]*")?\},?$/gm)) {
	lines.push(`${m[1]}\t${cloudUrl(m[1], m[2], m[3])}`);
}
if (lines.length < 100) throw new Error(`only ${lines.length} pictures found`);
writeFileSync(new URL('native/saver/assets/wallpapers.tsv', root), lines.join('\n') + '\n');
console.log(`${lines.length} pictures`);
