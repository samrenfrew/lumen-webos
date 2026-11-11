import {t as tr} from '../lib/i18n';
import {useCallback, useEffect, useRef, useState} from 'preact/hooks';
import {useKeys, clamp, type NavKey} from '../hooks/useKeys';
import {useOkPress} from '../hooks/useOkPress';
import {channelColor, clockText, nowNextAt, progress, timeLeft, type Channel, type Programme} from '../lib/livetv';
import {FullGuide} from '../screens/FullGuide';
import {Rail, type RailItem} from '../components/Rail';
import {currentChannel, openChannel} from '../lib/root';
import {isWebOS} from '../lib/luna';
import {logError} from '../lib/log';
import {rowWindow} from '../lib/windowedList';
import {setWhere} from '../lib/where';
import type {GuideData} from './useGuideData';

export type View = 'now' | 'next' | 'full';

/** a row's height and the space under it (keep in step with .lguide__row in theme/lguide.css) */
const ROW_STEP = 132;
const IN_VIEW = 7;
/** the focused row is held on the fourth row while the list scrolls */
const FOCUS_AT = 3;
const SPARE = 2;
/** the guide closes itself after this long without a key */
const IDLE_MS = 30000;
/** the rows wait this long at most for the TV to say which channel is on, so they do not jump to it after drawing */
const CHANNEL_WAIT_MS = 800;
/** longer than --dur-reveal, so the shells have faded out before they go */
const SHELLS_GONE_MS = 400;

const RAIL: (RailItem & {key: View})[] = [
	{key: 'now', label: 'Now', glyph: 'list'},
	{key: 'next', label: 'Next', glyph: 'next'},
	{key: 'full', label: 'Full guide', glyph: 'grid'}
];

/** Lumen Guide (docs/plans/guide-overlay.md): a narrow panel at the left of the screen over Live TV, with a rail of three views (Now, Next,
 *  the Full guide). Opens on Now with the channel on screen focused on the fourth row. Up/Down move, Left goes onto the rail, which expands
 *  with names; OK there picks a view, Right goes back. OK on a channel tunes to it and closes. Back closes (from the Full guide it returns to
 *  the panel, unless GUIDE opened it there), as does IDLE_MS without a key; LIST and GUIDE again are handled in main.tsx. Each showing
 *  mounts it afresh on startView: Now for LIST, the Full guide for GUIDE. */
export function GuideApp ({data, startView, onView, onClose: close}: {data: GuideData; startView: View; onView: (v: View) => void; onClose: () => void}) {
	const {channels, listings, logos, now} = data;
	const [view, setView] = useState<View>(startView);
	const [onRail, setOnRail] = useState(false);
	const [railAt, setRailAt] = useState(0);
	const [focus, setFocusNow] = useState(0);
	const moved = useRef(false);
	const setFocus = useCallback((f: number | ((f: number) => number)) => { moved.current = true; setFocusNow(f); }, []);
	const [tvChannel, setTvChannel] = useState('');
	const [ready, setReady] = useState(!isWebOS());
	const last = channels.length - 1;
	const at = clamp(focus, 0, Math.max(0, last));
	const atRef = useRef(at);
	atRef.current = at;

	useEffect(() => {
		if (!isWebOS()) return undefined;
		const id = window.setTimeout(() => setReady(true), CHANNEL_WAIT_MS);
		currentChannel().then((c) => { setTvChannel(c); setReady(true); }, () => setReady(true));
		return () => window.clearTimeout(id);
	}, []);
	useEffect(() => {
		if (!tvChannel || moved.current) return;
		const i = channels.findIndex((c) => c.id === tvChannel);
		if (i >= 0) setFocusNow(i);
	}, [tvChannel, channels]);

	// any key keeps it open for another IDLE_MS
	useEffect(() => {
		let idle = window.setTimeout(close, IDLE_MS);
		const onKey = () => { window.clearTimeout(idle); idle = window.setTimeout(close, IDLE_MS); };
		window.addEventListener('keydown', onKey, true);
		return () => { window.clearTimeout(idle); window.removeEventListener('keydown', onKey, true); };
	}, []);

	// the shells fade out as the real rows fade in over them, then go
	const [shellsGone, setShellsGone] = useState(false);
	useEffect(() => {
		if (!ready) return undefined;
		const id = window.setTimeout(() => setShellsGone(true), SHELLS_GONE_MS);
		return () => window.clearTimeout(id);
	}, [ready]);

	useEffect(() => { setWhere(`lumen guide: ${view}${onRail ? ', rail' : ''}`, false); onView(view); }, [view, onRail]);
	// the guide hides by unmounting this, which stops the Full guide's schedule read
	const alive = useRef(true);
	useEffect(() => () => { alive.current = false; }, []);
	useEffect(() => { if (view === 'full') data.readFull(() => atRef.current, () => !alive.current); }, [view, channels.length]);

	const tune = (c: Channel) => {
		openChannel(c.id).catch((e: Error) => logError('lguide', `openChannel failed: ${e.message}`)).then(close);
	};

	const handleKey = useCallback((key: NavKey): boolean => {
		if (key === 'back') close();
		else if (onRail) {
			if (key === 'up') setRailAt((r) => clamp(r - 1, 0, RAIL.length - 1));
			else if (key === 'down') setRailAt((r) => clamp(r + 1, 0, RAIL.length - 1));
			else if (key === 'right') setOnRail(false);
		}
		else if (key === 'left') { setRailAt(RAIL.findIndex((r) => r.key === view)); setOnRail(true); }
		else if (key === 'up') setFocus(at - 1 < 0 ? 0 : at - 1);
		else if (key === 'down') setFocus((f) => clamp(f + 1, 0, Math.max(0, last)));
		return true;
	}, [at, last, onRail, view]);
	const panelKeys = view !== 'full';
	useKeys(handleKey, panelKeys);
	useOkPress({
		onPress: () => {
			if (onRail) { setView(RAIL[railAt].key); setOnRail(false); }
			else if (channels[at]) tune(channels[at]);
		}
	}, panelKeys);

	const openFromGrid = (c: Channel, p?: Programme) => {
		const t = Date.now();
		if (p && p.start <= t && t < p.end) tune(c);
	};

	if (view === 'full') {
		return (
			<div class="lguide lguide__full">
				<div class="guide__head"><h1 class="guide__title">{tr('Full guide')}</h1></div>
				<FullGuide channels={channels} listings={data.schedule.listings} from={data.schedule.from} thin={data.schedule.state === 'pf'} loading={data.schedule.state === 'loading'} logos={logos} now={now} at={at} onAt={setFocus} active onOpen={openFromGrid} onLeft={() => undefined} onClose={() => (startView === 'full' ? close() : setView('now'))} />
			</div>
		);
	}

	const {top, from, to} = rowWindow(channels.length, at, IN_VIEW, FOCUS_AT, SPARE);
	const t = Date.now();
	const isNext = view === 'next';
	return (
		<div class="lguide">
			<div class="lguide__panel" />
			<div class="lguide__head">{tr(isNext ? 'Next' : 'Now')}{isNext ? '' : ` · ${clockText(now)}`}</div>
			<div class="lguide__view">
				{ready && channels.length === 0 && <div class="guide__empty">{tr('No channels yet. Open Live TV once, then come back.')}</div>}
				{!shellsGone && channels.length > 0 && (
					<div class={`guide__track${ready ? ' lguide__shells--out' : ''}`} key="shells">
						{Array.from({length: IN_VIEW}, (_, k) => (
							<div key={k} class="lguide__row" style={{top: `${k * ROW_STEP}px`}}>
								<div class="guide__num"><i class="lguide__ph lguide__ph--num" /></div>
								<div class="lguide__slot" />
								<div class="lguide__prog">
									<i class="lguide__ph lguide__ph--title" />
									<div class="guide__meta"><i class="lguide__ph lguide__ph--time" /><i class="lguide__ph lguide__ph--meter" /></div>
								</div>
							</div>
						))}
					</div>
				)}
				{ready && <div class="guide__track lguide__rows" key="rows" style={{transform: `translateY(${-top * ROW_STEP}px)`}}>
					{channels.slice(from, to).map((c, k) => {
						const i = from + k;
						const {now: on, next} = nowNextAt(listings[c.signal], t);
						const shown = isNext ? next : on;
						const logo = c.logoPath && logos[c.logoPath];
						const color = channelColor(c.name);
						return (
							<div
								key={c.id}
								class={`lguide__row${i === at && !onRail ? ' lguide__row--focused' : ''}${i === at ? ' lguide__row--at' : ''}${shown ? '' : ' lguide__row--off'}`}
								style={{top: `${i * ROW_STEP}px`}}
							>
								<div class="guide__num">{c.number}</div>
								<div class="lguide__slot"><div class="guide__logo" style={color ? {backgroundColor: color} : undefined}>{logo ? <img src={logo} alt="" decoding="async" /> : <span>{c.name}</span>}</div></div>
								<div class="lguide__prog">
									<div class="guide__prog">{shown ? shown.title : tr(!isNext && next ? 'Off air' : 'No guide data')}</div>
									<div class="guide__meta">
										{isNext
											? shown && <span>{tr('Starts at')} {clockText(shown.start)}</span>
											: on
												? <><span>{clockText(on.start)} – {clockText(on.end)}</span><span class="guide__meter"><span style={{transform: `scaleX(${progress(on, t)})`}} /></span>{i === at && <span>{timeLeft(on, t)}</span>}</>
												: next && <span>{tr('Starts at')} {clockText(next.start)}</span>}
									</div>
								</div>
							</div>
						);
					})}
				</div>}
			</div>
			<Rail items={RAIL} on={RAIL.findIndex((r) => r.key === view)} at={railAt} open={onRail} />
		</div>
	);
}
