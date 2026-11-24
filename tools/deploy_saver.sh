#!/bin/sh
# Builds Lumen Screensaver (the third overlay app, docs/plans/side-apps.md), now the native one in native/saver/, with tools/build_native_saver.sh
# (private font included), and installs it on the TV over SSH, the way tools/deploy.sh installs Lumen.
# It is not launched: watch.sh opens it when the TV's screensaver starts over an app. npm run deploy:saver, or as part of npm run deploy:home.
set -e
ROOT=$(cd "$(dirname "$0")/.." && pwd)
. "$ROOT/tools/env.sh"
[ -n "$LUMEN_TV_HOST" ] || { echo "LUMEN_TV_HOST is not set. Copy .env.example to .env and set it." >&2; exit 1; }
if [ -n "$LUMEN_SIDE_IF_INSTALLED" ] && ! ssh -o BatchMode=yes -o LogLevel=ERROR "$LUMEN_TV_HOST" 'test -d /media/developer/apps/usr/palm/applications/com.samrenfrew.lumen.native.saver || test -d /media/developer/apps/usr/palm/applications/com.samrenfrew.lumen.saver'; then echo "Lumen Screensaver is off on the TV (Settings > Side apps), so it is left off."; exit 0; fi
OUT=${LUMEN_BUILD_DIR:-$ROOT/build}
IPK=$(LUMEN_BUILD_DIR="$OUT" sh "$ROOT/tools/build_native_saver.sh")

REMOTE_IPK="/tmp/$(basename "$IPK")"
echo "Copying $(basename "$IPK") to $LUMEN_TV_HOST..."
scp -q "$IPK" "$LUMEN_TV_HOST:$REMOTE_IPK"
echo "Installing com.samrenfrew.lumen.native.saver..."
ssh -tt "$LUMEN_TV_HOST" "luna-send -w 60000 -i luna://com.webos.appInstallService/dev/install '{\"id\":\"com.ares.defaultName\",\"ipkUrl\":\"$REMOTE_IPK\",\"subscribe\":true}' | grep -oE '\"state\":\"[^\"]*\"|\"errorText\":\"[^\"]*\"' | uniq; rm -f '$REMOTE_IPK'" | tr -d '\r'
OLD=com.samrenfrew.lumen.saver
if ssh -o BatchMode=yes -o LogLevel=ERROR "$LUMEN_TV_HOST" "test -d /media/developer/apps/usr/palm/applications/$OLD"; then
  echo "Removing $OLD, the old screensaver's id..."
  ssh -tt "$LUMEN_TV_HOST" "luna-send -w 20000 -i luna://com.webos.appInstallService/dev/remove '{\"id\":\"$OLD\",\"subscribe\":true}' | grep -oE '\"state\":\"[^\"]*\"' | uniq" | tr -d '\r'
fi
