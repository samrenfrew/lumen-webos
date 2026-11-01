#!/bin/sh
# Every switch-on opens Lumen, the Live TV of a resume never shows. Started at boot by 60-homehook; runs as root.
# With Quick Start+ on, switching on from standby (remote, Wake-on-LAN, Home Assistant) is a resume, not a boot: bootd does nothing and the
# TV brings back its last input. The power state reports "Prepare Resume" about 2 s before "Screen On", with nothing in front yet, so Home is
# opened then. With the Home takeover mounted (home-setup.sh native-home on) Home is the stand-in, which outlives standby and opens Lumen.
# watch.sh still opens Lumen at each wake as the fallback. Findings: docs/home-and-power-on.md, "Waking from standby".
# Switch off with /var/lib/homehook/no-wake-launch (shared with watch.sh).
export PATH="/bin:/sbin:/usr/bin:/usr/sbin:$PATH"
NOWAKE_FLAG=/var/lib/homehook/no-wake-launch
LGHOME=com.webos.app.home
ME="com.samrenfrew.lumen com.samrenfrew.lumen.dev"
APPS=/media/developer/apps/usr/palm/applications
LOG=/tmp/homehook.log

target () {
  # the stand-in while it is mounted over LG Home, else Lumen itself
  if grep -q " /usr/palm/applications/$LGHOME " /proc/mounts; then echo "$LGHOME"; return 0; fi
  for id in $ME; do [ -d "$APPS/$id" ] && { echo "$id"; return 0; }; done
  return 1
}

while true; do
  # a subscription made before standby survives it; if luna-send ever ends, subscribe again
  luna-send -i luna://com.webos.service.tvpower/power/getPowerState '{"subscribe":true}' </dev/null 2>/dev/null | while IFS= read -r l; do
    case "$l" in
      *'"Prepare Resume"'*)
        [ -f "$NOWAKE_FLAG" ] && continue
        t=$(target) || continue
        r=$(luna-send -n 1 luna://com.webos.applicationManager/launch "{\"id\":\"$t\",\"noSplash\":true}" </dev/null 2>&1)
        echo "$(date +%T) resume: open $t: $r" >> "$LOG" ;;
    esac
  done
  sleep 5
done
