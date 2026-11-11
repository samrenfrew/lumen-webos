#!/bin/sh
# The remote's gear button, bound to this script by watch.sh (LG Input Hook, key 139) when Lumen Quick Settings is installed.
# It asks the TV which app is in front at the press, so the answer is never stale: with a Lumen build in front it opens Lumen's
# Control Center, and with any other app it opens Lumen Quick Settings (docs/plans/quick-settings.md). The query takes about 30 ms.
ME="com.samrenfrew.lumen com.samrenfrew.lumen.dev com.ammar027.lumen com.ammar027.lumen.dev"
QUICK=com.samrenfrew.lumen.quick
out=$(luna-send -n 1 luna://com.webos.applicationManager/getForegroundAppInfo '{}' </dev/null 2>/dev/null)
for id in $ME; do
  case "$out" in *'"appId":"'$id'"'*)
    exec luna-send -n 1 luna://com.webos.applicationManager/launch "{\"id\":\"$id\",\"params\":{\"view\":\"controls\"}}" </dev/null >/dev/null 2>&1 ;;
  esac
done
exec luna-send -n 1 luna://com.webos.applicationManager/launch "{\"id\":\"$QUICK\"}" </dev/null >/dev/null 2>&1
