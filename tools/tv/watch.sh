#!/bin/sh
# Home button -> custom launcher, and screensaver -> Frame (art mode). Once a second, checks which app is in front.
#  - Only with /var/lib/homehook/home-takeover present (off by default: LG Input Hook owns the Home button on this TV): when LG's own Home has just
#    come up, opens the launcher instead (polling, because luna-send buffers a subscription's output when it is piped). The launcher's own
#    "LG Home" button touches /tmp/allow_lg_home first, so LG Home can still be opened on purpose (it then stays until you leave it).
#  - After the TV wakes (power state back to Active, or a gap in this loop's once-a-second ticks, which means it was suspended) and after a boot:
#    opens Lumen as soon as the screen is on, whatever the TV opened: every switch-on opens Lumen. resume.sh acts first, at "Prepare Resume";
#    this is the fallback. During a wake the loop ticks every 0.5 s and retries each tick until Lumen has stayed in front for WAKE_SETTLE
#    seconds, or WAKE_SECS have passed; after that, apps are left alone. Switch off with /var/lib/homehook/no-wake-launch.
#  - About a minute after a boot, if no Lumen build is open, starts Lumen hidden (a preload launch), so the first Home press after a boot into
#    Live TV is warm, not a cold start. Switching off (power state "Active Standby") closes every app but Live TV, so the same is done 10 s
#    into each standby, and the first Home press after switch-on is warm too. Switch both off with /var/lib/homehook/no-boot-preload.
#  - At start and after each wake, sets the apps in NO_PRELOAD back to false in the preload manager's file, because the TV turns them back on
#    by itself (seen 26 Oct and 1 Nov). The manager reads the file only at boot, so a fix takes effect at the next reboot.
#  - Keeps /var/lib/homehook/recent: the last eight apps that were in front (not Lumen, not LG Home), newest first, one id per line. Lumen reads
#    it for its Recent row, so apps opened from anywhere show there.
#  - When the TV's power state becomes "Screen Saver" and /var/lib/homehook/no-screensaver exists (the old Settings > Screensaver > Off; Lumen now removes it): ends LG's
#    screensaver at once, in every app. This comes before the video wallpaper, the photos and Frame.
#  - When the TV's power state becomes "Screen Saver" and /var/lib/homehook/ambient-screensaver exists (Settings > Screensaver > Enabled):
#    opens Lumen with {"view":"ambient"} (wallpaper pictures or Immich photos) and ends LG's screensaver. The video wallpaper, when on and in front, wins.
#    Over another app, "from" names it, so a key that ends the screensaver goes back to that app rather than to Lumen's home screen.
#  - When the TV's power state becomes "Screen Saver" (LG's screensaver is not a foreground app, so the power state is what shows it) and
#    /var/lib/homehook/frame-screensaver exists: opens Frame in screensaver mode, then ends LG's screensaver (turnOnScreen) so Frame is what shows.
# Never spins: reacts only when the app changes, and at most once every 3 s. Does nothing if the app is not installed.
# ME lists every Lumen build (stable, dev). Home opens the one that was last in front, or the first installed one in this list. The flag files in
# /var/lib/homehook/ are shared by all of them, which is fine while only one is in front at a time.
ME="com.samrenfrew.lumen com.samrenfrew.lumen.dev com.ammar027.lumen com.ammar027.lumen.dev"
FRAME=com.ammar027.lumen.gallery
LGHOME=com.webos.app.home
FLAG=/tmp/allow_lg_home
SAVER_FLAG=/var/lib/homehook/frame-screensaver
VIDEO_FLAG=/var/lib/homehook/video-wallpaper
AMBIENT_FLAG=/var/lib/homehook/ambient-screensaver
NOSAVER_FLAG=/var/lib/homehook/no-screensaver
AMBIENT_ON=/tmp/lumen-ambient-on
APPS=/media/developer/apps/usr/palm/applications
EXTRA=/var/lib/homehook/hosts-extra
PRIVACY_HOSTS=/var/lib/homehook/hosts-privacy   # Settings > Privacy > Block LG ad servers (privacy.sh hosts on)
TAKEOVER_FLAG=/var/lib/homehook/home-takeover
NOWAKE_FLAG=/var/lib/homehook/no-wake-launch
APPS_HOME=/usr/palm/applications/$LGHOME   # mounted over by home-setup.sh native-home on (the stand-in)
WAKE_SECS=30
WAKE_SETTLE=3
NOPRELOAD_FLAG=/var/lib/homehook/no-boot-preload
RECENT=/var/lib/homehook/recent
PRELOAD_CONF=/var/preferences/webos-preload-manager-conf.json
NO_PRELOAD="netflix airplay com.webos.app.lgchannels com.webos.app.home"   # Home: the preload manager starts it in front, and the stand-in then opens Lumen over the app you are watching
prev=""
prevapp=""
last=0
okhome=0
ticks=0
saverlast=0
pwprev=""
lastme=""
pwlast=""
mesince=0
launchpid=0
tick=$(date +%s)
wakeuntil=$((tick + 90))   # a boot counts as a wake; LG Home may take a while to come up
preloadat=$((tick + 60))   # once, after the boot rush; 0 when done
STANDBY_PRELOAD_SECS=10
standbyat=0                # when to preload in standby; -1 once done for this standby
# prints the Lumen build in front, if any
me_in_front() {
  for id in $ME; do case "$1" in *'"appId":"'$id'"'*) echo "$id"; return 0 ;; esac; done
  return 1
}
# prints the Lumen build Home should open: the last one in front if still installed, else the first installed one
me_target() {
  if [ -n "$lastme" ] && [ -d "$APPS/$lastme" ]; then echo "$lastme"; return 0; fi
  for id in $ME; do [ -d "$APPS/$id" ] && { echo "$id"; return 0; }; done
  return 1
}

# starts Lumen hidden (a preload launch) if no Lumen build is open; $1 names the reason in the log
preload_me() {
  [ -f "$NOPRELOAD_FLAG" ] && return
  target=$(me_target) || return
  run=$(luna-send -n 1 luna://com.webos.applicationManager/running '{}' </dev/null 2>/dev/null)
  for id in $ME; do case "$run" in *'"id":"'$id'"'*) return ;; esac; done
  r=$(luna-send -n 1 luna://com.webos.applicationManager/launch "{\"id\":\"$target\",\"preload\":\"full\"}" </dev/null 2>&1)
  echo "$(date +%T) $1 preload ($target): $r" >> /tmp/homehook.log
}

# sets each app in NO_PRELOAD back to false in the preload manager's file; setPreloadPolicy answers true but changes nothing on this TV
fix_preload() {
  [ -f "$PRELOAD_CONF" ] || return
  changed=""
  for id in $NO_PRELOAD; do grep -qF "\"$id\":true" "$PRELOAD_CONF" && changed="$changed $id"; done
  [ -z "$changed" ] && return
  cp "$PRELOAD_CONF" "$PRELOAD_CONF.tmp" || return
  for id in $changed; do sed -i "s/\"$id\":true/\"$id\":false/" "$PRELOAD_CONF.tmp"; done
  mv "$PRELOAD_CONF.tmp" "$PRELOAD_CONF"
  echo "$(date +%T) preload: set back to off:$changed (from the next reboot)" >> /tmp/homehook.log
}

# remembers an app that came to the front in $RECENT (plain app ids only; Lumen, Frame and LG Home are left out)
note_recent() {
  case "$1" in ''|$LGHOME|$FRAME) return ;; esac
  for id in $ME; do [ "$1" = "$id" ] && return; done
  printf '%s' "$1" | grep -qE '^[A-Za-z0-9._-]+$' || return
  { echo "$1"; [ -f "$RECENT" ] && grep -vxF "$1" "$RECENT"; } | head -n 8 > "$RECENT.tmp" && mv "$RECENT.tmp" "$RECENT"
}

# opens Lumen without waiting (a launch can take seconds right after a wake), unless the last one is still in flight; $1 is the app in front
wake_open() {
  [ "$launchpid" -gt 0 ] && [ -d "/proc/$launchpid" ] && return
  target=$(me_target) || return
  ( r=$(luna-send -n 1 luna://com.webos.applicationManager/launch "{\"id\":\"$target\"}" </dev/null 2>&1)
    echo "$(date +%T) wake: open $target over ${1:-nothing}: $r" >> /tmp/homehook.log ) &
  launchpid=$!
}

fix_preload
while true; do
  # wake: the power state goes back to Active from standby or off (not from the screensaver), or the loop missed its ticks (suspended)
  now=$(date +%s)
  gap=$((now - tick)); tick=$now
  pw=$(luna-send -n 1 luna://com.webos.service.tvpower/power/getPowerState '{}' </dev/null 2>/dev/null)
  # webOS 25 answers "state": "Active", with a space
  pwstate=$(printf '%s' "$pw" | sed -n 's/.*"state": *"\([^"]*\)".*/\1/p')
  woke=0
  [ "$gap" -gt 5 ] && woke=1
  if [ "$pwstate" = Active ] && [ -n "$pwlast" ] && [ "$pwlast" != Active ] && [ "$pwlast" != "Screen Saver" ]; then woke=1; fi
  [ -n "$pwstate" ] && pwlast=$pwstate
  if [ "$woke" -eq 1 ]; then
    wakeuntil=$((now + WAKE_SECS)); last=0
    echo "$(date +%T) wake (gap ${gap}s, power $pwstate)" >> /tmp/homehook.log
    # before the foreground query: right after a wake each query can take seconds
    [ ! -f "$NOWAKE_FLAG" ] && [ "$pwstate" = Active ] && wake_open ""
    fix_preload
  fi
  out=$(luna-send -n 1 luna://com.webos.applicationManager/getForegroundAppInfo '{}' </dev/null 2>/dev/null)
  front=$(me_in_front "$out") && lastme=$front
  app=$(printf '%s' "$out" | sed -n 's/.*"appId":"\([^"]*\)".*/\1/p')
  if [ -n "$app" ] && [ "$app" != "$prevapp" ]; then prevapp=$app; note_recent "$app"; fi
  # wake: Lumen in front as soon as the screen is on, whatever the TV opened
  if [ -n "$front" ]; then [ "$mesince" -eq 0 ] && mesince=$now; else mesince=0; fi
  if [ ! -f "$NOWAKE_FLAG" ] && [ "$now" -lt "$wakeuntil" ] && [ "$pwstate" = Active ]; then
    if [ -n "$front" ]; then
      # settled: the TV can still put its own app over Lumen in the first seconds, so the wake ends only once Lumen has stayed up
      [ $((now - mesince)) -ge "$WAKE_SETTLE" ] && wakeuntil=0
    elif [ "$app" != "$LGHOME" ] || ! grep -q " $APPS_HOME " /proc/mounts; then
      # LG Home's id is the stand-in while the takeover is mounted, and the stand-in opens Lumen itself
      wake_open "$app"
    fi
  fi
  case "$out" in
    *'"appId":"'$LGHOME'"'*)
      # LG Home is in front. Unless the launcher's own LG Home button just asked for it, send it to the launcher, and keep trying (every 3 s) until it
      # works: right after a cold boot LG Home is the first thing on screen and the launcher may not be ready yet.
      now=$(date +%s)
      if [ "$prev" != "$LGHOME" ]; then
        okhome=0
        if [ -f "$FLAG" ]; then
          age=$((now - $(stat -c %Y "$FLAG")))
          [ "$age" -ge 0 ] && [ "$age" -lt 30 ] && okhome=1
        fi
      fi
      takeover=0
      [ -f "$TAKEOVER_FLAG" ] && takeover=1
      if [ "$takeover" -eq 1 ] && [ "$okhome" -eq 0 ] && target=$(me_target) && { [ $((now - last)) -ge 3 ] || [ $((now - last)) -lt 0 ]; }; then
        last=$now
        luna-send -n 1 luna://com.webos.applicationManager/launch "{\"id\":\"$target\"}" </dev/null >/dev/null 2>&1
      fi
      prev=$LGHOME
      ;;
    *) prev="other" ;;
  esac
  # video wallpaper on and the launcher in front: no screensaver, LG's is ended as soon as it starts
  vid=0
  if [ -f "$VIDEO_FLAG" ] && [ -n "$front" ]; then vid=1; fi
  if [ -f "$NOSAVER_FLAG" ]; then
    case "$pw" in
      *'"Screen Saver"'*) luna-send -n 1 luna://com.webos.service.tvpower/power/turnOnScreen '{}' </dev/null >/dev/null 2>&1; echo "$(date +%T) ended screensaver (screensaver off)" >> /tmp/homehook.log ;;
    esac
  elif [ "$vid" -eq 1 ]; then
    case "$pw" in
      *'"Screen Saver"'*) luna-send -n 1 luna://com.webos.service.tvpower/power/turnOnScreen '{}' </dev/null >/dev/null 2>&1; echo "$(date +%T) ended screensaver (video wallpaper)" >> /tmp/homehook.log ;;
    esac
  elif [ -f "$AMBIENT_FLAG" ] && target=$(me_target); then
    # Lumen's ambient screen (your photos) instead of LG's screensaver. While it is up, LG's screensaver still starts after its timer (nothing
    # is pressed), so it is only ended, not answered by opening Lumen again.
    case "$pw" in
      *'"Screen Saver"'*)
        if [ -n "$front" ] && [ -f "$AMBIENT_ON" ]; then
          luna-send -n 1 luna://com.webos.service.tvpower/power/turnOnScreen '{}' </dev/null >/dev/null 2>&1
        elif [ "$pwprev" != "saver" ] && [ $((now - saverlast)) -ge 5 ]; then
          saverlast=$now
          # the app it came up over, so a key that ends the screensaver goes back to it (plain app ids only; not Lumen, LG Home or Frame)
          from=""
          case "$app" in ''|$LGHOME|$FRAME) ;; *) [ -z "$front" ] && printf '%s' "$app" | grep -qE '^[A-Za-z0-9._-]+$' && from=",\"from\":\"$app\"" ;; esac
          r=$(luna-send -n 1 luna://com.webos.applicationManager/launch "{\"id\":\"$target\",\"params\":{\"view\":\"ambient\"$from}}" </dev/null 2>&1)
          echo "$(date +%T) screensaver: ambient ($target) over ${app:-nothing}: $r" >> /tmp/homehook.log
          sleep 1
          luna-send -n 1 luna://com.webos.service.tvpower/power/turnOnScreen '{}' </dev/null >/dev/null 2>&1
        fi
        pwprev="saver"
        ;;
      *) pwprev="other" ;;
    esac
  elif [ -f "$SAVER_FLAG" ] && [ -d "$APPS/$FRAME" ]; then
    case "$pw" in
      *'"Screen Saver"'*)
        if [ "$pwprev" != "saver" ]; then
          now=$(date +%s)
          if [ $((now - saverlast)) -ge 5 ]; then
            saverlast=$now
            echo "$(date +%T) screensaver seen; foreground: $out" >> /tmp/homehook.log
            case "$out" in
              *'"appId":"'$FRAME'"'*) ;;
              *) r=$(luna-send -n 1 luna://com.webos.applicationManager/launch "{\"id\":\"$FRAME\",\"params\":{\"mode\":\"screensaver\"}}" </dev/null 2>&1); echo "$(date +%T) launch Frame: $r" >> /tmp/homehook.log; sleep 1 ;;
            esac
            luna-send -n 1 luna://com.webos.service.tvpower/power/turnOnScreen '{}' </dev/null >/dev/null 2>&1
          fi
        fi
        pwprev="saver"
        ;;
      *) pwprev="other" ;;
    esac
  fi
  # boot: start Lumen hidden once, so the first Home press is warm. Tried once only, and only with the screen on.
  if [ "$preloadat" -gt 0 ] && [ "$now" -ge "$preloadat" ] && [ "$pwstate" = Active ]; then
    preloadat=0
    preload_me boot
  fi
  # switch-off closes every app but Live TV, so start Lumen hidden again once the TV has settled in standby
  if [ "$pwstate" = "Active Standby" ]; then
    [ "$standbyat" -eq 0 ] && standbyat=$((now + STANDBY_PRELOAD_SECS))
    if [ "$standbyat" -gt 0 ] && [ "$now" -ge "$standbyat" ]; then standbyat=-1; preload_me standby; fi
  else
    standbyat=0
  fi
  # keep the extra blocked names (hosts-extra, and Settings > Privacy's list) in /etc/hosts (about every 30 s; the dashboard rewrites the table at boot)
  ticks=$((ticks + 1))
  if [ $((ticks % 30)) -eq 1 ] && { [ -f "$EXTRA" ] || [ -f "$PRIVACY_HOSTS" ]; }; then
    # only "0.0.0.0 <name>" or ":: <name>" lines are ever copied, so a stray line in a file cannot point a name somewhere real
    # one pass: the lines not yet in the table (two greps, not one per line: the privacy list is about 180 lines)
    missing=$(cat "$EXTRA" "$PRIVACY_HOSTS" 2>/dev/null | grep -E '^(0\.0\.0\.0|::)[[:space:]]+[A-Za-z0-9.-]+[[:space:]]*$' | grep -vxF -f /etc/hosts 2>/dev/null)
    [ -n "$missing" ] && printf '%s\n' "$missing" >> /etc/hosts
  fi
  # faster ticks while a wake is open, so Lumen goes up as soon as the TV lets it
  if [ "$now" -lt "$wakeuntil" ]; then sleep 0.5; else sleep 1; fi
done
