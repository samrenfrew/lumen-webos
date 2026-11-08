#!/usr/bin/env python3
"""Writes a Homebrew Channel repository for one or more .ipk files: <out>/apps.json (the index the Channel reads, same format as
https://repo.webosbrew.org/api/apps.json), and for each app <out>/<id>.manifest.json and <out>/<id>.png. Used by .github/workflows/release.yml.
Usage: gen_hbc_repo.py <base-url-of-out-dir> <out-dir> <ipk> <ipk-url> <dist-dir> [<ipk> <ipk-url> <dist-dir> ...]
<dist-dir> is the built app folder the .ipk was packaged from; its appinfo.json and largeIcon.png describe the app."""
import hashlib, json, os, shutil, sys
base, out = sys.argv[1:3]
apps = sys.argv[3:]
if not apps or len(apps) % 3: sys.exit(__doc__)
os.makedirs(out, exist_ok=True)
packages = []
for i in range(0, len(apps), 3):
    ipk, ipk_url, dist = apps[i:i + 3]
    info = json.load(open(f'{dist}/appinfo.json'))
    app_id = info['id']
    shutil.copy(f'{dist}/largeIcon.png', f'{out}/{app_id}.png')
    manifest = {
        'id': app_id, 'version': info['version'], 'type': 'web', 'title': info['title'],
        'appDescription': info.get('appDescription', ''), 'iconUri': f'{base}/{app_id}.png',
        'sourceUrl': 'https://github.com/samrenfrew/lumen-webos', 'rootRequired': True,
        'ipkUrl': ipk_url, 'ipkHash': {'sha256': hashlib.sha256(open(ipk, 'rb').read()).hexdigest()}, 'ipkSize': os.path.getsize(ipk),
    }
    name = f'{app_id}.manifest.json'
    json.dump(manifest, open(f'{out}/{name}', 'w'), indent=2)
    packages.append({
        'id': app_id, 'title': info['title'], 'iconUri': manifest['iconUri'], 'manifestUrl': f'{base}/{name}', 'manifest': manifest,
        'pool': 'main', 'shortDescription': manifest['appDescription'],
    })
    print(f"{out}: {app_id} {info['version']}")
n = len(packages)
index = {'paging': {'page': 1, 'count': n, 'maxPage': 1, 'itemsTotal': n, 'prevUrl': None, 'nextUrl': None}, 'packages': packages}
json.dump(index, open(f'{out}/apps.json', 'w'), indent=2)
