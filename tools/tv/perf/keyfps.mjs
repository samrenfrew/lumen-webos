// Frames per key press, for a TV in Developer Mode (no root, no relaunch). Needs the DevTools tunnel on 127.0.0.1:19998.
//   node tools/tv/perf/keyfps.mjs [keys...]       e.g. right right right left left down up   (default: a dock walk)
//   LIVECSS='<css>' applies a style first, for A/B tests; it is removed afterwards.
// For each key: frames painted in the 400 ms after it (60 fps = 24), the worst gap, and main-thread long tasks.
const PORT = process.env.LUMEN_CDP_PORT || 19998;
const KEYS = {left: [37, 'ArrowLeft'], up: [38, 'ArrowUp'], right: [39, 'ArrowRight'], down: [40, 'ArrowDown'], enter: [13, 'Enter'], back: [461, 'GoBack']};
const keys = process.argv.slice(2).length ? process.argv.slice(2) : ['right', 'right', 'right', 'right', 'left', 'left', 'left', 'left'];
const WINDOW = 400;

const pages = await fetch(`http://127.0.0.1:${PORT}/json`).then((r) => r.json());
const page = pages.find((p) => /lumen/i.test(p.url));
if (!page) { console.error('Lumen is not running'); process.exit(2); }
const ws = new WebSocket(page.webSocketDebuggerUrl);
await new Promise((ok, fail) => { ws.onopen = ok; ws.onerror = fail; });
let seq = 0; const waiting = new Map();
ws.onmessage = (m) => { const msg = JSON.parse(m.data); if (waiting.has(msg.id)) { waiting.get(msg.id)(msg); waiting.delete(msg.id); } };
const send = (method, params = {}) => new Promise((ok) => { const id = ++seq; waiting.set(id, ok); ws.send(JSON.stringify({id, method, params})); });
const evaluate = async (expression) => (await send('Runtime.evaluate', {expression, returnByValue: true, awaitPromise: true})).result.result.value;
const wait = (ms) => new Promise((ok) => setTimeout(ok, ms));

await evaluate(`(() => {
	window.__kf = {frames: [], long: []};
	const tick = (t) => { window.__kf.frames.push(t); window.__kf.raf = requestAnimationFrame(tick); };
	window.__kf.raf = requestAnimationFrame(tick);
	try { window.__kf.po = new PerformanceObserver((l) => { for (const e of l.getEntries()) window.__kf.long.push([e.startTime, e.duration]); }); window.__kf.po.observe({entryTypes: ['longtask']}); } catch (e) {}
	let s = document.getElementById('kf-live'); if (!s) { s = document.createElement('style'); s.id = 'kf-live'; document.head.appendChild(s); }
	s.textContent = ${JSON.stringify(process.env.LIVECSS || '')};
	return 1;
})()`);
await wait(600);

const marks = [];
for (const k of keys) {
	const [code, key] = KEYS[k];
	marks.push([k, await evaluate('performance.now()')]);
	for (const type of ['rawKeyDown', 'keyUp']) await send('Input.dispatchKeyEvent', {type, windowsVirtualKeyCode: code, nativeVirtualKeyCode: code, key, code: key});
	await wait(WINDOW + 250);
}
const data = await evaluate(`(() => { cancelAnimationFrame(window.__kf.raf); if (window.__kf.po) window.__kf.po.disconnect(); const s = document.getElementById('kf-live'); if (s) s.remove(); return JSON.stringify(window.__kf); })()`);
const {frames, long} = JSON.parse(data);
const rows = marks.map(([k, t0]) => {
	const f = frames.filter((t) => t >= t0 && t < t0 + WINDOW);
	let worst = 0; for (let i = 1; i < f.length; i++) worst = Math.max(worst, f[i] - f[i - 1]);
	const lt = long.filter(([s]) => s >= t0 - 50 && s < t0 + WINDOW).reduce((a, [, d]) => a + d, 0);
	return {key: k, frames: f.length, worstGapMs: Math.round(worst), longTaskMs: Math.round(lt)};
});
console.table(rows);
const avg = rows.reduce((a, r) => a + r.frames, 0) / rows.length;
console.log(`average ${avg.toFixed(1)} frames per ${WINDOW} ms (${Math.round(avg * 1000 / WINDOW)} fps; 60 fps = ${WINDOW * 60 / 1000})`);
ws.close();
