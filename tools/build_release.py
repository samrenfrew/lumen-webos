#!/usr/bin/env python3
"""Builds the two .ipk files that are published as a release, without app/private/:
  - Lumen, the stable app id (no third-party icon tiles or logos, no private font, the bundled public-domain paintings as default wallpapers).
  - Lumen Guide, the overlay over Live TV (docs/plans/guide-overlay.md).
Run: python3 tools/build_release.py   -> release/<id>_<version>_all.ipk for each"""
import glob, os, shutil, subprocess
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = f'{ROOT}/release'
env = dict(os.environ, LUMEN_PRIVATE='0', LUMEN_APP_ID='com.samrenfrew.lumen', LUMEN_APP_TITLE='Home', LUMEN_BUILD_DIR=OUT)
subprocess.run(['sh', f'{ROOT}/tools/build_ipk.sh'], env=env, check=True)

app = f'{ROOT}/app'
subprocess.run(['npm', 'run', 'build:guide'], cwd=app, env=env, check=True)
os.makedirs(f'{app}/dist-guide/licenses', exist_ok=True)
shutil.copy(f'{app}/LICENSE', f'{app}/dist-guide/licenses/LICENSE.txt')
for old in glob.glob(f'{OUT}/com.samrenfrew.lumen.guide_*.ipk'): os.remove(old)
subprocess.run(['npx', 'ares-package', 'dist-guide', '--no-minify', '-o', OUT], cwd=app, check=True)
print(glob.glob(f'{OUT}/*.ipk'))
