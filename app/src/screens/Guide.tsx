import {t as tr} from '../lib/i18n';
import {ScreenBackdrop} from '../components/LowerVeil';
import {Rail, type RailItem} from '../components/Rail';
import {useCallback, useEffect, useRef, useState} from 'preact/hooks';
import {useKeys, clamp, type NavKey} from '../hooks/useKeys';
import {useOkPress} from '../hooks/useOkPress';
import {channelColor, clockText, findChannel, nowNextAt, progress, timeLeft, type Channel, type Listings} from '../lib/livetv';
import {FullGuide} from './FullGuide';
import {LivePreview} from '../components/LivePreview';
import {currentChannel} from '../lib/root';
import {isWebOS} from '../lib/luna';
import {rowWindow} from '../lib/windowedList';
import type {LiveTv} from '../hooks/useLiveTv';

interface Props {
	channels: Channel[];
	listings: Listings;
	logos: Record<string, string>;
	/** the current minute, ms */
	now: number;
	/** the Full guide's listings (useLiveTv) */
	schedule: LiveTv['schedule'];
	/** channels the guide service has still to answer for */
	pending: LiveTv['pending'];
	/** showing the Full guide rather than Now and Next */
	full: boolean;
	/** dev builds: a live picture of the channel in focus (components/LivePreview.tsx) */
	preview: boolean;
	/** the preview plays its sound */
	sound: boolean;
	onMode: (full: boolean) => void;
	active: boolean;
	onOpen: (channel: Channel) => void;
	onClose: () => void;
}

/** a row's height and the space under it (keep in step with .guide__row in theme/guide.css) */
const ROW_STEP = 132;
/** rows in view at once */
const IN_VIEW = 6;
/** the focused row stays this many rows from the top while the list scrolls; lower with the live preview, which floats over the last rows */
const FOCUS_AT = 2;
const FOCUS_AT_PREVIEW = 3;
/** rows drawn beyond the view either side, so a scroll never shows a gap */
const SPARE = 2;
/** typing a channel number: the digits stay on screen this long after the last one, then the next digit starts a new number */
const TYPED_MS = 2000;
const TYPED_MAX = 4;
const RAIL: RailItem[] = [
	{key: 'nownext', label: 'Now and Next', glyph: 'list'},
	{key: 'full', label: 'Full guide', glyph: 'grid'}
];

/** a remote's number key (0-9, either row of a keyboard), or -1 */
const digitOf = (e: KeyboardEvent) => (e.keyCode >= 48 && e.keyCode <= 57 ? e.keyCode - 48 : e.keyCode >= 96 && e.keyCode <= 105 ? e.keyCode - 96 : -1);

/** The Guide. Now and Next: one row per channel (number, logo, now with its times and progress, next with its start time); the Full guide
 *  is the time grid (screens/FullGuide.tsx). The rail at the left edge picks between them: Left opens it (in the Full guide,
 *  Left past the earliest programme), Up/Down and OK pick, Right goes back to the channels. The choice is kept for next time. Up/Down move, OK opens Live TV on the
 *  channel, Back returns home. The number keys type a channel number and the list follows each digit (1 goes to 1, then 0 to 10, then 1 to
 *  101); the digits show at the top and clear a moment after the last one. Only the rows near the view are drawn. */
export function Guide ({channels, listings, logos, now, schedule, pending, full, preview, sound, onMode, active, onOpen, onClose}: Props) {
	const [focus, setFocusNow] = useState(0);
	// the Guide opens on Live TV's channel, once the TV says which it is, unless a key has moved the focus first
	const moved = useRef(false);
	const setFocus = useCallback((f: number | ((f: number) => number)) => { moved.current = true; setFocusNow(f); }, []);
	const [tvChannel, setTvChannel] = useState('');
	useEffect(() => { if (isWebOS()) currentChannel().then(setTvChannel, () => undefined); }, []);
	useEffect(() => {
		if (!tvChannel || moved.current) return;
		const i = channels.findIndex((c) => c.id === tvChannel);
		if (i >= 0) { moved.current = true; setFocusNow(i); }
	}, [tvChannel, channels]);
	const [onRail, setOnRail] = useState(false);
	const [railAt, setRailAt] = useState(0);
	const openRail = useCallback(() => { setRailAt(full ? 1 : 0); setOnRail(true); }, [full]);
	const last = channels.length - 1;
	const at = clamp(focus, 0, Math.max(0, last));
	const gridKeys = active && full && !onRail;
	// the Full guide reads the channels near the focus first
	useEffect(() => { schedule.focus(at); }, [at]);

	const handleKey = useCallback((key: NavKey): boolean => {
		if (key === 'back') onClose();
		else if (onRail) {
			if (key === 'up' || key === 'down') setRailAt((r) => clamp(r + (key === 'up' ? -1 : 1), 0, RAIL.length - 1));
			else if (key === 'right') setOnRail(false);
		}
		else if (key === 'left') openRail();
		else if (key === 'up') setFocus(clamp(at - 1, 0, Math.max(0, last)));
		else if (key === 'down') setFocus((f) => clamp(f + 1, 0, Math.max(0, last)));
		return true;
	}, [at, last, onRail, openRail, onClose]);
	useKeys(handleKey, active && !gridKeys);
	useOkPress({onPress: () => { if (onRail) { onMode(railAt === 1); setOnRail(false); } else if (channels[at]) onOpen(channels[at]); }, onLongPress: () => undefined}, active && !gridKeys);

	const [typed, setTyped] = useState('');
	const typedRef = useRef('');
	const list = useRef(channels);
	list.current = channels;
	useEffect(() => {
		if (!active) return undefined;
		let timer = 0;
		const onKey = (e: KeyboardEvent) => {
			const d = digitOf(e);
			if (d < 0) return;
			e.preventDefault();
			const next = (typedRef.current.length >= TYPED_MAX ? '' : typedRef.current) + d;
			typedRef.current = next;
			setTyped(next);
			const i = findChannel(list.current, next);
			if (i >= 0) setFocus(i);
			window.clearTimeout(timer);
			timer = window.setTimeout(() => { typedRef.current = ''; setTyped(''); }, TYPED_MS);
		};
		window.addEventListener('keydown', onKey);
		return () => { window.clearTimeout(timer); window.removeEventListener('keydown', onKey); };
	}, [active]);
	const typedHit = typed !== '' && channels.some((c) => c.number === typed);

	// with the preview the list scrolls on past its end, so even the last channels stop on the fourth row, clear of the box
	const {top, from, to} = rowWindow(preview ? channels.length + IN_VIEW - FOCUS_AT_PREVIEW - 1 : channels.length, at, IN_VIEW, preview ? FOCUS_AT_PREVIEW : FOCUS_AT, SPARE);
	const t = Date.now();
	const focused = channels[at];
	const focusedLogo = focused && focused.logoPath ? logos[focused.logoPath] : undefined;

	return (
		<div class={`screen screen--settings screen--guide${full ? ' screen--guide-full' : ''}${preview ? ' screen--guide-preview' : ''}`}>
			<ScreenBackdrop />
			<div class="guide__head">
				<h1 class="guide__title">{tr('Guide')}</h1>
				{typed && <div class={`guide__typed${typedHit ? '' : ' guide__typed--miss'}`}>{typed}</div>}
			</div>
			{preview && channels.length > 0 && <div class="guide__side">
				<LivePreview channel={focused} logo={focusedLogo} active={active} sound={sound} />
			</div>}
			{full && <FullGuide channels={channels} listings={schedule.listings} from={schedule.from} thin={schedule.state === 'pf'} loading={schedule.state === 'loading'} logos={logos} now={now} at={at} onAt={setFocus} active={gridKeys} onOpen={onOpen} onLeft={openRail} railOpen={onRail} onClose={onClose} />}
			{!full && <>
			<div class="guide__cols">
				<span class="guide__col guide__col--now">{tr('Now')} · {clockText(now)}</span>
				<span class="guide__col guide__col--next">{tr('Next')}</span>
			</div>
			<div class="guide__view">
				{channels.length === 0 && <div class="guide__empty">{tr('No channels yet. Open Live TV once, then come back.')}</div>}
				<div class="guide__track" style={{transform: `translateY(${-top * ROW_STEP}px)`}}>
					{channels.slice(from, to).map((c, k) => {
						const i = from + k;
						const {now: on, next} = nowNextAt(listings[c.signal], t);
						const logo = c.logoPath && logos[c.logoPath];
						const color = channelColor(c.name);
						// still waiting on the guide service: placeholder blocks, and the text fades in over them when it comes
						const waiting = !on && !next && !!pending[c.signal];
						return (
							<div
								key={c.id}
								class={`guide__row${i === at && !onRail ? ' guide__row--focused' : ''}${on ? '' : ' guide__row--off'}`}
								style={{top: `${i * ROW_STEP}px`}}
								onMouseEnter={() => setFocus(i)}
								onClick={() => onOpen(c)}
							>
								<div class="guide__num">{c.number}</div>
								<div class="guide__logo" style={color ? {backgroundColor: color} : undefined}>{logo ? <img src={logo} alt="" decoding="async" /> : <span>{c.name}</span>}</div>
								{waiting
									? <div class="guide__now" key="ph"><i class="ph guide__ph-title" /><div class="guide__meta"><i class="ph guide__ph-time" /><i class="ph guide__ph-meter" /></div></div>
									: <div class="guide__now reveal" key="on">
										<div class="guide__prog">{on ? on.title : tr(next ? 'Off air' : 'No guide data')}</div>
										<div class="guide__meta">
											{on
												? <><span>{clockText(on.start)} – {clockText(on.end)}</span><span class="guide__meter"><span style={{transform: `scaleX(${progress(on, t)})`}} /></span>{i === at && <span>{timeLeft(on, t)}</span>}</>
												: next && <span>{tr('Starts at')} {clockText(next.start)}</span>}
										</div>
									</div>}
								{waiting
									? <div class="guide__next" key="ph"><i class="ph guide__ph-next" /><div class="guide__meta"><i class="ph guide__ph-start" /></div></div>
									: <div class="guide__next reveal" key="on">
										{next && <><div class="guide__prog guide__prog--next">{next.title}</div><div class="guide__meta"><span>{clockText(next.start)}</span></div></>}
									</div>}
							</div>
						);
					})}
				</div>
			</div>
			</>}
			<Rail items={RAIL} on={full ? 1 : 0} at={railAt} open={onRail} />
		</div>
	);
}
