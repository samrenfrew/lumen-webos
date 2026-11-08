#!/bin/sh
# Builds Lumen Guide (the overlay app, docs/plans/guide-overlay.md) and installs it on the TV over SSH, the way tools/deploy.sh installs Lumen.
# It is not launched: LIST opens it (the Input Hook bind). npm run deploy:guide, or as part of npm run deploy:home.
set -e
ROOT=$(cd "$(dirname "$0")/.." && pwd)
. "$ROOT/tools/env.sh"
[ -n "$LUMEN_TV_HOST" ] || { echo "LUMEN_TV_HOST is not set. Copy .env.example to .env and set it." >&2; exit 1; }
OUT=${LUMEN_BUILD_DIR:-$ROOT/build}
cd "$ROOT/app"
[ -d node_modules ] || npm ci --no-audit --no-fund
LUMEN_DEV=${LUMEN_DEV:-1} npm run build:guide
mkdir -p "$OUT"
rm -f "$OUT/com.samrenfrew.lumen.guide_"*.ipk
npx ares-package dist-guide --no-minify -o "$OUT"
IPK=$(ls "$OUT/com.samrenfrew.lumen.guide_"*.ipk)

REMOTE_IPK="/tmp/$(basename "$IPK")"
echo "Copying $(basename "$IPK") to $LUMEN_TV_HOST..."
scp -q "$IPK" "$LUMEN_TV_HOST:$REMOTE_IPK"
echo "Installing com.samrenfrew.lumen.guide..."
ssh -tt "$LUMEN_TV_HOST" "luna-send -w 60000 -i luna://com.webos.appInstallService/dev/install '{\"id\":\"com.ares.defaultName\",\"ipkUrl\":\"$REMOTE_IPK\",\"subscribe\":true}' | grep -oE '\"state\":\"[^\"]*\"|\"errorText\":\"[^\"]*\"' | uniq; rm -f '$REMOTE_IPK'" | tr -d '\r'
