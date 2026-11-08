#!/bin/sh
# Home button -> custom launcher, and screensaver -> Frame (art mode). Once a second, checks which app is in front.
#  - Only with /var/lib/homehook/home-takeover present (off by default: LG Input Hook owns the Home button on this TV): when LG's own Home has just
#    come up, opens the launcher instead (polling, because luna-send buffers a subscription's output when it is piped). The launcher's own
#    "LG Home" button touches /tmp/allow_lg_home first, so LG Home can still be opened on purpose (it then stays until you leave it).
#  - After the TV wakes (power state back to Active, or a gap in this loop's once-a-second ticks, which means it was suspended) and after a boot:
#    opens Lumen as soon as the screen is on, whatever the TV opened: every switch-on opens Lumen. resume.sh acts first, at "Prepare Resume";
#    this is the fallback. During a wake the loop ticks every 0.5 s and retries each tick until Lumen has stayed in front for WAKE_SETTLE
#    seconds, or WAKE_SECS have passed; after that, apps are left alone. Only while Settings > Home and Power opens Lumen at power-on
#    (home-setup.sh power-on on leaves POWERON_SUM); also switched off by /var/lib/homehook/no-wake-launch.
#  - About a minute after a boot, if no Lumen build is open, starts Lumen hidden (a preload launch), so the first Home press after a boot into
#    Live TV is warm, not a cold start. Switching off (power state "Active Standby") closes every app but Live TV, so the same is done 10 s
#    into each standby, and the first Home press after switch-on is warm too. Switch both off with /var/lib/homehook/no-boot-preload.
#  - Keeps LG's preload manager stopped: checked at start, after each wake and every 30 ticks. It opens LG Home in front every few minutes, and
#    the stand-in then opens Lumen over the app you are watching. Its config file did not stop it: the TV turns Home back on by itself.
#  - Lumen Guide (the overlay, docs/plans/guide-overlay.md), when installed: while Live TV is in front, LIST and GUIDE open it (Input Hook binds),
#    and it is started hidden as Live TV comes up, so the first LIST shows it at once. Elsewhere LIST is left to the app, GUIDE opens Lumen on
#    its Guide, and the guide is closed to free its memory. Input Hook reloads keybinds.json by itself when it changes.
#  - Keeps /var/lib/homehook/recent: the last eight apps that were in front (not Lumen, not LG Home), newest first, one id per line. Lumen reads
#    it for its Recent row, so apps opened from anywhere show there.
#  - When the TV's power state becomes "Screen Saver" and /var/lib/homehook/no-screensaver exists (the old Settings > Screensaver > Off; Lumen now removes it): ends LG's
#    screensaver at once, in every app. This comes before the video wallpaper, the photos and Frame.
#  - When the TV's power state becomes "Screen Saver" and /var/lib/homehook/ambient-screensaver exists (Settings > Screensaver > Enabled):
#    opens Lumen with {"view":"ambient"} (wallpaper pictures or Immich photos) and ends LG's screensaver. The video wallpaper, when on and in front, wins.
#    Over another app, "from" names it, so a key that ends the screensaver goes back to that app rather than to Lumen's home screen.
#  - When the TV's power state becomes "Screen Saver" (LG's screensaver is not a foreground app, so the power state is what shows it) and
#    /var/lib/homehook/frame-screensaver exists: opens Frame in screensaver mode, then ends LG's screensaver (turnOnScreen) so Frame is what shows.
#  - Settings > Brightness: with /var/lib/homehook/backlight-schedule holding "HH:MM <backlight> on|off|keep" lines (Lumen writes it), sets the
#    backlight and Eye Comfort when each part of the day starts, then leaves them, so a change by hand stays until the next one. Checked once a
#    minute with the screen on, so a TV switched on later catches up. With a "ramp <minutes>" line, each part of the day starts at the last
#    one's backlight and moves to its own a little each minute over that time. While Lumen's screensaver is up the backlight waits: the
#    screensaver puts back the level for now when it ends, and this sets it again a minute later if not. The TV keeps a backlight per input
#    (apps, Live TV, each HDMI), so the level is set again on the first switch to each input since it last changed.
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
POWERON_SUM=/var/lib/homehook/bootd-override.md5   # written by home-setup.sh power-on on, removed by power-on off
APPS_HOME=/usr/palm/applications/$LGHOME   # mounted over by home-setup.sh native-home on (the stand-in)
WAKE_SECS=30
WAKE_SETTLE=3
NOPRELOAD_FLAG=/var/lib/homehook/no-boot-preload
RECENT=/var/lib/homehook/recent
BL_SCHEDULE=/var/lib/homehook/backlight-schedule
PRELOADMGR=preload-manager.service
GUIDE=com.samrenfrew.lumen.guide
LIVETV=com.webos.app.livetv
KEYBINDS=/home/root/.config/lginputhook/keybinds.json
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
# a restart of the scripts (TV up over 5 minutes) is not a boot, so it leaves the app in front alone
up=$(cut -d. -f1 /proc/uptime 2>/dev/null)
[ "${up:-0}" -gt 300 ] && wakeuntil=0
preloadat=$((tick + 60))   # once, after the boot rush; 0 when done
STANDBY_PRELOAD_SECS=10
standbyat=0                # when to preload in standby; -1 once done for this standby
blmin=""                   # the minute the brightness schedule was last checked
blset=""                   # the schedule step and level last set
blinputs=""                # the inputs that have had that level (the TV keeps a backlight per input)
eyeset=""                  # the schedule step whose Eye Comfort was last set
eyeinputs=""               # the inputs that have had it
blsoon=0                   # when to check the schedule again after a switch of app, so a new input gets the level
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

# stops LG's preload manager if it runs; Restart=on-failure, so a clean stop holds until the next boot
stop_preload() {
  systemctl is-active -q "$PRELOADMGR" 2>/dev/null || return
  systemctl stop "$PRELOADMGR" 2>/dev/null
  echo "$(date +%T) preload manager stopped" >> /tmp/homehook.log
}

# remembers an app that came to the front in $RECENT (plain app ids only; Lumen, Frame and LG Home are left out)
note_recent() {
  case "$1" in ''|$LGHOME|$FRAME) return ;; esac
  for id in $ME; do [ "$1" = "$id" ] && return; done
  printf '%s' "$1" | grep -qE '^[A-Za-z0-9._-]+$' || return
  { echo "$1"; [ -f "$RECENT" ] && grep -vxF "$1" "$RECENT"; } | head -n 8 > "$RECENT.tmp" && mv "$RECENT.tmp" "$RECENT"
}

# rewrites the LIST (787) and GUIDE (362) binds for $1 = live (both open the guide, GUIDE on its Full guide) or away (GUIDE opens Lumen's Guide, LIST unbound), keeping
# every other bind; the file is written only when it changes
guide_binds() {
  python3 - "$KEYBINDS" "$1" "$GUIDE" "$(me_target)" <<'PY' 2>>/tmp/homehook.log
import json, sys
path, mode, guide, lumen = sys.argv[1:5]
try:
    with open(path) as f: binds = json.load(f)
except Exception:
    binds = {}
want = dict(binds)
if mode == 'live':
    want['787'] = {'action': 'launch', 'id': guide}
    want['362'] = {'action': 'launch', 'id': guide, 'params': {'view': 'full'}}
else:
    want.pop('787', None)
    if lumen:
        want['362'] = {'action': 'exec', 'command': 'luna-send -n 1 luna://com.webos.applicationManager/launch \'{"id":"%s","params":{"view":"guide"}}\'' % lumen}
    else:
        want.pop('362', None)
if want != binds:
    with open(path + '.tmp', 'w') as f: json.dump(want, f)
    import os; os.rename(path + '.tmp', path)
PY
}
# the app in front changed from $2 to $1: into Live TV, the binds for it and the guide started hidden; out of it, the other binds and the guide closed
guide_follow() {
  [ -d "$APPS/$GUIDE" ] || return
  if [ "$1" = "$LIVETV" ]; then
    guide_binds live
    ( r=$(luna-send -n 1 luna://com.webos.applicationManager/launch "{\"id\":\"$GUIDE\",\"params\":{\"hidden\":true}}" </dev/null 2>&1)
      echo "$(date +%T) guide: start hidden: $r" >> /tmp/homehook.log ) &
  elif [ "$2" = "$LIVETV" ] || [ -z "$2" ]; then
    guide_binds away
    luna-send -n 1 luna://com.webos.applicationManager/closeByAppId "{\"id\":\"$GUIDE\"}" </dev/null >/dev/null 2>&1
    [ -n "$2" ] && echo "$(date +%T) guide: closed, left Live TV for $1" >> /tmp/homehook.log
  fi
}

# with power-on set to last input, a switch-on leaves the TV's own app in front
wake_wanted () { [ ! -f "$NOWAKE_FLAG" ] && [ -f "$POWERON_SUM" ]; }
# opens Lumen without waiting (a launch can take seconds right after a wake), unless the last one is still in flight; $1 is the app in front
wake_open() {
  [ "$launchpid" -gt 0 ] && [ -d "/proc/$launchpid" ] && return
  target=$(me_target) || return
  ( r=$(luna-send -n 1 luna://com.webos.applicationManager/launch "{\"id\":\"$target\"}" </dev/null 2>&1)
    echo "$(date +%T) wake: open $target over ${1:-nothing}: $r" >> /tmp/homehook.log ) &
  launchpid=$!
}

# minutes since midnight for "HH:MM" (a leading zero stripped, so 08 and 09 are not read as octal)
hm_min() { h=${1%:*}; m=${1#*:}; echo $(( ${h#0} * 60 + ${m#0} )); }

# prints the brightness schedule's step in force now and the backlight for this minute, as "HH:MM level eye backlight": the step is the
# last one at or before now, else the day's last (from yesterday); with a ramp, the backlight moves from the step before's level to its own
bl_step() {
  [ -s "$BL_SCHEDULE" ] || return 1
  steps=$(grep -E '^([01][0-9]|2[0-3]):[0-5][0-9] [0-9]{1,3} (on|off|keep)$' "$BL_SCHEDULE" | sort)
  [ -n "$steps" ] || return 1
  ramp=$(sed -n 's/^ramp \([0-9]\{1,3\}\)$/\1/p' "$BL_SCHEDULE" | head -n 1)
  now_min=$(hm_min "$(date +%H:%M)")
  last=$(printf '%s\n' "$steps" | tail -n 1)
  cur=$last; prev=""; before=$last
  while read -r at lv eye; do
    if [ "$(hm_min "$at")" -le "$now_min" ]; then prev=$before; cur="$at $lv $eye"; fi
    before="$at $lv $eye"
  done <<STEPS
$steps
STEPS
  # before the day's first step, the last one holds and the one before it is the second to last
  [ -z "$prev" ] && prev=$(printf '%s\n' "$steps" | tail -n 2 | head -n 1)
  set -- $cur
  level=$2
  if [ -n "$ramp" ] && [ "$ramp" -gt 0 ]; then
    since=$(( (now_min - $(hm_min "$1") + 1440) % 1440 ))
    from=$(printf '%s' "$prev" | cut -d' ' -f2)
    [ "$since" -lt "$ramp" ] && level=$(( from + (level - from) * since / ramp ))
  fi
  echo "$cur $level"
}

# prints the input whose picture settings are in use now ("default" for apps, the tuner for Live TV, "hdmi1"...), else the app in front
bl_input() {
  r=$(luna-send -n 1 luna://com.webos.settingsservice/getSystemSettings '{"category":"picture","keys":["backlight"]}' </dev/null 2>/dev/null)
  i=$(printf '%s' "$r" | sed -n 's/.*"input":"\([A-Za-z0-9_]*\)".*/\1/p')
  echo "${i:-$app}"
}

# sets the backlight and Eye Comfort when the schedule's level for now changes (once per step, or each minute of a ramp), on the input in use,
# and again on the first switch to each other input until it changes; a change by hand on an input stays until then
bl_apply() {
  out=$(bl_step) || { blset=""; blinputs=""; eyeset=""; eyeinputs=""; return; }
  set -- $out
  [ "$4" -le 100 ] || return
  [ "$1 $2 $3 $4" != "$blset" ] && { blset="$1 $2 $3 $4"; blinputs=""; }
  [ "$1 $2 $3" != "$eyeset" ] && { eyeset="$1 $2 $3"; eyeinputs=""; }
  inp=$(bl_input)
  case " $blinputs " in
    *" $inp "*) ;;
    *) if [ ! -f "$AMBIENT_ON" ]; then
         luna-send -n 1 luna://com.webos.settingsservice/setSystemSettings "{\"category\":\"picture\",\"settings\":{\"backlight\":\"$4\"}}" </dev/null >/dev/null 2>&1
         blinputs="$blinputs $inp"
         echo "$(date +%T) brightness: backlight $4 on $inp (from $1)" >> /tmp/homehook.log
       fi ;;
  esac
  case " $eyeinputs " in
    *" $inp "*) ;;
    *) eyeinputs="$eyeinputs $inp"
       if [ "$3" != keep ]; then
         luna-send -n 1 luna://com.webos.settingsservice/setSystemSettings "{\"category\":\"picture\",\"settings\":{\"eyeComfortMode\":\"$3\"}}" </dev/null >/dev/null 2>&1
         echo "$(date +%T) brightness: Eye Comfort $3 on $inp (from $1)" >> /tmp/homehook.log
       fi ;;
  esac
}

stop_preload
while true; do
  # wake: the power state goes back to Active from standby or off (not from the screensaver), or the loop missed its ticks (suspended)
  now=$(date +%s)
  gap=$((now - tick))
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
    wake_wanted && [ "$pwstate" = Active ] && wake_open ""
    stop_preload
  fi
  out=$(luna-send -n 1 luna://com.webos.applicationManager/getForegroundAppInfo '{}' </dev/null 2>/dev/null)
  front=$(me_in_front "$out") && lastme=$front
  app=$(printf '%s' "$out" | sed -n 's/.*"appId":"\([^"]*\)".*/\1/p')
  if [ -n "$app" ] && [ "$app" != "$prevapp" ]; then guide_follow "$app" "$prevapp"; prevapp=$app; note_recent "$app"; blsoon=$((now + 3)); fi
  # wake: Lumen in front as soon as the screen is on, whatever the TV opened
  if [ -n "$front" ]; then [ "$mesince" -eq 0 ] && mesince=$now; else mesince=0; fi
  if [ "$now" -lt "$wakeuntil" ] && wake_wanted && [ "$pwstate" = Active ]; then
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
  # Settings > Brightness, once a minute with the screen on; a minute missed in standby is caught up at the first one after
  case "$pwstate" in
    Active|"Screen Saver")
      m=$((now / 60))
      # a switch of app may bring another input; 3 s on, so the TV has moved its picture settings to it
      if [ "$m" != "$blmin" ] || { [ "$blsoon" -gt 0 ] && [ "$now" -ge "$blsoon" ]; }; then blmin=$m; blsoon=0; bl_apply; fi
      ;;
  esac
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
  [ $((ticks % 30)) -eq 0 ] && stop_preload
  if [ $((ticks % 30)) -eq 1 ] && { [ -f "$EXTRA" ] || [ -f "$PRIVACY_HOSTS" ]; }; then
    # only "0.0.0.0 <name>" or ":: <name>" lines are ever copied, so a stray line in a file cannot point a name somewhere real
    # one pass: the lines not yet in the table (two greps, not one per line: the privacy list is about 180 lines)
    missing=$(cat "$EXTRA" "$PRIVACY_HOSTS" 2>/dev/null | grep -E '^(0\.0\.0\.0|::)[[:space:]]+[A-Za-z0-9.-]+[[:space:]]*$' | grep -vxF -f /etc/hosts 2>/dev/null)
    [ -n "$missing" ] && printf '%s\n' "$missing" >> /etc/hosts
  fi
  # the gap counts only the sleep, so a slow pass (the screensaver hand-off takes about 6 s) is not taken for a wake
  tick=$(date +%s)
  # faster ticks while a wake is open, so Lumen goes up as soon as the TV lets it
  if [ "$now" -lt "$wakeuntil" ]; then sleep 0.5; else sleep 1; fi
done
