#!/bin/sh
# The remote's Home button, bound to this script by watch.sh (LG Input Hook, key 773) while Lumen Overlay is installed and its Dock is
# not switched off (/var/lib/homehook/dock-off). docs/plans/one-overlay.md, "The Dock over any app".
# With the old Lumen Screensaver up, Home only closes it. While Lumen Overlay is up, the press goes to it: over its screensaver it closes,
# over the Dock it opens Lumen, and over the Guide or Quick Settings it switches to the Dock. Otherwise, with Lumen or LG Home in front it
# opens Lumen as before, and over any other app it opens the Dock, named in "app" so the Dock starts on it.
HOME_ID=com.samrenfrew.lumen.native
OVERLAY=com.samrenfrew.lumen.native.overlay
OLD_SAVER=com.samrenfrew.lumen.native.saver
LAUNCH=luna://com.webos.applicationManager/launch
run=$(luna-send -n 1 luna://com.webos.applicationManager/running '{}' </dev/null 2>/dev/null)
case "$run" in
  *'"id":"'$OLD_SAVER'"'*) exec luna-send -n 1 luna://com.webos.applicationManager/closeByAppId "{\"id\":\"$OLD_SAVER\"}" </dev/null >/dev/null 2>&1 ;;
esac
app=$(luna-send -n 1 luna://com.webos.applicationManager/getForegroundAppInfo '{}' </dev/null 2>/dev/null | sed -n 's/.*"appId":"\([^"]*\)".*/\1/p')
up=0; case "$run" in *'"id":"'$OVERLAY'"'*) up=1 ;; esac
[ "$up" = 1 ] || case "$app" in
  ''|com.samrenfrew.lumen|com.samrenfrew.lumen.*|com.webos.app.home) exec luna-send -n 1 $LAUNCH "{\"id\":\"$HOME_ID\"}" </dev/null >/dev/null 2>&1 ;;
esac
printf '%s' "$app" | grep -qE '^[A-Za-z0-9._-]+$' || app=""
exec luna-send -n 1 $LAUNCH "{\"id\":\"$OVERLAY\",\"params\":{\"mode\":\"dock\",\"app\":\"$app\"}}" </dev/null >/dev/null 2>&1
