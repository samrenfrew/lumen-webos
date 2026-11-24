#!/bin/sh
# Builds Lumen Native (native/lumen/) with tools/build_native.sh and installs it on the TV over SSH.
# It is a separate app, so Lumen is left as it is. LUMEN_LAUNCH=1 also opens it. npm run deploy:native.
# LUMEN_SAVER=1 also installs its screensaver package, which replaces the C one (native/saver) under the same id.
set -e
ROOT=$(cd "$(dirname "$0")/.." && pwd)
. "$ROOT/tools/env.sh"
[ -n "$LUMEN_TV_HOST" ] || { echo "LUMEN_TV_HOST is not set. Copy .env.example to .env and set it." >&2; exit 1; }
OUT=${LUMEN_BUILD_DIR:-$ROOT/build}
BUILT=$(LUMEN_BUILD_DIR="$OUT" sh "$ROOT/tools/build_native.sh")
IPKS=$(echo "$BUILT" | head -1)
[ "$LUMEN_SAVER" = 1 ] && IPKS="$IPKS $(echo "$BUILT" | sed -n 2p)"

for IPK in $IPKS; do
  REMOTE_IPK="/tmp/$(basename "$IPK")"
  echo "Copying $(basename "$IPK") to $LUMEN_TV_HOST..."
  scp -q "$IPK" "$LUMEN_TV_HOST:$REMOTE_IPK"
  echo "Installing $(basename "$IPK" | cut -d_ -f1)..."
  ssh -tt "$LUMEN_TV_HOST" "luna-send -w 60000 -i luna://com.webos.appInstallService/dev/install '{\"id\":\"com.ares.defaultName\",\"ipkUrl\":\"$REMOTE_IPK\",\"subscribe\":true}' | grep -oE '\"state\":\"[^\"]*\"|\"errorText\":\"[^\"]*\"' | uniq; rm -f '$REMOTE_IPK'" | tr -d '\r'
done
if [ -n "$LUMEN_LAUNCH" ]; then ssh -tt "$LUMEN_TV_HOST" "luna-send -n 1 luna://com.webos.applicationManager/launch '{\"id\":\"com.samrenfrew.lumen.native\"}'" | tr -d '\r'; fi
