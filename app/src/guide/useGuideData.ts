// Lumen Guide's data: the same root calls as Lumen's guide (lib/livetv.ts over lib/root.ts), cached in this app's own localStorage, so an
// open draws the last answer at once and then asks the TV again.

import {useEffect, useMemo, useRef, useState} from 'preact/hooks';
import {cachedLive, cachedLogos, mergeListings, readChannels, readEventsNear, readListings, readLogos, readSchedule, saveLive, type Channel, type Listings} from '../lib/livetv';
import {SCHEDULE_HOURS} from '../hooks/useLiveTv';
import {logError} from '../lib/log';

const HALF_HOUR = 1800000;
/** the cache keeps this much of the schedule, so a second open stays quick */
const CACHE_HOURS = 4;
const minute = () => { const t = Date.now(); return t - (t % 60000); };

export interface GuideData {
	channels: Channel[];
	/** now and next, with the schedule's programmes added where it has them */
	listings: Listings;
	logos: Record<string, string>;
	now: number;
	/** the Full guide's listings, read when it first opens */
	schedule: {listings: Listings; from: number; state: 'loading' | 'ready' | 'pf'};
	/** the Full guide opened: read every channel's schedule, nearest `focus()` first, until `stopped()` */
	readFull: (focus: () => number, stopped: () => boolean) => void;
	/** the guide is shown again after hiding: ask the TV again */
	refresh: () => void;
}

function forCache (l: Listings, t: number): Listings {
	const out: Listings = {};
	for (const k in l) {
		const left = l[k].filter((p) => p.end > t && p.start < t + CACHE_HOURS * 3600000);
		if (left.length > 0) out[k] = left;
	}
	return out;
}

export function useGuideData (): GuideData {
	const [first] = useState(cachedLive);
	const [channels, setChannels] = useState<Channel[]>(first.channels);
	const [pf, setPf] = useState<Listings>(first.listings);
	const [sch, setSch] = useState<{listings: Listings; state: 'loading' | 'ready' | 'pf'}>({listings: first.schedule || {}, state: 'loading'});
	const [logos, setLogos] = useState<Record<string, string>>(cachedLogos);
	const [now, setNow] = useState(minute);
	const latest = useRef({channels, pf, sch: sch.listings});
	latest.current = {channels, pf, sch: sch.listings};
	const fullAsked = useRef(false);

	const save = () => { const {channels: c, pf: l, sch: s} = latest.current; saveLive({channels: c, listings: l, schedule: forCache(s, Date.now())}); };

	const busy = useRef(false);
	const refresh = () => {
		if (busy.current) return;
		busy.current = true;
		fullAsked.current = false;
		setNow(minute());
		readChannels()
				.then((list) => {
					setChannels(list);
					latest.current.channels = list;
					return readListings().then((l) => { setPf(l); latest.current.pf = l; save(); return list; });
				})
				.then((list) => readLogos(list).then((l) => setLogos((was) => (Object.keys(l).length === Object.keys(was).length ? was : {...l}))))
				.catch((e: Error) => logError('lguide', `now and next failed: ${e.message}`))
				.then(() => { busy.current = false; });
	};

	useEffect(() => {
		// after the first frame, so the cached rows are on screen before the root calls start
		const id = window.setTimeout(refresh, 50);
		const tick = window.setInterval(() => setNow(minute()), 15000);
		return () => { window.clearTimeout(id); window.clearInterval(tick); };
	}, []);

	const readFull = (focus: () => number, stopped: () => boolean) => {
		if (fullAsked.current || latest.current.channels.length === 0) return;
		fullAsked.current = true;
		// stops at the next batch once the guide hides, so a hidden guide does not grow while the TV is short of memory
		const t = Date.now();
		const from = t - (t % HALF_HOUR);
		const to = from + SCHEDULE_HOURS * 3600000;
		const got = (l: Listings) => setSch((s) => ({...s, listings: {...s.listings, ...l}}));
		readEventsNear(latest.current.channels, from, to, focus, (l) => got(l), stopped)
			.then(() => 'ready' as const, () => readSchedule(from, to, got).then(() => 'ready' as const, () => 'pf' as const))
			.then((state) => { setSch((s) => ({...s, state})); window.setTimeout(save, 0); });
	};

	const listings = useMemo(() => mergeListings(sch.listings, pf), [sch.listings, pf]);
	return {channels, listings, logos, now, schedule: {listings, from: now - (now % HALF_HOUR), state: sch.state}, readFull, refresh};
}
