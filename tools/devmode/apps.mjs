// Apps found on TVs with `sh tools/devmode.sh find-apps` (LG's own, Indian and global streaming), and how each one's tile looks.
// tools/devmode/icons.mjs turns this into the icon pack (app/private/iconpack/) and the catalogue (app/src/config/catalog.local.ts), so the
// two never disagree.
//
// Every tile is drawn the same way: a two-stop gradient in the app's hue (light top-left, deep bottom-right), one soft highlight, and a
// white mark at one optical size. The mark is the brand's glyph where Simple Icons has it, a Lucide line icon for LG's own apps, or a
// set wordmark. `from`/`to` are the gradient stops.
//
//   mark: {si: 'siNetflix'}            a Simple Icons glyph
//         {lucide: 'tv'}               a Lucide icon
//         {word: 'ZEE5', size: 150}    a wordmark in Geist
//         {svg: '<path …/>'}           a custom 24×24 mark (filled white)
//         {logo: 'zee5', …}            the brand's own logo (mode: 'luma' keeps only its light parts, for a logo on a box) from tools/logos/raw/zee5.svg|png (gitignored: other people's logos are never
//                                      committed), turned white; box: [w, h] is the space it fits in. When the file is missing, the
//                                      mark's other fields (word, si, lucide) are drawn instead, so the pack builds without the logos.
//
// `ids` lists other ids the same app is known by. Apps that are not installed cost nothing (the catalogue is filtered by
// getAppLoadStatus), so an app installed later shows up with its tile, in its place on the home rows.

export const APPS = [
	// ----- streaming -----
	{id: 'youtube.leanback.v4', title: 'YouTube', from: '#d8342e', to: '#4a0b09', mark: {si: 'siYoutube', scale: 0.94}},
	{id: 'netflix', title: 'Netflix', from: '#c8121c', to: '#250103', mark: {si: 'siNetflix', scale: 0.9}},
	{id: 'amazon', title: 'Prime Video', from: '#2a93f2', to: '#052a52', mark: {logo: 'primevideo', box: [470, 170], word: 'prime video', size: 96, weight: 640, smile: true}},
	{id: 'hotstar', title: 'JioHotstar', from: '#3b38c9', to: '#0d0b3d', ids: ['com.jio.hotstar', 'jiohotstar', 'com.jiostar.jiohotstar', 'in.startv.hotstar', 'com.hotstar.lg', 'com.disney.hotstar', 'com.hotstar.hotstar'], mark: {logo: 'jiohotstar', box: [480, 170], word: 'JioHotstar', size: 100, weight: 680}},
	{id: 'com.sonyliv2.tvapp', title: 'SonyLIV', from: '#4b4f5c', to: '#0c0d11', ids: ['com.sonyliv.tvapp', 'sonyliv', 'com.sonyliv', 'com.sonyliv.sonyliv'], mark: {logo: 'sonyliv', mode: 'luma', box: [600, 330], word: 'SonyLIV', size: 116, weight: 700}},
	{id: 'com.zee5.app', title: 'ZEE5', from: '#8b35d0', to: '#21063b', ids: ['zee5', 'com.zee5.webos', 'com.graymatrix.did', 'com.zee5.zee5'], mark: {logo: 'zee5', box: [400, 170], word: 'ZEE5', size: 148, weight: 760}},
	{id: 'com.viacom18.media.ondemand', title: 'JioCinema', from: '#d81b80', to: '#3a0322', ids: ['jiocinema', 'com.jio.media.ondemand', 'com.jiocinema.app', 'com.jio.jiocinema', 'com.jiocinema.jiocinema'], mark: {logo: 'jiocinema', box: [480, 170], word: 'JioCinema', size: 104, weight: 680}},
	{id: 'spotify-beehive', title: 'Spotify', from: '#1fae50', to: '#032b17', mark: {si: 'siSpotify'}},
	{id: 'org.jellyfin.webos', title: 'Jellyfin', from: '#8a45c4', to: '#07466b', mark: {si: 'siJellyfin'}},
	{id: 'io.strem.tv', title: 'Stremio', from: '#7550e0', to: '#1a0e4a', mark: {si: 'siStremio'}},
	{id: 'com.viacom18.voot', title: 'Voot', from: '#6232cf', to: '#170a3a', ids: ['voot', 'com.voot.voot'], mark: {logo: 'voot', box: [400, 170], word: 'voot', size: 146, weight: 760}},
	// ----- more Indian OTT and music apps: drawn now, shown once one of their ids is installed -----
	{id: 'com.jio.jiotv', title: 'JioTV', ids: ['jiotv', 'com.jio.media.jiotv'], from: '#d7263d', to: '#3a070f', mark: {logo: 'jiotv', mode: 'luma', box: [320, 240], word: 'JioTV', size: 140, weight: 720}},
	{id: 'amazon.minitv', title: 'miniTV', ids: ['com.amazon.minitv'], from: '#4a3ad6', to: '#120c40', mark: {logo: 'amazonminitv', box: [470, 170], word: 'miniTV', size: 128, weight: 700}},
	{id: 'com.mxplayer.tv', title: 'MX Player', ids: ['mxplayer', 'com.mxtech.mxplayer', 'com.mxplayer.webos', 'com.mxplayer.mxplayer'], from: '#1d6fe0', to: '#061f45', mark: {logo: 'mxplayer', box: [440, 170], word: 'MX Player', size: 110, weight: 720}},
	{id: 'com.airtel.xstream', title: 'Xstream Play', ids: ['airtelxstream', 'tv.accedo.airtel.wynk'], from: '#e0262c', to: '#3b0507', mark: {logo: 'airtelxstream', box: [470, 170], word: 'Xstream', size: 120, weight: 720}},
	{id: 'com.tataplay.binge', title: 'Tata Play Binge', ids: ['tataplaybinge', 'com.tatasky.binge'], from: '#a0249c', to: '#2e0730', mark: {logo: 'tataplaybinge', box: [440, 170], word: 'Binge', size: 140, weight: 740}},
	{id: 'com.altbalaji.app', title: 'ALTT', ids: ['altbalaji', 'com.balaji.alt', 'com.altbalaji.alt'], from: '#e01b3a', to: '#3a050e', mark: {logo: 'altt', mode: 'luma', box: [500, 260], word: 'ALTT', size: 150, weight: 780}},
	{id: 'com.d27n.app.erosnow', title: 'Eros Now', ids: ['erosnow', 'com.erosnow.erosnow'], from: '#c8202e', to: '#33060b', mark: {logo: 'erosnow', box: [440, 170], word: 'Eros Now', size: 112, weight: 700}},
	{id: 'com.hungama.lgtvapp', title: 'Hungama', ids: ['hungamaplay', 'com.hungama.hungama', 'com.hungama.movies'], from: '#ee6a22', to: '#3b1504', mark: {logo: 'hungama', box: [440, 170], word: 'Hungama', size: 118, weight: 720}},
	{id: 'com.sun.sunnxt', title: 'Sun NXT', ids: ['sunnxt', 'com.suntv.sunnxt', 'com.sunnxt.sunnxt'], from: '#eb8f1c', to: '#3b2104', mark: {logo: 'sunnxt', box: [440, 170], word: 'SUN NXT', size: 116, weight: 760}},
	{id: 'aha', title: 'aha', ids: ['com.aha.app', 'com.arha.media', 'com.aha.aha', 'ahaVideo'], from: '#f26a10', to: '#3b1602', mark: {logo: 'aha', box: [380, 170], word: 'aha', size: 170, weight: 780}},
	{id: 'com.hoichoi.app', title: 'hoichoi', ids: ['hoichoi', 'com.viewlift.hoichoi', 'com.hoichoi.hoichoi'], from: '#dd2433', to: '#36060b', mark: {logo: 'hoichoi', box: [440, 170], word: 'hoichoi', size: 128, weight: 720}},
	{id: 'com.shemaroome.app', title: 'ShemarooMe', ids: ['shemaroome', 'com.shemaroo.shemaroome'], from: '#d99a1e', to: '#352204', mark: {word: 'ShemarooMe', size: 100, weight: 700}},
	{id: 'com.lionsgateplay.app', title: 'Lionsgate Play', ids: ['lionsgateplay', 'com.lionsgateplay.lionsgate'], from: '#8a7748', to: '#1e190c', mark: {logo: 'lionsgateplay', box: [460, 170], word: 'LIONSGATE', size: 96, weight: 640}},
	{id: 'com.discovery.dplus.india', title: 'discovery+', ids: ['com.discovery.dplus.emea', 'com.discovery.discoveryplus', 'discoveryplus', 'com.discoveryplus.india', 'com.discoveryplus.app'], from: '#2a7ae0', to: '#06204a', mark: {logo: 'discoveryplus', box: [460, 170], word: 'discovery+', size: 106, weight: 700}},
	{id: 'yupptv', title: 'YuppTV', ids: ['com.yupptv.webos', 'com.yupptv.yupptv'], from: '#e4572e', to: '#391206', mark: {logo: 'yupptv', box: [420, 170], word: 'YuppTV', size: 128, weight: 720}},
	{id: 'com.dangalplay.app', title: 'Dangal Play', ids: ['dangalplay', 'com.dangal.play'], from: '#d23a2a', to: '#350b07', mark: {logo: 'dangalplay', box: [440, 170], word: 'Dangal', size: 128, weight: 760}},
	{id: 'com.chaupal.app', title: 'Chaupal', ids: ['chaupal', 'com.chaupal.chaupal'], from: '#d1562a', to: '#341407', mark: {logo: 'chaupal', box: [440, 170], word: 'Chaupal', size: 124, weight: 720}},
	{id: 'com.timesplay.app', title: 'Times Play', ids: ['timesplay', 'com.timesplay.timesplay'], from: '#b8202a', to: '#2e0609', mark: {logo: 'timesplay', box: [440, 170], word: 'Times Play', size: 108, weight: 700}},
	{id: 'com.fancode.app.streaming', title: 'FanCode', ids: ['fancode', 'com.fancode.fancode'], from: '#ef5320', to: '#3b1205', mark: {logo: 'fancode', box: [440, 170], word: 'FanCode', size: 120, weight: 740}},
	{id: 'com.etvwin.app', title: 'ETV Win', from: '#d63031', to: '#350a0a', mark: {logo: 'etvwin', mode: 'luma', box: [440, 260], word: 'ETV Win', size: 124, weight: 740}},
	{id: 'com.manoramamax.app', title: 'manoramaMAX', ids: ['com.manoramamax.max'], from: '#d4204e', to: '#350712', mark: {logo: 'manoramamax', box: [470, 170], word: 'manoramaMAX', size: 92, weight: 700}},
	{id: 'com.epicon.app', title: 'EPIC ON', ids: ['com.epicon.epicon'], from: '#b8860b', to: '#2c1f03', mark: {logo: 'epicon', mode: 'luma', box: [420, 170], word: 'EPIC ON', size: 120, weight: 760}},
	{id: 'com.docubay.app', title: 'DocuBay', ids: ['com.docubay.docubay'], from: '#0a9e9e', to: '#032b2b', mark: {logo: 'docubay', box: [440, 170], word: 'DocuBay', size: 118, weight: 720}},
	{id: 'com.distrotv.app', title: 'DistroTV', ids: ['com.distro.distrotv'], from: '#6c2bd9', to: '#1a0a3d', mark: {logo: 'distrotv', box: [440, 170], word: 'DistroTV', size: 116, weight: 720}},
	{id: 'com.prasarbharati.waves', title: 'WAVES', from: '#2f6fd6', to: '#0a1d44', mark: {logo: 'waves', box: [600, 280], word: 'WAVES', size: 136, weight: 760}},
	{id: 'com.planetmarathi.app', title: 'Planet Marathi', from: '#e85d04', to: '#381501', mark: {logo: 'planetmarathi', box: [460, 170], word: 'Planet Marathi', size: 88, weight: 700}},
	{id: 'com.gaana.app', title: 'Gaana', ids: ['gaana', 'com.gaana.gaana'], from: '#e2312f', to: '#380808', mark: {logo: 'gaana', box: [420, 170], word: 'gaana', size: 150, weight: 760}},
	{id: 'com.jio.saavn', title: 'JioSaavn', ids: ['jiosaavn', 'com.saavn.app', 'com.jiosaavn.jiosaavn', 'com.saavn.android'], from: '#1fb2a3', to: '#04302b', mark: {logo: 'jiosaavn', box: [460, 170], word: 'JioSaavn', size: 112, weight: 720}},
	// ----- TV -----
	{id: 'com.webos.app.livetv', title: 'Live TV', from: '#2f9488', to: '#062c28', mark: {lucide: 'tv'}},
	{id: 'com.webos.app.lgchannels', title: 'LG Channels', from: '#e2583b', to: '#3b0f05', mark: {logo: 'lgchannels', box: [540, 210], lucide: 'radio-tower'}},
	{id: 'com.webos.app.recordings', title: 'Recordings', from: '#c94047', to: '#360a0d', mark: {lucide: 'circle-dot'}},
	// ----- LG apps -----
	{id: 'com.webos.app.browser', title: 'Browser', from: '#3d74e3', to: '#0a1c4b', mark: {lucide: 'globe'}},
	{id: 'com.webos.app.mediadiscovery', title: 'Media Player', from: '#7a5cf0', to: '#1c104f', mark: {lucide: 'circle-play'}},
	{id: 'com.webos.app.discovery', title: 'App Store', from: '#c0245a', to: '#330415', mark: {lucide: 'shopping-bag'}},
	{id: 'com.webos.app.miracast', title: 'Screen Share', from: '#2c86c9', to: '#06243d', mark: {lucide: 'cast'}},
	{id: 'airplay', title: 'AirPlay', from: '#5d6472', to: '#121419', mark: {si: 'siAirplayvideo', scale: 0.84}},
	{id: 'com.webos.app.homeconnect', title: 'Home Dashboard', from: '#4a5bd0', to: '#0f163c', mark: {lucide: 'house'}},
	{id: 'com.webos.app.lifeonscreen', title: 'Gallery', from: '#c0913f', to: '#30200a', mark: {lucide: 'images'}},
	{id: 'com.webos.app.sportsteamsettings', title: 'Sports', from: '#e3812f', to: '#3b1c04', mark: {lucide: 'trophy'}},
	{id: 'com.webos.app.roomconnect', title: 'Room Share', from: '#2f9873', to: '#062c1e', mark: {lucide: 'arrow-left-right'}},
	{id: 'com.webos.app.rdp', title: 'Remote PC', from: '#58606f', to: '#101216', mark: {lucide: 'monitor'}},
	{id: 'com.webos.app.camera', title: 'Camera', from: '#646a78', to: '#14161b', mark: {lucide: 'camera'}},
	{id: 'com.webos.app.tvuserguide', title: 'User Guide', from: '#9a7a46', to: '#2a1e0c', mark: {lucide: 'book-open'}},
	{id: 'com.palmdts.devmode', title: 'Dev Mode', from: '#454b58', to: '#0b0c10', mark: {lucide: 'code-xml'}},
	// ----- assistants -----
	{id: 'amazon.alexa.view', title: 'Alexa', from: '#18b0dc', to: '#042c3b', mark: {logo: 'alexa', box: [200, 200], lucide: 'message-circle'}},
	{id: 'google.assistant', title: 'Assistant', from: '#4c70f0', to: '#0e1a4c', mark: {si: 'siGoogleassistant'}},
	// ----- games -----
	{id: 'com.twin.app.gamingportal', title: 'Gaming', from: '#7343e0', to: '#1b0c45', mark: {lucide: 'gamepad-2'}},
	{id: 'com.webos.app.gameoptimizer', title: 'Game Optimizer', from: '#3f46a8', to: '#0d0f33', mark: {lucide: 'gauge'}},
	{id: 'com.miniclip.eightballpool', title: '8 Ball Pool', from: '#24855a', to: '#05261a', mark: {logo: 'eightballpool', box: [420, 210], svg: '<path fill-rule="evenodd" d="M12 2.5a9.5 9.5 0 1 0 0 19 9.5 9.5 0 0 0 0-19zm0 4.3a5.2 5.2 0 1 1 0 10.4 5.2 5.2 0 0 1 0-10.4z"/><text x="12" y="15.15" text-anchor="middle" font-family="Geist" font-weight="700" font-size="8.6">8</text>'}},
	// ----- Lumen's own tiles (config/builtinApps.ts), drawn to match -----
	{id: 'lumen:settings', title: 'Settings', from: '#737887', to: '#17181d', mark: {lucide: 'settings'}, builtin: true},
	{id: 'com.palm.app.settings', title: 'TV Settings', from: '#5b6378', to: '#121520', mark: {lucide: 'monitor-cog'}, builtin: true},
	{id: 'lumen:inputs', title: 'Inputs', from: '#3c67cc', to: '#0b183b', mark: {lucide: 'cable'}, builtin: true}
];

/** The home screen, in order: the glass bar holds the first six, the rest make rows below it. Folders come after the apps. */
export const HOME = [
	'youtube.leanback.v4', 'netflix', 'amazon', 'hotstar', 'com.sonyliv2.tvapp', 'com.zee5.app',
	'com.viacom18.media.ondemand',
	// the other Indian OTT and music apps: each takes its place here once it is installed (a missing id is skipped)
	'com.jio.jiotv', 'amazon.minitv', 'com.mxplayer.tv', 'com.airtel.xstream', 'com.tataplay.binge', 'com.altbalaji.app', 'com.d27n.app.erosnow',
	'com.hungama.lgtvapp', 'com.sun.sunnxt', 'aha', 'com.hoichoi.app', 'com.shemaroome.app', 'com.lionsgateplay.app', 'com.discovery.dplus.india',
	'yupptv', 'com.dangalplay.app', 'com.chaupal.app', 'com.timesplay.app', 'com.fancode.app.streaming', 'com.etvwin.app', 'com.manoramamax.app',
	'com.epicon.app', 'com.docubay.app', 'com.distrotv.app', 'com.prasarbharati.waves', 'com.planetmarathi.app',
	'spotify-beehive', 'com.jio.saavn', 'com.gaana.app', 'org.jellyfin.webos', 'io.strem.tv', 'com.webos.app.livetv', 'com.webos.app.lgchannels',
	'com.webos.app.browser', 'com.webos.app.mediadiscovery', 'com.webos.app.miracast', 'airplay', 'com.webos.app.discovery', 'lumen:settings'
];

export const FOLDERS = [
	{name: 'Games', appIds: ['com.twin.app.gamingportal', 'com.miniclip.eightballpool', 'com.webos.app.gameoptimizer']},
	{name: 'LG', appIds: ['com.webos.app.homeconnect', 'com.webos.app.sportsteamsettings', 'com.webos.app.lifeonscreen', 'amazon.alexa.view', 'google.assistant',
		'com.webos.app.roomconnect', 'com.webos.app.rdp', 'com.webos.app.recordings', 'com.webos.app.camera', 'com.webos.app.tvuserguide',
		'com.viacom18.voot', 'com.palmdts.devmode', 'com.palm.app.settings', 'lumen:inputs']}
];
