#!/bin/sh
# Every switch-on opens Lumen, the Live TV of a resume never shows. Started at boot by 60-homehook; runs as root.
# With Quick Start+ on, switching on from standby (remote, Wake-on-LAN, Home Assistant) is a resume, not a boot: bootd does nothing and the
# TV brings back its last input. The power state reports "Prepare Resume" about 2 s before "Screen On", with nothing in front yet, so Home is
# opened then. Lumen is opened itself, not the Home stand-in, which showed a "Home" loading screen before Lumen.
# watch.sh still opens Lumen at each wake as the fallback. Findings: docs/home-and-power-on.md, "Waking from standby".
# At "Request Power Off", about 3.5 s before standby, a Lumen in front is sent to its home screen. The TV shows a picture of the last screen
# for about 2.5 s after the next switch-on, and it was the screensaver. Lumen is closed in standby, so it starts fresh at each switch-on anyway.
# Only while Settings > Home and Power opens Lumen at power-on (POWERON_SUM); also switched off by /var/lib/homehook/no-wake-launch (shared with watch.sh).
export PATH="/bin:/sbin:/usr/bin:/usr/sbin:$PATH"
NOWAKE_FLAG=/var/lib/homehook/no-wake-launch
POWERON_SUM=/var/lib/homehook/bootd-override.md5   # written by home-setup.sh power-on on, removed by power-on off
ME="com.samrenfrew.lumen com.samrenfrew.lumen.dev"
APPS=/media/developer/apps/usr/palm/applications
LOG=/tmp/homehook.log

target () {
  for id in $ME; do [ -d "$APPS/$id" ] && { echo "$id"; return 0; }; done
  return 1
}

while true; do
  # a subscription made before standby survives it; if luna-send ever ends, subscribe again
  luna-send -i luna://com.webos.service.tvpower/power/getPowerState '{"subscribe":true}' </dev/null 2>/dev/null | while IFS= read -r l; do
    case "$l" in
      *'"Prepare Resume"'*)
        { [ -f "$NOWAKE_FLAG" ] || [ ! -f "$POWERON_SUM" ]; } && continue
        t=$(target) || continue
        r=$(luna-send -n 1 luna://com.webos.applicationManager/launch "{\"id\":\"$t\",\"noSplash\":true}" </dev/null 2>&1)
        echo "$(date +%T) resume: open $t: $r" >> "$LOG" ;;
      *'"Request Power Off"'*)
        [ -f "$NOWAKE_FLAG" ] && continue
        f=$(luna-send -n 1 luna://com.webos.applicationManager/getForegroundAppInfo '{}' </dev/null 2>/dev/null)
        for id in $ME; do
          case "$f" in *'"appId":"'$id'"'*)
            # a plain relaunch is the Home button: Lumen leaves the screensaver, Settings or the Guide for its home screen
            r=$(luna-send -n 1 luna://com.webos.applicationManager/launch "{\"id\":\"$id\"}" </dev/null 2>&1)
            echo "$(date +%T) power off: $id to its home screen: $r" >> "$LOG" ;;
          esac
        done ;;
    esac
  done
  sleep 5
done
