import {cpSync, existsSync, mkdirSync, readFileSync, readdirSync, statSync, writeFileSync} from 'node:fs';
import {dirname, join, normalize, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {defineConfig, loadEnv, type Plugin} from 'vite';
import preact from '@preact/preset-vite';

const APP_DIR = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(APP_DIR, '..');

// webOS 6.x runs Chrome 79. Two things matter for that engine:
//  1. Module scripts (`<script type="module">`) are refused when loaded from file://
//     because the sandbox serves them with no MIME type, so the bundle must be a
//     classic script.
//  2. ES2020 syntax such as `?.` and `??` is a SyntaxError, so everything
//     (dependencies included) is compiled down to the chrome79 target.
const BROWSER_TARGET = 'chrome79';

/** Rewrites Vite's module entry tag into a classic deferred script tag. */
function classicScriptTag (): Plugin {
	return {
			name: 'classic-script-tag',
			enforce: 'post',
			transformIndexHtml (html) {
				return html
					.replace(/<script type="module" crossorigin src="([^"]+)"><\/script>/g, '<script defer src="$1"></script>')
					.replace(/<link rel="stylesheet" crossorigin href="/g, '<link rel="stylesheet" href="');
			}
		};
	}

	/**
	 * Build settings from the repo-root `.env` (see `.env.example`); variables already set in the environment win.
	 *  - LUMEN_APP_ID, LUMEN_APP_TITLE: stamped into dist/appinfo.json and the APP_ID constant. Default: the dev build.
	 *  - LUMEN_PRIVATE=0: build without app/private/ (what the public release is).
	 *  - LUMEN_DEV=1 / 0: a dev build or not (default: dev when the id ends in .dev). Dev builds offer Settings > Frame-rate counter.
	 */
	function lumenSettings (mode: string) {
		const env = loadEnv(mode, REPO_ROOT, 'LUMEN_');
		const privateDir = join(APP_DIR, 'private');
		return {
			appId: env.LUMEN_APP_ID || 'com.samrenfrew.lumen.dev',
			title: env.LUMEN_APP_TITLE || 'Home Dev',
			// a dev build (the .dev id, or LUMEN_DEV=1 from the release workflow for branch pushes) offers tools that stable does not
			dev: env.LUMEN_DEV === '1' || (env.LUMEN_DEV !== '0' && /\.dev$/.test(env.LUMEN_APP_ID || 'com.samrenfrew.lumen.dev')),
			privateDir: env.LUMEN_PRIVATE !== '0' && existsSync(privateDir) ? privateDir : ''
		};
	}

	/** The TV-side scripts, shipped inside the app as dist/tv/ so Settings > System > TV scripts can install them (lib/root.ts installTvScripts).
	 *  Their source stays in tools/tv/, where the SSH tools and the docs point. Keep this list in step with TV_SCRIPTS in lib/root.ts. */
	const TV_SCRIPTS = ['watch.sh', 'stadia.sh', 'resume.sh', 'home-setup.sh', 'privacy.sh', '60-homehook', 'hosts-extra'];
	function tvScripts (): Plugin {
		let outDir = 'dist';
		return {
			name: 'lumen-tv-scripts',
			configResolved (config) { outDir = resolve(config.root, config.build.outDir); },
			writeBundle () {
				mkdirSync(join(outDir, 'tv'), {recursive: true});
				for (const f of TV_SCRIPTS) cpSync(join(REPO_ROOT, 'tools', 'tv', f), join(outDir, 'tv', f));
			}
		};
	}

	const PRIVATE_MODULE = 'virtual:lumen-private';
	const slug = (s: string) => s.toLowerCase().replace(/[^a-z0-9]+/g, '');

	/**
	 * Personal assets that are never committed or released live in app/private/ (gitignored). Its layout mirrors dist/, and the build lays it over dist/:
	 *   private/iconpack/*.png|jpg|webp   icon tiles, matched to apps by file name (tools/gen_iconpack.py)
	 *   private/icons/apps/<id>.png       extra system app icons
	 *   private/wallpapers/default-N*.jpg your own default wallpapers, with private/wallpapers.json: [{"title": ..., "file": "wallpapers/default-N.jpg"}]
	 *   private/fonts/sf.woff2            the private SF font (tools/make_sf_font.py)
	 * The dev server serves the same files. Without the folder (or with LUMEN_PRIVATE=0) the build is the public one.
	 */
	function lumenPrivate (privateDir: string, appId: string, title: string): Plugin {
		let outDir = 'dist';
		return {
			name: 'lumen-private',
			configResolved (config) { outDir = resolve(config.root, config.build.outDir); },
			resolveId (id) { return id === PRIVATE_MODULE ? '\0' + PRIVATE_MODULE : undefined; },
			load (id) {
				if (id !== '\0' + PRIVATE_MODULE) return undefined;
				const packDir = privateDir && join(privateDir, 'iconpack');
				const pack: Record<string, string> = {};
				if (packDir && existsSync(packDir)) {
					for (const f of readdirSync(packDir).sort()) if (/\.(png|jpe?g|webp)$/i.test(f)) pack[slug(f.replace(/\.[^.]+$/, ''))] = f;
				}
				const wpFile = privateDir && join(privateDir, 'wallpapers.json');
				const wallpapers = wpFile && existsSync(wpFile) ? JSON.parse(readFileSync(wpFile, 'utf8')) : null;
				return `export const ICON_PACK = ${JSON.stringify(pack)};\nexport const PRIVATE_WALLPAPERS = ${JSON.stringify(wallpapers)};\n`;
			},
			configureServer (server) {
				if (!privateDir) return;
				server.middlewares.use((req, res, next) => {
					const path = decodeURIComponent((req.url || '').split('?')[0]);
					const file = normalize(join(privateDir, path));
					if (!file.startsWith(privateDir + '/') || file.endsWith('wallpapers.json') || !existsSync(file) || !statSync(file).isFile()) { next(); return; }
					res.end(readFileSync(file));
				});
			},
			writeBundle () {
				if (privateDir) cpSync(privateDir, outDir, {recursive: true, filter: (src) => !src.endsWith('wallpapers.json') && !src.endsWith('.DS_Store')});
				const infoPath = join(outDir, 'appinfo.json');
				const info = JSON.parse(readFileSync(infoPath, 'utf8'));
				info.id = appId;
				info.title = title;
				writeFileSync(infoPath, JSON.stringify(info, null, 2) + '\n');
			}
		};
	}

	/** Lumen Guide (docs/plans/guide-overlay.md): the overlay app, built from guide.html into dist-guide/ with LUMEN_TARGET=guide. Its appinfo is
	 *  public-guide/appinfo.json; from app/private/ it takes only the font, since it shows no icons or wallpapers. */
	const GUIDE_ID = 'com.samrenfrew.lumen.guide';
	function guideFont (privateDir: string): Plugin {
		let outDir = 'dist-guide';
		return {
			name: 'lumen-guide-font',
			configResolved (config) { outDir = resolve(config.root, config.build.outDir); },
			resolveId (id) { return id === PRIVATE_MODULE ? '\0' + PRIVATE_MODULE : undefined; },
			load (id) { return id === '\0' + PRIVATE_MODULE ? 'export const ICON_PACK = {};\nexport const PRIVATE_WALLPAPERS = null;\n' : undefined; },
			writeBundle () {
				const font = privateDir && join(privateDir, 'fonts', 'sf.woff2');
				if (font && existsSync(font)) { mkdirSync(join(outDir, 'fonts'), {recursive: true}); cpSync(font, join(outDir, 'fonts', 'sf.woff2')); }
			}
		};
	}

	export default defineConfig(({mode}) => {
		const lumen = lumenSettings(mode);
		const guide = process.env.LUMEN_TARGET === 'guide';
		const appId = guide ? GUIDE_ID : lumen.appId;
		return {
		base: './',            // the app is loaded from a local file path on the TV
		publicDir: guide ? 'public-guide' : 'public',
		plugins: guide
			? [preact(), classicScriptTag(), guideFont(lumen.privateDir)]
			: [preact(), classicScriptTag(), lumenPrivate(lumen.privateDir, lumen.appId, lumen.title), tvScripts()],
		define: {__LUMEN_APP_ID__: JSON.stringify(appId), __LUMEN_DEV__: JSON.stringify(mode === 'development' || lumen.dev)},
		build: {
			target: BROWSER_TARGET,
			outDir: guide ? 'dist-guide' : 'dist',
			modulePreload: false,
			cssTarget: BROWSER_TARGET,
			rollupOptions: {
				input: guide ? resolve(APP_DIR, 'guide.html') : resolve(APP_DIR, 'index.html'),
				output: {
					format: 'iife',
					inlineDynamicImports: true,
					entryFileNames: 'assets/app.js',
					assetFileNames: 'assets/[name][extname]'
				}
			}
		}
	};
});
