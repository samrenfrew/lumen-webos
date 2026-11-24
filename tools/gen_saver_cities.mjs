import {readFileSync, writeFileSync} from 'node:fs';

const root = new URL('../', import.meta.url);
const text = readFileSync(new URL('app/src/config/cities.ts', root), 'utf8');
const city = /"name":\s*"([^"]+)",\s*"lat":\s*(-?[\d.]+),\s*"lon":\s*(-?[\d.]+)/g;
const lines = [];
for (const m of text.matchAll(city)) lines.push(`${m[1]}\t${m[2]}\t${m[3]}`);
writeFileSync(new URL('native/saver/assets/cities.tsv', root), lines.join('\n') + '\n');
console.log(lines.length + ' cities');
