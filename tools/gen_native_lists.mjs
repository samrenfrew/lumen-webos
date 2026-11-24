import {readFileSync, writeFileSync} from 'node:fs';

const root = new URL('..', import.meta.url);
const read = (file) => readFileSync(new URL(file, root), 'utf8');
const clean = (text) => (text || '').replace(/[\t\n]/g, ' ');

function cloudUrl(id, ext, u) {
	if (!u) return `https://i.redd.it/${id}.${ext === 'j' ? 'jpeg' : 'png'}`;
	if (u.startsWith('w:')) return `https://thumb.wikimedia.org/wikipedia/commons/thumb/${u.slice(2)}/1920px-${u.slice(u.lastIndexOf('/') + 1)}`;
	if (u.startsWith('c:')) return `https://openaccess-cdn.clevelandart.org/${u.slice(2)}/${u.slice(2)}_print.jpg`;
	return 'https://' + u;
}

const pictures = [];
for (const m of read('app/src/config/wallpapers.ts').matchAll(/^\t\{id: "([^"]+)", ext: "([jp])", title: "((?:[^"\\]|\\.)*)", category: "([^"]*)"(?:, url: "([^"]*)")?(?:, credit: "((?:[^"\\]|\\.)*)")?\},?$/gm)) {
	pictures.push([m[1], cloudUrl(m[1], m[2], m[5]), clean(JSON.parse(`"${m[3]}"`)), clean(m[6] ? JSON.parse(`"${m[6]}"`) : ''), m[4]].join('\t'));
}
if (pictures.length < 100) throw new Error(`only ${pictures.length} pictures found`);
writeFileSync(new URL('native/lumen/assets/wallpapers.tsv', root), pictures.join('\n') + '\n');

const cities = [];
const countries = JSON.parse(read('app/src/config/cities.ts').match(/export const COUNTRIES: Country\[\] = (\[[\s\S]*?\n\]);/)[1]);
for (const country of countries) {
	for (const city of country.cities) {
		if (!cities.some((line) => line.startsWith(city.name + '\t'))) cities.push([city.name, city.lat, city.lon, country.code, clean(country.name)].join('\t'));
	}
}
if (cities.length < 50) throw new Error(`only ${cities.length} cities found`);
writeFileSync(new URL('native/lumen/assets/cities.tsv', root), cities.join('\n') + '\n');
console.log(`${pictures.length} pictures, ${cities.length} cities`);
