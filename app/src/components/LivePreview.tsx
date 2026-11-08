import {t as tr} from '../lib/i18n';
import {useEffect, useRef, useState} from 'preact/hooks';
import {isWebOS, lunaCall} from '../lib/luna';
import {channelColor, type Channel} from '../lib/livetv';

/** Read at each tune, and on dev builds open to DevTools as `lumen.preview` so the waits can be tried live. `last` holds the newest tune's times. */
export const previewTiming = {
	/** wait this long on a channel before tuning it: each tune makes the TV store guide data and slows the page for a while, so normal scrolling must not tune (docs/research/frame-rate.md) */
	dwell: 600,
	/** the Guide's first preview waits this long after the screen has drawn, so tuning does not slow its opening */
	first: 800,
	/** after `playing`, show the picture anyway if the TV has not unmuted it by then */
	settle: 4000,
	/** how often to ask the TV whether the picture is still muted */
	poll: 60,
	last: {tuned: 0, frames: 0},
};
if (__LUMEN_DEV__) { const w = window as unknown as {lumen?: Record<string, unknown>}; w.lumen = w.lumen || {}; w.lumen.preview = previewTiming; }
/** no picture by then: the channel is off air, or the tuner is busy */
const PREVIEW_START_MS = 4000;

/** Whether the TV shows a picture: it mutes the video while it tunes, about 2.5 s past `playing` on this TV, and the page sees no
 *  other sign (its time and frame counts stay at 0). An empty list, between tunes, is not yet a picture. */
async function pictureShown (): Promise<boolean> {
	type Mutes = {videoMute?: {videoMuteStatus: boolean}[]};
	const r = await lunaCall<Mutes & {returnValue?: boolean}>('luna://com.webos.service.avmutemanager/listMuteStatus', {}, 1000, true).catch(() => null);
	const list = r && r.videoMute;
	return !!list && list.length > 0 && list.every((m) => !m.videoMuteStatus);
}

/** tuned: the tuner says `playing`, but the TV still mutes the picture, so the plate stays up */
type State = 'wait' | 'tuned' | 'playing' | 'none';

/** Lets the tuner go: an emptied, reloaded video gives it up; a hidden or paused one keeps it. */
function release (v: HTMLVideoElement | null): void {
	if (!v) return;
	v.onplaying = v.onerror = null;
	try { v.pause(); } catch (e) { /* not playing */ }
	while (v.firstChild) v.removeChild(v.firstChild);
	if (v.getAttribute('src') !== null) v.removeAttribute('src');
	try { v.load(); } catch (e) { /* nothing loaded */ }
}

interface Props {
	channel: Channel | undefined;
	logo?: string;
	/** false while the Guide is covered or closing: the tuner is let go */
	active: boolean;
	/** play the channel's sound */
	sound?: boolean;
	/** fills a Home channel tile: no Live badge or "No picture" line */
	fill?: boolean;
}

/** false while another app is in front: Lumen's page stays loaded and would keep the tuner */
function usePageShown (): boolean {
	const [shown, setShown] = useState(!document.hidden);
	useEffect(() => {
		const on = () => setShown(!document.hidden);
		document.addEventListener('visibilitychange', on);
		return () => document.removeEventListener('visibilitychange', on);
	}, []);
	return shown;
}

/** A live picture of `channel`, silent unless `sound`, after it has had focus for `previewTiming.dwell`. On the TV a muted video plays
 *  `service/webos-broadcast-preview`, which tunes without changing Live TV's last channel (docs/research/live-tv.md, from Glasshouse).
 *  The logo plate sits over the video until the TV unmutes the picture (`pictureShown`), since the hole the video cuts in
 *  the page shows black until then. Off the TV the wallpaper stands in for the picture. */
export function LivePreview ({channel, logo, active, sound = false, fill = false}: Props) {
	const [state, setState] = useState<State>('wait');
	const video = useRef<HTMLVideoElement>(null);
	const id = channel ? channel.id : '';
	const shown = usePageShown();
	// in the Guide, nothing tunes until two frames have drawn and `previewTiming.first` has passed; a Home tile mounts on focus, after Home has drawn
	const [drawn, setDrawn] = useState(fill);
	useEffect(() => {
		if (drawn) return undefined;
		let timer = 0;
		let frame = requestAnimationFrame(() => { frame = requestAnimationFrame(() => { timer = window.setTimeout(() => setDrawn(true), previewTiming.first); }); });
		return () => { cancelAnimationFrame(frame); window.clearTimeout(timer); };
	}, []);
	const on = active && shown && drawn;

	useEffect(() => {
		setState('wait');
		if (!id || !on) return undefined;
		let gaveUp = 0;
		let poll = 0;
		let stopped = false;
		const dwell = window.setTimeout(() => {
			const v = video.current;
			if (!isWebOS() || !v) { setState('playing'); return; }
			v.muted = !sound;
			v.defaultMuted = !sound;
			const s = document.createElement('source');
			s.setAttribute('type', 'service/webos-broadcast-preview');
			s.setAttribute('src', `tv://${id}`);
			v.appendChild(s);
			const t0 = performance.now();
			// the hole stays black until the TV unmutes the picture, so the plate holds until then
			v.onplaying = () => {
				const tuned = performance.now();
				previewTiming.last = {tuned: Math.round(tuned - t0), frames: 0};
				setState('tuned');
				const look = () => {
					pictureShown().then((yes) => {
						if (stopped) return;
						const now = performance.now();
						if (yes || now - tuned >= previewTiming.settle) {
							previewTiming.last = {tuned: previewTiming.last.tuned, frames: Math.round(now - t0)};
							setState('playing');
							return;
						}
						poll = window.setTimeout(look, previewTiming.poll);
					});
				};
				look();
			};
			v.onerror = () => setState('none');
			v.load();
			const played = v.play() as Promise<void> | undefined;
			// a page may not start sound on its own: then play it silent rather than not at all
			if (played && played.then) played.then(undefined, () => { if (!v.muted) { v.muted = true; v.play(); } });
			gaveUp = window.setTimeout(() => setState((st) => st === 'wait' ? 'none' : st), PREVIEW_START_MS);
		}, previewTiming.dwell);
		return () => { stopped = true; window.clearTimeout(dwell); window.clearTimeout(gaveUp); window.clearTimeout(poll); release(video.current); };
	}, [id, on, sound]);

	// the spinner stays mounted through its fade, then goes, so a hidden spinner does not keep the TV drawing frames
	const loading = !!id && (state === 'wait' || state === 'tuned');
	const [spin, setSpin] = useState(loading);
	useEffect(() => {
		if (loading) { setSpin(true); return undefined; }
		const t = window.setTimeout(() => setSpin(false), 400);
		return () => window.clearTimeout(t);
	}, [loading]);

	const color = channel ? channelColor(channel.name) : undefined;
	return (
		<div class={`lpv lpv--${state}${fill ? ' lpv--fill' : ''}`}>
			{isWebOS() ? <video ref={video} class="lpv__picture" muted={!sound} /> : <div class="lpv__picture lpv__picture--mock" />}
			<div class="lpv__plate" style={color ? {backgroundColor: color} : undefined}>
				{logo ? <img src={logo} alt="" decoding="async" /> : <span>{channel ? channel.name : ''}</span>}
				{state === 'none' && !fill && <div class="lpv__none">{tr('No picture')}</div>}
			</div>
			{!fill && <span class="lpv__live">{tr('Live')}</span>}
			{spin && <span class={`lpv__spin${loading ? '' : ' lpv__spin--done'}`} />}
		</div>
	);
}
