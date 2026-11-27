import {connect} from 'node:net';
import {writeFileSync} from 'node:fs';

const USAGE = `node tools/tv/nctl.mjs <command>
  where | stats | get | set <name> <value> | reset | key <left|right|up|down|ok|back>... | call <uri> <json> | quit
  shot <out.png>
Talks to Lumen Native's control channel. Open the tunnel first:
  ssh -o BatchMode=yes -o ExitOnForwardFailure=yes -f -N -L 19997:127.0.0.1:9997 root@192.168.0.20
Lumen Overlay (and the other side apps) listen on 9996: tunnel 19996 to it and set LUMEN_NCTL_PORT=19996.`;

const PORT = Number(process.env.LUMEN_NCTL_PORT || 19997);
const [verb, ...rest] = process.argv.slice(2);
if (!verb) { console.error(USAGE); process.exit(1); }
const savePngTo = verb === 'shot' ? rest[0] || 'native.png' : undefined;
const line = verb === 'shot' ? 'shot' : [verb, ...rest].join(' ');

const socket = connect(PORT, '127.0.0.1');
socket.on('error', (e) => { console.error(`No control channel on 127.0.0.1:${PORT} (${e.code}).\n${USAGE}`); process.exit(2); });
socket.on('connect', () => socket.write(line + '\n'));

let got = Buffer.alloc(0);
socket.on('data', (chunk) => {
	got = Buffer.concat([got, chunk]);
	const end = got.indexOf(10);
	if (end < 0) return;
	const head = got.subarray(0, end).toString();
	const answer = JSON.parse(head);
	if (savePngTo && typeof answer.png === 'number') {
		const png = got.subarray(end + 1);
		if (png.length < answer.png) return;
		writeFileSync(savePngTo, png.subarray(0, answer.png));
		console.log(`saved ${savePngTo}`);
	} else {
		console.log(JSON.stringify(answer, null, 1));
		if (answer.error) process.exitCode = 1;
	}
	socket.end();
});
