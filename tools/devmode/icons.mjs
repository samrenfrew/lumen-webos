// Draws the icon pack from tools/devmode/apps.mjs and writes the catalogue that goes with it.
//   node tools/devmode/icons.mjs            → app/private/iconpack/<title>.png, app/src/config/catalog.local.ts,
//                                             tools/devmode/home.json (the home rows and folders) and build/icons-sheet.png (a preview)
// Needs Google Chrome (CHROME in the environment if it is not in /Applications) and `npm --prefix tools install`.
// Brand logos are read from tools/logos/raw/ (gitignored) when they are there; see tools/devmode/apps.mjs.
import {existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {spawn} from 'node:child_process';
import {tmpdir} from 'node:os';
import {dirname, join, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import * as si from 'simple-icons';
import {APPS, FOLDERS, HOME} from './apps.mjs';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const OUT = join(ROOT, 'app/private/iconpack');
const CHROME = process.env.CHROME || '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
const FONT = join(ROOT, 'app/src/assets/geist-wght.woff2');
const LOGOS = join(ROOT, 'tools/logos/raw');
const W = 800, H = 480, MARK = 196;
/** Tiles are drawn at 800×480, the size of the pack from tools/gen_iconpack.py. A full-size tile decodes to 1.5 MB, so on a TV with little
 *  memory to spare LUMEN_TILE_SCALE=0.45 saves them at 360×216, still sharp for a tile about 240 px wide on screen. */
const TILE_SCALE = Number(process.env.LUMEN_TILE_SCALE) || 1;
const slug = (s) => s.toLowerCase().replace(/[^a-z0-9]+/g, '');

function lucide (name) {
	const svg = readFileSync(join(ROOT, `tools/node_modules/lucide-static/icons/${name}.svg`), 'utf8').replace(/<!--.*?-->/s, '');
	return svg.replace(/width="24"/, `width="${MARK}"`).replace(/height="24"/, `height="${MARK}"`).replace(/stroke-width="2"/, 'stroke-width="1.75"').replace(/currentColor/g, '#fff');
}

/** The brand's own logo file for a {logo} mark, if it has been downloaded. */
function logoFile (name) {
	for (const ext of ['svg', 'png', 'webp']) if (existsSync(join(LOGOS, `${name}.${ext}`))) return join(LOGOS, `${name}.${ext}`);
	return null;
}

function mark (m) {
	const file = m.logo && logoFile(m.logo);
	if (file) {
		// Turned white so it matches the rest of the set whatever colours the brand uses: every opaque pixel ('mono'), or for a logo that
		// sits on its own coloured box only the light parts, so the box drops out and the lettering stays ('luma'). Fitted into a fixed box,
		// since many SVGs state a tiny size or none.
		const [bw, bh] = m.box || [440, 170];
		return `<img src="file://${file}" style="width:${bw}px;height:${bh}px;object-fit:contain;filter:${m.mode === 'luma' ? 'url(#luma)' : 'brightness(0) invert(1)'}">`;
	}
	const size = Math.round(MARK * (m.scale || (m.si ? 0.86 : 1)));
	if (m.si) {
		const icon = si[m.si];
		if (!icon) throw new Error(`Simple Icons has no ${m.si}`);
		return `<svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="#fff"><path d="${icon.path}"/></svg>`;
	}
	if (m.lucide) return lucide(m.lucide);
	if (m.svg) return `<svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="#fff">${m.svg}</svg>`;
	const smile = m.smile ? `<svg class="smile" width="300" height="48" viewBox="0 0 300 48" fill="none" stroke="#fff" stroke-width="11" stroke-linecap="round"><path d="M14 10 Q150 62 268 14"/><path d="M246 6 L272 12 L262 36"/></svg>` : '';
	return `<div class="word" style="font-size:${m.size}px;font-weight:${m.weight || 650}">${m.word}${smile}</div>`;
}

function tile (app) {
	return `<div class="tile" style="--from:${app.from};--to:${app.to}"><div class="mark">${mark(app.mark)}</div></div>`;
}

const css = `
@font-face { font-family: Geist; src: url("file://${FONT}") format("woff2"); font-weight: 100 900; }
* { box-sizing: border-box; margin: 0; }
body { background: #000; font-family: Geist, sans-serif; }
.tile { position: relative; width: ${W}px; height: ${H}px; overflow: hidden; display: flex; align-items: center; justify-content: center;
	background:
		radial-gradient(85% 75% at 14% 0%, rgba(255,255,255,0.17) 0%, rgba(255,255,255,0) 62%),
		linear-gradient(to top, rgba(0,0,0,0.22) 0%, rgba(0,0,0,0) 48%),
		linear-gradient(148deg, var(--from) 0%, var(--to) 100%); }
.mark { display: flex; filter: drop-shadow(0 10px 26px rgba(0,0,0,0.30)); }
.word { position: relative; color: #fff; letter-spacing: -0.035em; line-height: 1; white-space: nowrap; text-align: center; }
.word .smile { position: absolute; left: 50%; bottom: -54px; transform: translateX(-46%); }
.sheet { display: grid; grid-template-columns: repeat(6, 264px); gap: 34px 26px; padding: 48px; background: #101114; width: max-content; }
.sheet figure { display: flex; flex-direction: column; align-items: center; gap: 12px; }
.sheet .tile { transform: scale(0.33); transform-origin: top left; }
.sheet .frame { width: 264px; height: 158px; border-radius: 22px; overflow: hidden; box-shadow: 0 10px 30px rgba(0,0,0,0.45); }
.sheet figcaption { color: #c9ccd4; font-size: 17px; letter-spacing: -0.01em; }
`;

// white where the logo is light, clear where it is dark (alpha = 3 × luminance − 0.95), then cut to the logo's own shape
const FILTERS = `<svg width="0" height="0" style="position:absolute"><filter id="luma" color-interpolation-filters="sRGB">
	<feColorMatrix type="matrix" values="0 0 0 0 1  0 0 0 0 1  0 0 0 0 1  0.64 2.15 0.21 0 -0.95"/><feComposite in2="SourceAlpha" operator="in"/></filter></svg>`;
const strip = `<html><head><style>${css}</style></head><body>${FILTERS}${APPS.map(tile).join('')}</body></html>`;
const sheet = `<html><head><style>${css}</style></head><body>${FILTERS}<div class="sheet">${APPS.map((a) => `<figure><div class="frame">${tile(a)}</div><figcaption>${a.title}</figcaption></figure>`).join('')}</div></body></html>`;

// ---------- render with headless Chrome over DevTools ----------
const tmp = mkdtempSync(join(tmpdir(), 'lumen-icons-'));
writeFileSync(join(tmp, 'strip.html'), strip);
writeFileSync(join(tmp, 'sheet.html'), sheet);
const port = 9400 + Math.floor(Math.random() * 400);
const chrome = spawn(CHROME, ['--headless=new', `--remote-debugging-port=${port}`, `--user-data-dir=${join(tmp, 'profile')}`, '--allow-file-access-from-files',
	'--hide-scrollbars', `--window-size=${W},${H}`, 'about:blank'], {stdio: 'ignore'});
const wait = (ms) => new Promise((ok) => setTimeout(ok, ms));
let target;
for (let i = 0; i < 50 && !target; i++) {
	await wait(200);
	target = await fetch(`http://127.0.0.1:${port}/json`).then((r) => r.json()).then((l) => l.find((t) => t.type === 'page')).catch(() => undefined);
}
if (!target) { chrome.kill(); throw new Error('Chrome did not start'); }
const ws = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((ok, fail) => { ws.onopen = ok; ws.onerror = fail; });
let seq = 0; const waiting = new Map();
ws.onmessage = (m) => { const msg = JSON.parse(m.data); if (waiting.has(msg.id)) { waiting.get(msg.id)(msg); waiting.delete(msg.id); } };
const send = (method, params = {}) => new Promise((ok) => { const id = ++seq; waiting.set(id, ok); ws.send(JSON.stringify({id, method, params})); });
const open = async (file) => {
	await send('Page.navigate', {url: `file://${join(tmp, file)}`});
	await wait(400);
	await send('Runtime.evaluate', {expression: 'document.fonts.ready.then(() => 1)', awaitPromise: true});
	await wait(150);
};
const shot = async (clip, scale = 1) => Buffer.from((await send('Page.captureScreenshot', {format: 'png', clip: {...clip, scale}, captureBeyondViewport: true})).result.data, 'base64');

try {
	await send('Page.enable');
	// only these tiles are replaced: anything else in the folder (tools/gen_iconpack.py's tiles) stays
	mkdirSync(OUT, {recursive: true});
	await open('strip.html');
	await send('Runtime.evaluate', {expression: 'Promise.all([...document.images].map((i) => i.decode().catch(() => 0))).then(() => 1)', awaitPromise: true});
	// named by title: lib/apps.ts matches the pack by id or by title, so one tile covers all of an app's ids
	for (let i = 0; i < APPS.length; i++) writeFileSync(join(OUT, `${slug(APPS[i].title)}.png`), await shot({x: 0, y: i * H, width: W, height: H}, TILE_SCALE));
	await open('sheet.html');
	await send('Runtime.evaluate', {expression: 'Promise.all([...document.images].map((i) => i.decode().catch(() => 0))).then(() => 1)', awaitPromise: true});
	const size = JSON.parse((await send('Runtime.evaluate', {expression: 'JSON.stringify((({width, height}) => ({width, height}))(document.querySelector(".sheet").getBoundingClientRect()))', returnByValue: true})).result.result.value);
	mkdirSync(join(ROOT, 'build'), {recursive: true});
	writeFileSync(join(ROOT, 'build/icons-sheet.png'), await shot({x: 0, y: 0, width: Math.ceil(size.width), height: Math.ceil(size.height)}));
} finally {
	ws.close();
	chrome.kill();
	await wait(300);
	rmSync(tmp, {recursive: true, force: true});
}

// ---------- the catalogue and the home rows ----------
/** each app's id followed by its other ids, so the row keeps its place whichever one is installed (Lumen skips the ones that are not) */
function expand (ids) {
	return ids.flatMap((id) => { const app = APPS.find((a) => a.id === id); return [id].concat(app && app.ids || []); });
}
const apps = APPS.filter((a) => !a.builtin);
writeFileSync(join(ROOT, 'app/src/config/catalog.local.ts'), `// Made by tools/devmode/icons.mjs from tools/devmode/apps.mjs: edit that file, not this one.
import type {CatalogApp} from './catalog';

export const EXTRA_APPS: CatalogApp[] = [
${apps.flatMap((a) => [a.id].concat(a.ids || []).map((id) => `\t{id: ${JSON.stringify(id)}, title: ${JSON.stringify(a.title)}, color: ${JSON.stringify(a.from)}}`)).join(',\n')}
];

export const DEFAULT_APP_IDS: string[] = ${JSON.stringify(expand(HOME))};
`);
writeFileSync(join(ROOT, 'tools/devmode/home.json'), JSON.stringify({appIds: expand(HOME), folders: FOLDERS.map((f) => ({...f, appIds: expand(f.appIds)}))}, null, '\t') + '\n');
const missing = APPS.filter((a) => a.mark.logo && !logoFile(a.mark.logo)).map((a) => a.mark.logo);
if (missing.length) console.log(`No logo file yet (drawn from the fallback): ${missing.join(', ')}`);
console.log(`${APPS.length} tiles in app/private/iconpack/, catalog.local.ts with ${apps.length} apps, preview in build/icons-sheet.png`);
