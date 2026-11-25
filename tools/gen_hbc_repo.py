#!/usr/bin/env python3
"""Writes a Homebrew Channel repository for one or more .ipk files: <out>/apps.json (the index the Channel reads, same format as
https://repo.webosbrew.org/api/apps.json), and for each app <out>/<id>.manifest.json and <out>/<id>.png. Used by .github/workflows/release.yml.
Usage: gen_hbc_repo.py [--keep <old-dir> <id>[,<id>...]] <base-url-of-out-dir> <out-dir> <ipk> <ipk-url> <dist-dir> [<ipk> <ipk-url> <dist-dir> ...]
<dist-dir> is the built app folder the .ipk was packaged from; its appinfo.json and largeIcon.png describe the app.
--keep lists apps from an earlier listing in <old-dir> as they were, still pointing at their old release, so the frozen web Lumen
stays installable from the Channel. Apps in LISTED_AS get that name in the Channel; on the TV they keep their appinfo.json title."""
import hashlib, json, os, shutil, sys
LISTED_AS = {
    'com.samrenfrew.lumen.native': 'Lumen Native',
    'com.samrenfrew.lumen': 'Lumen (old web version)',
    'com.samrenfrew.lumen.guide': 'Lumen Guide (old web version)',
    'com.samrenfrew.lumen.quick': 'Lumen Quick Settings (old web version)',
}
args = sys.argv[1:]
keep_dir, keep_ids = None, []
if args[:1] == ['--keep']:
    keep_dir, keep_ids, args = args[1], args[2].split(','), args[3:]
if len(args) < 5 or (len(args) - 2) % 3: sys.exit(__doc__)
base, out = args[:2]
apps = args[2:]
os.makedirs(out, exist_ok=True)


def entry(manifest, name):
    title = LISTED_AS.get(manifest['id'], manifest['title'])
    manifest = dict(manifest, title=title)
    json.dump(manifest, open(f'{out}/{name}', 'w'), indent=2)
    return {
        'id': manifest['id'], 'title': title, 'iconUri': manifest['iconUri'], 'manifestUrl': f'{base}/{name}', 'manifest': manifest,
        'pool': 'main', 'shortDescription': manifest['appDescription'],
    }


packages = []
for i in range(0, len(apps), 3):
    ipk, ipk_url, dist = apps[i:i + 3]
    info = json.load(open(f'{dist}/appinfo.json'))
    app_id = info['id']
    shutil.copy(f'{dist}/largeIcon.png', f'{out}/{app_id}.png')
    manifest = {
        'id': app_id, 'version': info['version'], 'type': info.get('type', 'web'), 'title': info['title'],
        'appDescription': info.get('appDescription', ''), 'iconUri': f'{base}/{app_id}.png',
        'sourceUrl': 'https://github.com/samrenfrew/lumen-webos', 'rootRequired': True,
        'ipkUrl': ipk_url, 'ipkHash': {'sha256': hashlib.sha256(open(ipk, 'rb').read()).hexdigest()}, 'ipkSize': os.path.getsize(ipk),
    }
    packages.append(entry(manifest, f'{app_id}.manifest.json'))
    print(f"{out}: {app_id} {info['version']}")
for app_id in keep_ids:
    manifest = json.load(open(f'{keep_dir}/{app_id}.manifest.json'))
    shutil.copy(f'{keep_dir}/{app_id}.png', f'{out}/{app_id}.png')
    packages.append(entry(manifest, f'{app_id}.manifest.json'))
    print(f"{out}: {app_id} {manifest['version']} (kept)")
n = len(packages)
index = {'paging': {'page': 1, 'count': n, 'maxPage': 1, 'itemsTotal': n, 'prevUrl': None, 'nextUrl': None}, 'packages': packages}
json.dump(index, open(f'{out}/apps.json', 'w'), indent=2)
