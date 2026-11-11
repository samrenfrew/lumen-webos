// Talks to the live Lumen page on the TV over the DevTools tunnel (see .claude/skills/tv/SKILL.md). Node 22, no packages.
//   node tools/tv/cdp.mjs eval '<js expression>'      prints the value (promises awaited; wrap objects in JSON.stringify)
//   node tools/tv/cdp.mjs css '<css>'                 replaces the live <style id="live"> block; css '' clears it
//   node tools/tv/cdp.mjs shot <out.png> [x,y,w,h]    screenshot of the page, or of a clip
//   node tools/tv/cdp.mjs key down down enter 500 back   presses keys in order; a number waits that many ms
// The tunnel must be open: ssh -o BatchMode=yes -o ExitOnForwardFailure=yes -f -N -L 19998:127.0.0.1:9998 root@192.168.0.20
import {writeFileSync} from 'node:fs';

const PORT = process.env.LUMEN_CDP_PORT || 19998;
const KEYS = {left: [37, 'ArrowLeft'], up: [38, 'ArrowUp'], right: [39, 'ArrowRight'], down: [40, 'ArrowDown'], enter: [13, 'Enter'], back: [461, 'GoBack'], esc: [27, 'Escape']};

const pages = await fetch(`http://127.0.0.1:${PORT}/json`).then((r) => r.json()).catch(() => {
	console.error(`No DevTools on 127.0.0.1:${PORT}. Open the tunnel first (see the top of this file).`);
	process.exit(2);
});
// the page id changes on every relaunch, so it is looked up each time
// Lumen's own page by default; LUMEN_CDP_PAGE=guide or quick picks an overlay app's page (guide.html, quick.html)
const want = process.env.LUMEN_CDP_PAGE ? new RegExp(`lumen.*/${process.env.LUMEN_CDP_PAGE}\\.html`, 'i') : /lumen[^/]*\/index\.html/i;
const page = pages.find((p) => want.test(p.url)) || (process.env.LUMEN_CDP_PAGE ? undefined : pages.find((p) => p.title === 'Launcher'));
if (!page) { console.error('Lumen is not running on the TV:', pages.map((p) => p.url).join(', ')); process.exit(2); }

const ws = new WebSocket(page.webSocketDebuggerUrl);
await new Promise((ok, fail) => { ws.onopen = ok; ws.onerror = fail; });
let seq = 0;
const waiting = new Map();
ws.onmessage = (m) => { const msg = JSON.parse(m.data); if (waiting.has(msg.id)) { waiting.get(msg.id)(msg); waiting.delete(msg.id); } };
const send = (method, params = {}) => new Promise((ok) => { const id = ++seq; waiting.set(id, ok); ws.send(JSON.stringify({id, method, params})); });

const evaluate = async (expression) => {
	const r = await send('Runtime.evaluate', {expression, returnByValue: true, awaitPromise: true});
	if (r.result && r.result.exceptionDetails) throw new Error(r.result.exceptionDetails.exception?.description || r.result.exceptionDetails.text);
	return r.result && r.result.result ? r.result.result.value : r.error;
};

const [cmd, ...args] = process.argv.slice(2);
try {
	if (cmd === 'eval') console.log(await evaluate(args.join(' ')));
	else if (cmd === 'css') {
		await evaluate(`(() => { let s = document.getElementById('live'); if (!s) { s = document.createElement('style'); s.id = 'live'; document.head.appendChild(s); } s.textContent = ${JSON.stringify(args.join(' '))}; return 'ok'; })()`);
		console.log(args.join(' ') ? 'live css set' : 'live css cleared');
	} else if (cmd === 'shot') {
		const params = {format: 'png'};
		if (args[1]) { const [x, y, width, height] = args[1].split(',').map(Number); params.clip = {x, y, width, height, scale: 1}; }
		const r = await send('Page.captureScreenshot', params);
		writeFileSync(args[0] || 'tv.png', Buffer.from(r.result.data, 'base64'));
		console.log(`saved ${args[0] || 'tv.png'}`);
	} else if (cmd === 'key') {
		for (const k of args) {
			if (/^\d+$/.test(k)) { await new Promise((ok) => setTimeout(ok, Number(k))); continue; }
			const [code, key] = KEYS[k] || [];
			if (!code) throw new Error(`unknown key ${k}: use ${Object.keys(KEYS).join(' ')}`);
			for (const type of ['rawKeyDown', 'keyUp']) await send('Input.dispatchKeyEvent', {type, windowsVirtualKeyCode: code, nativeVirtualKeyCode: code, key, code: key});
			await new Promise((ok) => setTimeout(ok, 250));
		}
		console.log('keys sent');
	} else {
		console.error('usage: cdp.mjs eval|css|shot|key ... (see the top of this file)');
		process.exitCode = 1;
	}
} catch (e) {
	console.error(e.message);
	process.exitCode = 1;
}
ws.close();
