#!/bin/sh
# The remote's Home button, bound to this script by watch.sh (LG Input Hook, key 773) while Lumen Overlay is installed and its Dock is
# not switched off (/var/lib/homehook/overlay-dock-off). docs/plans/one-overlay.md, "The Dock over any app".
# While Lumen Overlay's screensaver is showing (a /tmp/lumen-saver-<uid>/showing flag names its process), the press goes to it, and it
# closes. With Lumen or LG Home in front it opens Lumen as before. Over any other app it goes to Lumen Overlay, named in "app": the
# Dock opens and starts on that app, a press over the Dock opens Lumen, and one over the Guide or Quick Settings switches to the Dock.
# One TV query only (the app in front, about 30 ms), as the press waits for it.
HOME_ID=com.samrenfrew.lumen.native
OVERLAY=com.samrenfrew.lumen.native.overlay
LAUNCH=luna://com.webos.applicationManager/launch
up=0
for f in /tmp/lumen-saver-*/showing; do
  [ -f "$f" ] || continue
  pid=""; read -r pid 2>/dev/null < "$f"
  case "$pid" in ''|*[!0-9]*) continue ;; esac
  [ -r "/proc/$pid/cmdline" ] || continue
  case "$(tr '\0' ' ' < "/proc/$pid/cmdline")" in *lumen-overlay*) up=1 ;; esac
done
app=$(luna-send -n 1 luna://com.webos.applicationManager/getForegroundAppInfo '{}' </dev/null 2>/dev/null | sed -n 's/.*"appId":"\([^"]*\)".*/\1/p')
[ "$up" = 1 ] || case "$app" in
  ''|com.samrenfrew.lumen|com.samrenfrew.lumen.*|com.webos.app.home) exec luna-send -n 1 $LAUNCH "{\"id\":\"$HOME_ID\"}" </dev/null >/dev/null 2>&1 ;;
esac
printf '%s' "$app" | grep -qE '^[A-Za-z0-9._-]+$' || app=""
exec luna-send -n 1 $LAUNCH "{\"id\":\"$OVERLAY\",\"params\":{\"mode\":\"dock\",\"app\":\"$app\"}}" </dev/null >/dev/null 2>&1
