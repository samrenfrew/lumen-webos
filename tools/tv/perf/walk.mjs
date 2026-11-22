// Walks the common parts of Lumen on the TV and records frames + a light trace per segment.
// usage: node walk.mjs <out.json> [segment,segment...]
import {writeFileSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
const PORT = 19998;
const GAP = 380; // ms between key presses, a quick but human pace

const relaunch = () => execFileSync('ssh', ['-tt', '-o', 'BatchMode=yes', '-o', 'LogLevel=ERROR', 'root@192.168.0.20', `luna-send -n 1 luna://com.webos.applicationManager/launch '{"id":"com.samrenfrew.lumen"}'`]);

const SEGMENTS = {
	dock: ['right', 'right', 'right', 'right', 'right', 'right', 'left', 'left', 'left', 'left', 'left', 'left'],
	rows: ['down', 'right', 'right', 'right', 'right', 'right', 'left', 'left', 'left', 'left', 'left', 'down', 'right', 'right', 'right', 'right', 'left', 'left', 'left', 'left', 'up', 'up'],
	cc: ['up', {ok: `!!document.querySelector('[data-top=controls].pill--focused')`}, 'enter', 900, 'down', 'down', 'up', 'up', 'enter', 900, 'down', 'up', 'back', 900, 'back', 600],
	guide: ['down', 'down', 'left', 'left', 'left', 'left', 'left', 'left', {ok: `(document.querySelector('.tile--focused')||{}).textContent === 'Guide'`}, 'enter', 1500, 'down', 'down', 'down', 'down', 'down', 'down', 'up', 'up', 'up', 'back', 900],
	vert: ['down', 900, 'up', 900, 'down', 900, 'up', 900, 'down', 900, 'up', 900, 'down', 900, 'up', 900],
	settings: ['up', 'right', {ok: `!!document.querySelector('[data-top=settings].pill--focused')`}, 'enter', 1500, 'right', 'right', 'left', 'left', 'down', 'down', 'down', 'down', 'down', 'down', 'up', 'up', 'up', 'back', 900],
};
const LIVECSS = process.env.LIVECSS || '';
const LIVEJS = process.env.LIVEJS || '';
const pick = (process.argv[3] || Object.keys(SEGMENTS).join(',')).split(',');

const pages = await fetch(`http://127.0.0.1:${PORT}/json`).then((r) => r.json());
let page = pages.find((p) => /lumen/i.test(p.url));
const ws = new WebSocket(page.webSocketDebuggerUrl);
await new Promise((ok, fail) => { ws.onopen = ok; ws.onerror = fail; });
let seq = 0; const waiting = new Map(); const events = []; let done;
ws.onmessage = (m) => {
	const msg = JSON.parse(m.data);
	if (msg.method === 'Tracing.dataCollected') events.push(...msg.params.value);
	else if (msg.method === 'Tracing.tracingComplete') done();
	else if (waiting.has(msg.id)) { waiting.get(msg.id)(msg); waiting.delete(msg.id); }
};
const send = (method, params = {}) => new Promise((ok) => { const id = ++seq; waiting.set(id, ok); ws.send(JSON.stringify({id, method, params})); });
const ev = async (expression) => (await send('Runtime.evaluate', {expression, returnByValue: true, awaitPromise: true})).result?.result?.value;
const sleep = (ms) => new Promise((ok) => setTimeout(ok, ms));
const KEYS = {left: [37, 'ArrowLeft'], up: [38, 'ArrowUp'], right: [39, 'ArrowRight'], down: [40, 'ArrowDown'], enter: [13, 'Enter'], back: [461, 'GoBack']};
const key = async (k) => { const [code, name] = KEYS[k]; for (const type of ['rawKeyDown', 'keyUp']) await send('Input.dispatchKeyEvent', {type, windowsVirtualKeyCode: code, nativeVirtualKeyCode: code, key: name, code: name}); };
const mark = (name) => ev(`(() => { window.__marks.push([${JSON.stringify(name)}, performance.now()]); console.timeStamp(${JSON.stringify(name)}); return 1; })()`);

await ev(`(() => { window.__f = []; window.__marks = []; let on = true; window.__stopF = () => { on = false; return window.__f; }; const t = (n) => { window.__f.push(n); if (on) requestAnimationFrame(t); }; if (!${process.env.NORAF ? 'true' : 'false'}) requestAnimationFrame(t); return 1; })()`);
await send('Tracing.start', {traceConfig: {includedCategories: ['devtools.timeline', 'viz', 'gpu', 'benchmark', 'toplevel'], recordMode: 'recordAsMuchAsPossible'}, transferMode: 'ReportEvents'});
await sleep(500);
for (const name of pick) {
	relaunch();         // resets the home screen to the glass bar, not engaged
	await sleep(4500);  // the lower rows stay hidden for 4 s after Lumen comes back
	if (LIVECSS) await ev(`(() => { let s = document.getElementById('live'); if (!s) { s = document.createElement('style'); s.id = 'live'; document.head.appendChild(s); } s.textContent = ${JSON.stringify(LIVECSS)}; return 1; })()`);
	if (LIVEJS) await ev(LIVEJS);
	if ((await ev('document.visibilityState')) !== 'visible') { console.log('Lumen is not in front; stopping'); break; }
	await key('up');    // the first key only engages focus
	await sleep(800);
	await mark(`start:${name}`);
	const seen = new Set();
	let aborted = false;
	for (const k of SEGMENTS[name]) { if (aborted) break; if (typeof k === 'object') { if (!(await ev(k.ok))) { aborted = true; console.log(name, 'ABORTED: focus not where expected'); } continue; } if (typeof k === 'number') { seen.add(await ev(`[...document.querySelectorAll('.panel--cc,.cc__list--shown,.guide,.fullguide,.settings,[class*=guide],[class*=settings]')].map(e=>e.className.split(' ')[0]).filter((v,i,a)=>a.indexOf(v)===i).join('+')`)); await sleep(k); } else { await key(k); await sleep(GAP); } }
	console.log(name, 'saw:', [...seen].join(' | '));
	await sleep(500);
	await mark(`end:${name}`);
}
const data = JSON.parse(await ev(`JSON.stringify({f: window.__stopF(), marks: window.__marks})`));
const fin = new Promise((ok) => { done = ok; });
await send('Tracing.end');
await fin;
relaunch();
writeFileSync(process.argv[2], JSON.stringify({...data, events}));
console.log('events', events.length);
ws.close();
