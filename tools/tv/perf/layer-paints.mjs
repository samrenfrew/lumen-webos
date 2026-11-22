// Which layers repaint on Down and on Up (DevTools layer paint events). usage: node layer-paints.mjs [pairs] ; env LIVECSS, LIVEJS, NORELAUNCH
import {execFileSync} from 'node:child_process';
const PAIRS = +(process.argv[2] || 2);
const relaunch = () => execFileSync('ssh', ['-tt', '-o', 'BatchMode=yes', '-o', 'LogLevel=ERROR', 'root@192.168.0.20', `luna-send -n 1 luna://com.webos.applicationManager/launch '{"id":"com.samrenfrew.lumen"}'`]);
if (!process.env.NORELAUNCH) { relaunch(); await new Promise((ok) => setTimeout(ok, 5000)); }
const pages = await fetch('http://127.0.0.1:19998/json').then((r) => r.json());
const page = pages.find((p) => /lumen/i.test(p.url));
const ws = new WebSocket(page.webSocketDebuggerUrl);
await new Promise((ok) => { ws.onopen = ok; });
let seq = 0; const waiting = new Map(); let layers = new Map(); const painted = []; let phase = 'idle';
ws.onmessage = (m) => {
	const msg = JSON.parse(m.data);
	if (msg.method === 'LayerTree.layerTreeDidChange' && msg.params.layers) for (const l of msg.params.layers) layers.set(l.layerId, l);
	else if (msg.method === 'LayerTree.layerPainted') painted.push({phase, id: msg.params.layerId, clip: msg.params.clip});
	else if (waiting.has(msg.id)) { waiting.get(msg.id)(msg); waiting.delete(msg.id); }
};
const send = (method, params = {}) => new Promise((ok) => { const id = ++seq; waiting.set(id, ok); ws.send(JSON.stringify({id, method, params})); });
const ev = async (expression) => (await send('Runtime.evaluate', {expression, returnByValue: true, awaitPromise: true})).result?.result?.value;
const sleep = (ms) => new Promise((ok) => setTimeout(ok, ms));
const KEYS = {up: [38, 'ArrowUp'], down: [40, 'ArrowDown']};
const key = async (k) => { const [code, name] = KEYS[k]; for (const type of ['rawKeyDown', 'keyUp']) await send('Input.dispatchKeyEvent', {type, windowsVirtualKeyCode: code, nativeVirtualKeyCode: code, key: name, code: name}); };
if (process.env.LIVECSS) await ev(`(() => { let s = document.getElementById('live'); if (!s) { s = document.createElement('style'); s.id = 'live'; document.head.appendChild(s); } s.textContent = ${JSON.stringify(process.env.LIVECSS)}; return 1; })()`);
if (process.env.LIVEJS) await ev(process.env.LIVEJS);
await key('up'); await sleep(1000);
for (let n = 0; n < 3 && !(await ev(`!!document.querySelector('.tile--focused') && !document.querySelector('.screen--lower')`)); n++) { await key(await ev(`!!document.querySelector('.screen--lower')`) ? 'up' : 'down'); await sleep(1000); }
await send('DOM.getDocument', {depth: -1});
await send('LayerTree.enable');
await sleep(1500);
phase = 'rest';
await sleep(1000);
for (let i = 0; i < PAIRS; i++) for (const d of ['down', 'up']) { phase = d; await key(d); await sleep(1300); }
phase = 'done';
// describe each layer by its DOM node
const desc = new Map();
for (const [id, l] of layers) {
	let s = l.backendNodeId ? '' : '(no node)';
	if (l.backendNodeId) {
		const r = await send('DOM.describeNode', {backendNodeId: l.backendNodeId});
		const n = r.result?.node; if (n) { const a = n.attributes || []; const cls = a[a.indexOf('class') + 1]; s = `${n.localName}${a.includes('id') ? '#' + a[a.indexOf('id') + 1] : ''}${a.includes('class') ? '.' + cls.split(' ').slice(0, 3).join('.') : ''}`; if (n.localName === '' || n.nodeName.startsWith('::')) s = n.nodeName; }
		if (n && n.pseudoType) s = `::${n.pseudoType}`;
	}
	desc.set(id, `${s} ${Math.round(l.width)}x${Math.round(l.height)}`);
}
const agg = {};
for (const p of painted) {
	if (p.phase === 'idle' || p.phase === 'done') continue;
	const k = `${p.phase}|${desc.get(p.id) || 'layer ' + p.id}`;
	agg[k] = agg[k] || {n: 0, px: 0};
	agg[k].n++; agg[k].px += p.clip.width * p.clip.height;
}
const out = Object.entries(agg).sort((a, b) => b[1].px - a[1].px);
for (const ph of ['rest', 'down', 'up']) {
	console.log(`\n${ph}: (paints per ${ph === 'rest' ? '1 s' : PAIRS + ' moves'}, megapixels)`);
	for (const [k, v] of out.filter(([k]) => k.startsWith(ph + '|')).slice(0, 25)) console.log(`  ${String(v.n).padStart(4)} ${(v.px / 1e6).toFixed(2).padStart(7)}  ${k.split('|')[1]}`);
}
console.log('\nlayers:', layers.size);
if (process.env.LIVECSS) await ev(`(document.getElementById('live').textContent = '', 1)`);
ws.close();
