#!/usr/bin/env python3
"""Builds the .ipk files that are published as a release, without app/private/:
  - Lumen, the stable app id (no third-party icon tiles or logos, no private font, the bundled public-domain paintings as default wallpapers).
  - The side apps (docs/plans/side-apps.md): Lumen Guide, Lumen Quick Settings and Lumen Screensaver.

Each side app has its own version, which goes up only when the app changes (docs/plans/side-apps.md, "Versions of their own"). Its built
files are hashed (every path and its bytes, with the version taken out of appinfo.json) and compared with the last release's, read from
the file named by LUMEN_SIDE_STATE (stable/side-apps.json on the hbc-repo branch; missing on a first release):
  - the same hash: the version stays;
  - a different hash: the last release's patch goes up by one;
  - either way, a version raised by hand in app/public-<app>/appinfo.json above the last release's wins (a minor or major step);
  - no last release for the app: the version in app/public-<app>/appinfo.json.
It writes release/side-apps.json ({id: {version, hash}}) for the workflow to keep beside the listing.
Run: python3 tools/build_release.py   -> release/<id>_<version>_all.ipk for each, and release/side-apps.json"""
import glob, hashlib, json, os, shutil, subprocess
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = f'{ROOT}/release'
APP = f'{ROOT}/app'
SIDE_APPS = [('guide', 'com.samrenfrew.lumen.guide'), ('quick', 'com.samrenfrew.lumen.quick'), ('saver', 'com.samrenfrew.lumen.saver')]
env = dict(os.environ, LUMEN_PRIVATE='0', LUMEN_APP_ID='com.samrenfrew.lumen', LUMEN_APP_TITLE='Home', LUMEN_BUILD_DIR=OUT)


def parse(v):
    return tuple(int(x) for x in v.split('.'))


def dist_hash(dist):
    """SHA-256 of every file's path and bytes, with appinfo.json's version left out, so the hash changes only when the app does."""
    h = hashlib.sha256()
    for path in sorted(glob.glob(f'{dist}/**/*', recursive=True)):
        if not os.path.isfile(path):
            continue
        rel = os.path.relpath(path, dist)
        data = open(path, 'rb').read()
        if rel == 'appinfo.json':
            info = json.loads(data)
            info.pop('version', None)
            data = json.dumps(info, sort_keys=True).encode()
        h.update(rel.encode() + b'\0' + data + b'\0')
    return h.hexdigest()


def side_version(repo, last, digest):
    if not last:
        return repo
    if parse(repo) > parse(last['version']):
        return repo
    if last.get('hash') == digest:
        return last['version']
    a, b, c = parse(last['version'])
    return f'{a}.{b}.{c + 1}'


state_file = os.environ.get('LUMEN_SIDE_STATE', '')
previous = json.load(open(state_file)) if state_file and os.path.isfile(state_file) and os.path.getsize(state_file) else {}

subprocess.run(['sh', f'{ROOT}/tools/build_ipk.sh'], env=env, check=True)

state = {}
for target, app_id in SIDE_APPS:
    dist = f'{APP}/dist-{target}'
    subprocess.run(['npm', 'run', f'build:{target}'], cwd=APP, env=env, check=True)
    os.makedirs(f'{dist}/licenses', exist_ok=True)
    shutil.copy(f'{APP}/LICENSE', f'{dist}/licenses/LICENSE.txt')
    info_path = f'{dist}/appinfo.json'
    info = json.load(open(info_path))
    digest = dist_hash(dist)
    version = side_version(info['version'], previous.get(app_id), digest)
    info['version'] = version
    json.dump(info, open(info_path, 'w'), indent=2)
    state[app_id] = {'version': version, 'hash': digest}
    last = previous.get(app_id)
    print(f"{app_id}: {version} ({'unchanged' if last and last.get('hash') == digest else 'changed' if last else 'first release'})")
    for old in glob.glob(f'{OUT}/{app_id}_*.ipk'):
        os.remove(old)
    subprocess.run(['npx', 'ares-package', f'dist-{target}', '--no-minify', '-o', OUT], cwd=APP, check=True)

json.dump(state, open(f'{OUT}/side-apps.json', 'w'), indent=2)
print(glob.glob(f'{OUT}/*.ipk'))
