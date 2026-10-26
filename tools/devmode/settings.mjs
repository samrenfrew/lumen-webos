// Reads and writes Lumen's settings in the live page on the TV, over the DevTools tunnel (tools/devmode.sh opens it).
//   node tools/devmode/settings.mjs get                 prints every Lumen key in localStorage as JSON
//   node tools/devmode/settings.mjs set <file>          puts a saved copy back, then reloads Lumen
//   node tools/devmode/settings.mjs home <home.json>    sets the home rows (appIds) and folders, keeps every other setting, then reloads
import {readFileSync} from 'node:fs';

const PORT = process.env.LUMEN_CDP_PORT || 19998;
const CONFIG_KEY = 'launcher-config-v2';
/** the start-up caches (lib/boot.ts) hold the last home rows: dropped on a change so the next start draws the new ones */
const BOOT_CACHES = ['lumen-boot-apps-v1'];

const pages = await fetch(`http://127.0.0.1:${PORT}/json`).then((r) => r.json()).catch(() => null);
const page = pages && pages.find((p) => /com\.samrenfrew\.lumen/.test(p.url));
if (!page) { console.error('Lumen is not open on the TV (or the DevTools tunnel is closed).'); process.exit(2); }
const ws = new WebSocket(page.webSocketDebuggerUrl);
await new Promise((ok, fail) => { ws.onopen = ok; ws.onerror = fail; });
let seq = 0; const waiting = new Map();
ws.onmessage = (m) => { const msg = JSON.parse(m.data); if (waiting.has(msg.id)) { waiting.get(msg.id)(msg); waiting.delete(msg.id); } };
const evaluate = (expression) => new Promise((ok, fail) => {
	const id = ++seq;
	waiting.set(id, (msg) => msg.result && msg.result.exceptionDetails ? fail(new Error(msg.result.exceptionDetails.text)) : ok(msg.result.result.value));
	ws.send(JSON.stringify({id, method: 'Runtime.evaluate', params: {expression, returnByValue: true}}));
});

const [cmd, file] = process.argv.slice(2);
if (cmd === 'get') {
	console.log(await evaluate(`JSON.stringify(Object.keys(localStorage).filter((k) => /^(launcher|lumen)-/.test(k)).reduce((o, k) => (o[k] = localStorage.getItem(k), o), {}), null, 1)`));
} else if (cmd === 'set') {
	const saved = JSON.parse(readFileSync(file, 'utf8'));
	if (typeof saved[CONFIG_KEY] !== 'string') throw new Error(`${file} has no ${CONFIG_KEY}`);
	await evaluate(`(() => { const s = ${JSON.stringify(saved)}; Object.keys(localStorage).filter((k) => /^(launcher|lumen)-/.test(k)).forEach((k) => localStorage.removeItem(k)); Object.keys(s).forEach((k) => localStorage.setItem(k, s[k])); setTimeout(() => location.reload(), 100); return 1; })()`);
	console.log(`Settings restored from ${file}; Lumen is reloading.`);
} else if (cmd === 'home') {
	const home = JSON.parse(readFileSync(file, 'utf8'));
	if (!Array.isArray(home.appIds) || !Array.isArray(home.folders)) throw new Error(`${file} needs appIds and folders`);
	await evaluate(`(() => { const c = JSON.parse(localStorage.getItem('${CONFIG_KEY}') || '{}'); c.appIds = ${JSON.stringify(home.appIds)}; c.folders = ${JSON.stringify(home.folders)}; localStorage.setItem('${CONFIG_KEY}', JSON.stringify(c)); ${JSON.stringify(BOOT_CACHES)}.forEach((k) => localStorage.removeItem(k)); setTimeout(() => location.reload(), 100); return 1; })()`);
	console.log(`Home screen set: ${home.appIds.length} apps, ${home.folders.length} folders; Lumen is reloading.`);
} else {
	console.error('Usage: node tools/devmode/settings.mjs get | set <file> | home <home.json>');
	process.exit(1);
}
ws.close();
