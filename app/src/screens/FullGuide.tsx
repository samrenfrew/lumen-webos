import {t as tr} from '../lib/i18n';
import {useCallback, useState} from 'preact/hooks';
import {useKeys, clamp, type NavKey} from '../hooks/useKeys';
import {useOkPress} from '../hooks/useOkPress';
import {channelColor, clockText, progress, timeLeft, type Channel, type Listings, type Programme} from '../lib/livetv';
import {SCHEDULE_HOURS} from '../hooks/useLiveTv';
import {rowWindow} from '../lib/windowedList';

interface Props {
	channels: Channel[];
	/** the schedule, with Now and Next's listings where it has none */
	listings: Listings;
	/** where the schedule starts (a half hour), ms */
	from: number;
	/** the schedule could not be read, so only now and next are in the grid */
	thin: boolean;
	/** still reading: a row with nothing yet says so rather than "No guide data" */
	loading: boolean;
	logos: Record<string, string>;
	/** the current minute, ms */
	now: number;
	/** the focused channel (the Guide keeps it, so typed numbers and the switch to Now and Next keep it too) */
	at: number;
	onAt: (i: number) => void;
	active: boolean;
	/** OK: the channel, and the programme the focus is on (Lumen Guide tunes only to one that is on now) */
	onOpen: (channel: Channel, programme?: Programme) => void;
	/** Up on the first channel: focus moves to the switcher */
	onTop: () => void;
	onClose: () => void;
}

/** a row's height and the space under it (keep in step with .fg__row in theme/guide.css) */
const ROW_STEP = 80;
const IN_VIEW = 7;
const FOCUS_AT = 3;
const SPARE = 2;
const HALF_HOUR = 1800000;
/** the grid shows two hours across, in half-hour columns */
const SPAN = 4 * HALF_HOUR;
/** the grid's width (keep in step with .fg__line in theme/guide.css): the rows' 1670px less the 372px channel column */
const GRID_W = 1298;
const PX_PER_MS = GRID_W / SPAN;
const halfHour = (ms: number) => ms - (ms % HALF_HOUR);

/** "in 40 min", "in 1 hr 5 min" */
function startsIn (p: Programme, t: number): string {
	const min = Math.max(1, Math.ceil((p.start - t) / 60000));
	return min < 60 ? `in ${min} min` : `in ${Math.floor(min / 60)} hr${min % 60 ? ` ${min % 60} min` : ''}`;
}

/** The programme the focus is on: the one on at `a`, else the first that starts after it within the window. */
function pick (list: Programme[], a: number, winEnd: number): Programme | undefined {
	return list.find((p) => p.start <= a && a < p.end) || list.find((p) => p.start > a && p.start < winEnd);
}

/** The Full guide: two hours across, seven channels down, a line at now. The focused programme's details sit above the grid. Up/Down move
 *  between channels and keep the time the focus is at, so the focus stays on what is on now as it moves; Left/Right move between programmes
 *  and scroll the grid half an hour at a time, as far back as the half hour now is in and as far on as the schedule was read. OK opens Live TV
 *  on the channel. Only the rows near the view are drawn, and only the programmes inside the two hours. */
export function FullGuide ({channels, listings, from, thin, loading, logos, now, at, onAt, active, onOpen, onTop, onClose}: Props) {
	const base = halfHour(now);
	const lastWin = Math.max(base, from + SCHEDULE_HOURS * 3600000 - SPAN);
	const [winState, setWin] = useState(base);
	const [anchorState, setAnchor] = useState(now);
	const win = clamp(winState, base, lastWin);
	const winEnd = win + SPAN;
	const anchor = Math.max(anchorState, now);
	const listOf = (c: Channel | undefined) => (c && listings[c.signal]) || [];
	const focused = pick(listOf(channels[at]), anchor, winEnd);
	const last = channels.length - 1;

	/** Focus a programme: anchor on its start (or now, when it is on), and scroll so it starts inside the window. */
	const focusOn = (p: Programme) => {
		setAnchor(Math.max(p.start, now));
		if (p.start >= winEnd - HALF_HOUR || p.start < win) setWin(clamp(halfHour(p.start) - (p.start >= winEnd - HALF_HOUR ? HALF_HOUR : 0), base, lastWin));
	};

	const handleKey = useCallback((key: NavKey): boolean => {
		if (key === 'back') onClose();
		else if (key === 'up') { if (at === 0) onTop(); else onAt(at - 1); }
		else if (key === 'down') onAt(clamp(at + 1, 0, Math.max(0, last)));
		else if (key === 'right' || key === 'left') {
			const list = listOf(channels[at]);
			const i = focused ? list.indexOf(focused) : -1;
			const p = key === 'right'
				? (i >= 0 ? list[i + 1] : list.find((x) => x.start > anchor))
				: (i >= 0 ? list[i - 1] : list.filter((x) => x.end <= anchor).pop());
			if (p && p.end > now && p.start < lastWin + SPAN) focusOn(p);
			else if (key === 'right' && win < lastWin) { setWin(win + HALF_HOUR); setAnchor(Math.max(anchor, win + 2 * HALF_HOUR)); }
			else if (key === 'left' && win > base) { setWin(win - HALF_HOUR); setAnchor(Math.max(now, win - HALF_HOUR)); }
		}
		return true;
	}, [at, last, focused, anchor, win, base, lastWin, now, channels, listings, onAt, onTop, onClose]);
	useKeys(handleKey, active);
	useOkPress({onPress: () => { if (channels[at]) onOpen(channels[at], focused); }, onLongPress: () => undefined}, active);

	const {top, from: rowFrom, to: rowTo} = rowWindow(channels.length, at, IN_VIEW, FOCUS_AT, SPARE);
	const t = Date.now();
	const ch = channels[at];
	const x = (ms: number) => Math.round((clamp(ms, win, winEnd) - win) * PX_PER_MS);
	const nowX = now >= win && now < winEnd ? x(t) : -1;

	return (
		<>
			<div class="fg__detail">
				{ch && <div class="fg__chan">{ch.number} · {ch.name}</div>}
				<div class="fg__title">{focused ? focused.title : tr(thin ? 'Only now and next in the guide data' : 'No guide data')}</div>
				{focused && (
					<div class="fg__meta">
						<span>{clockText(focused.start)} – {clockText(focused.end)}</span>
						{focused.start <= t
							? <><span class="guide__meter fg__meter"><span style={{transform: `scaleX(${progress(focused, t)})`}} /></span><span>{timeLeft(focused, t)}</span></>
							: <span class="fg__in">{startsIn(focused, t)}</span>}
						{focused.subs && <span class="fg__flag">S</span>}
						{focused.ad && <span class="fg__flag">AD</span>}
					</div>
				)}
				{focused && focused.desc && <div class="fg__desc">{focused.desc}</div>}
			</div>
			<div class="fg__times">
				{[0, 1, 2, 3].map((k) => <span key={k} style={{left: `${k * GRID_W / 4}px`}}>{clockText(win + k * HALF_HOUR)}</span>)}
			</div>
			<div class="fg__view">
				{channels.length === 0 && <div class="guide__empty">{tr('No channels yet. Open Live TV once, then come back.')}</div>}
				<div class="guide__track" style={{transform: `translateY(${-top * ROW_STEP}px)`}}>
					{channels.slice(rowFrom, rowTo).map((c, k) => {
						const i = rowFrom + k;
						const logo = c.logoPath && logos[c.logoPath];
						const color = channelColor(c.name);
						const shown = listOf(c).filter((p) => p.end > win && p.start < winEnd);
						const isAt = i === at;
						return (
							<div key={c.id} class={`fg__row${isAt ? ' fg__row--at' : ''}`} style={{top: `${i * ROW_STEP}px`}} onMouseEnter={() => onAt(i)}>
								<div class="fg__cell-chan">
									<span class="fg__num">{c.number}</span>
									<span class="fg__logo" style={color ? {backgroundColor: color} : undefined}>{logo ? <img src={logo} alt="" decoding="async" /> : <span>{c.name}</span>}</span>
									<span class="fg__name">{c.name}</span>
								</div>
								<div class="fg__line">
									{shown.length === 0 && <div class={`fg__prog fg__prog--empty${isAt ? ' fg__prog--focused' : ''}`} style={{left: '0px', width: `${GRID_W - 8}px`}}><div class="fg__ptitle">{tr(loading ? 'Loading…' : 'No guide data')}</div></div>}
									{shown.map((p) => {
										const left = x(p.start);
										const width = x(p.end) - left - 8;
										const cut = p.start < win;
										return (
											<div
												key={p.start}
												class={`fg__prog${isAt && p === focused ? ' fg__prog--focused' : ''}${p.end <= t ? ' fg__prog--past' : ''}`}
												style={{left: `${left}px`, width: `${Math.max(0, width)}px`}}
												onMouseEnter={() => { onAt(i); focusOn(p); }}
												onClick={() => onOpen(c)}
											>
												{width > 60 && <><div class="fg__ptitle">{cut ? '‹ ' : ''}{p.title}</div><div class="fg__ptime">{clockText(p.start)}</div></>}
											</div>
										);
									})}
								</div>
							</div>
						);
					})}
					{nowX >= 0 && <div class="fg__nowline" style={{top: `${top * ROW_STEP}px`, transform: `translateX(${nowX}px)`}} />}
				</div>
			</div>
		</>
	);
}
