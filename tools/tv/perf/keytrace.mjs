// What the main thread does after each key: a devtools.timeline trace, summed by event name. Needs the tunnel on 127.0.0.1:19998.
//   node tools/tv/perf/keytrace.mjs down up        (keys as in keyfps.mjs)
// Prints, per key, the events that took the most main-thread time in the 600 ms after it, and the slowest single ones.
const PORT = process.env.LUMEN_CDP_PORT || 19998;
const KEYS = {left: [37, 'ArrowLeft'], up: [38, 'ArrowUp'], right: [39, 'ArrowRight'], down: [40, 'ArrowDown'], enter: [13, 'Enter'], back: [461, 'GoBack']};
const keys = process.argv.slice(2);
const pages = await fetch(`http://127.0.0.1:${PORT}/json`).then((r) => r.json());
const page = pages.find((p) => /lumen/i.test(p.url));
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
const wait = (ms) => new Promise((ok) => setTimeout(ok, ms));

await send('Tracing.start', {categories: 'devtools.timeline,disabled-by-default-devtools.timeline,v8.execute,blink.user_timing', transferMode: 'ReportEvents'});
await wait(300);
for (const k of keys) {
	const [code, key] = KEYS[k];
	await send('Runtime.evaluate', {expression: `performance.mark('key-${k}')`});
	for (const type of ['rawKeyDown', 'keyUp']) await send('Input.dispatchKeyEvent', {type, windowsVirtualKeyCode: code, nativeVirtualKeyCode: code, key, code: key});
	await wait(900);
}
const finished = new Promise((ok) => { done = ok; });
await send('Tracing.end');
await finished;

const main = events.filter((e) => e.ph === 'X' && e.dur);
const marks = events.filter((e) => e.name && e.name.indexOf('key-') === 0 && (e.ph === 'R' || e.ph === 'I' || e.ph === 'n')).sort((a, b) => a.ts - b.ts);
const SHOW = ['FunctionCall', 'EvaluateScript', 'TimerFire', 'EventDispatch', 'FireAnimationFrame', 'UpdateLayoutTree', 'Layout', 'Paint', 'PrePaint', 'UpdateLayerTree', 'CompositeLayers', 'Decode Image', 'ImageDecodeTask', 'RasterTask', 'ParseHTML', 'MajorGC', 'MinorGC', 'V8.GCScavenger', 'V8.GCFinalizeMC', 'HitTest', 'ScheduleStyleRecalculation', 'v8.run', 'GPUTask'];
marks.forEach((m, i) => {
	const t0 = m.ts, t1 = t0 + 600e3;
	const inWin = main.filter((e) => e.ts >= t0 && e.ts < t1 && SHOW.indexOf(e.name) !== -1);
	const sum = {};
	for (const e of inWin) sum[e.name] = (sum[e.name] || 0) + e.dur / 1000;
	const top = Object.entries(sum).sort((a, b) => b[1] - a[1]).slice(0, 7).map(([n, ms]) => `${n} ${ms.toFixed(0)}ms`);
	const slow = inWin.sort((a, b) => b.dur - a.dur).slice(0, 4).map((e) => `${e.name}${e.args && e.args.data && e.args.data.functionName ? ':' + e.args.data.functionName : ''}${e.args && e.args.data && e.args.data.url ? ' ' + String(e.args.data.url).slice(-40) : ''} ${(e.dur / 1000).toFixed(0)}ms @${((e.ts - t0) / 1000).toFixed(0)}`);
	console.log(`\n${m.name}  totals: ${top.join(', ')}\n  slowest: ${slow.join(' | ')}`);
});
ws.close();
