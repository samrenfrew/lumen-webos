#!/bin/sh
# Stadia controller watcher, for Moonlight. After the controller is switched off and on it reconnects but sends no input until it is paired
# again (docs/research/spikes.md). While Moonlight is in front, this notices that and pairs it again as soon as you hold Stadia + Y (orange light).
#  - A connection counts as working once a report arrives on its hidraw node. A pairing made here counts as working at once.
#  - Broken: a connection this script did not pair sends nothing for SILENT_SECS, or a connection drops (it always comes back broken).
#  - Repair: forget the controller, scan, pair, connect. Tried again and again until its input device appears, for up to REPAIR_SECS.
#    A try counts only if the old input device went away first, since a pair made outside pairing mode "succeeds" and changes nothing.
#  - A connection that is not known to work shows an alert: B on the controller dismisses it, OK on the remote starts the repair at once,
#    so there is no wait for SILENT_SECS or for the dead link to time out. OK is read from the remote's input device (the alert's own
#    buttons cannot start a root command, tried 2026-11-04). It is ignored once the controller has sent input.
# Log: /var/lib/homehook/stadia.log. Off: /var/lib/homehook/no-stadia. In any app, not only Moonlight: /var/lib/homehook/stadia-any-app.
# While watching, it also copies the Bluetooth lines from LG's system log to /var/lib/homehook/bt.log (times in UTC), so a drop can be read later.
B=luna://com.webos.service.bluetooth2
MOON=com.limelight.webos
DIR=/var/lib/homehook
LOG=$DIR/stadia.log
ADDR_FILE=$DIR/stadia-address
RAW=/tmp/stadia-raw
SILENT_SECS=30
REPAIR_SECS=180
FIND_TRIES=15

log() { echo "$(date '+%F %T') $*" >> "$LOG"; }
ls1() { luna-send -n 1 "$@" </dev/null 2>/dev/null; }
# an LG toast; Lumen is named as the source because the notification service wants an installed app id
toast() { ls1 luna://com.webos.notification/createToast "{\"sourceId\":\"com.samrenfrew.lumen\",\"message\":\"$1\"}" >/dev/null; }
# an LG alert stays up until closed, so it is the one place that says what to do; LG's own toasts already cover connect and disconnect
alert_id=""
alert() {
  close_alert
  alert_id=$(ls1 luna://com.webos.notification/createAlert "{\"sourceId\":\"com.samrenfrew.lumen\",\"title\":\"$1\",\"message\":\"$2\",\"buttons\":[{\"label\":\"OK\"}]}" \
    | sed -n 's/.*"alertId": *"\([^"]*\)".*/\1/p')
}
close_alert() {
  [ -n "$alert_id" ] && ls1 luna://com.webos.notification/closeAlert "{\"alertId\":\"$alert_id\"}" >/dev/null
  alert_id=""
}
HOLD_MSG="Hold Stadia + Y until the light flashes orange."
# an OK press on the remote (key 28, from the "LGE RCU" node) while the check alert is up starts the repair; the alert's own buttons cannot
ASK=/tmp/stadia-reconnect
RCU=/dev/input/event1
check_alert() {
  alert "Controller check" "Press B on the controller to dismiss. If nothing happens, press OK on the remote to reconnect it."
  alert_shown=1
}
# the node is opened once and read one 16-byte event at a time, so no press is missed between reads
start_ok() {
  [ -n "$okpid" ] && return
  ( exec 3<"$RCU" || exit
    while ev=$(dd bs=16 count=1 <&3 2>/dev/null | od -An -t x2 -v); do
      [ -n "$ev" ] || break
      case "$ev" in *"0001 001c 0001 0000"*) : > "$ASK" ;; esac
    done ) &
  okpid=$!
}
stop_ok() { [ -n "$okpid" ] && kill "$okpid" 2>/dev/null; okpid=""; }
present() { grep -q '^N: Name="Stadia' /proc/bus/input/devices; }
# the input device's instance (9400.0003); it changes at every reconnect, so a drop that comes back within one poll still shows
inst() { grep -A3 '^N: Name="Stadia' /proc/bus/input/devices | sed -n 's/^S: Sysfs=.*\(9400\.[0-9a-f]*\).*/\1/p' | head -1; }
# the controller's hidraw node (Google 18d1:9400)
hidraw() {
  for x in /sys/class/hidraw/hidraw*; do
    readlink "$x" 2>/dev/null | grep -qi '18d1:9400' && { echo "/dev/${x##*/}"; return 0; }
  done
  return 1
}
# the controller's address: read from the paired list while it is there, else the last one seen (it is gone from the list once forgotten)
addr() {
  # one line per device (each starts with classOfDevice); the name and the address are far apart in it
  a=$(ls1 $B/device/getStatus '{}' | tr -d ' \n' | sed 's/{"classOfDevice"/\n&/g' | grep '"name":"Stadia' | grep '"paired":true' \
    | sed -n 's/.*"address":"\([0-9a-f:]*\)".*/\1/p' | head -1)
  if printf '%s' "$a" | grep -qE '^([0-9a-f]{2}:){5}[0-9a-f]{2}$'; then
    [ "$a" = "$(cat "$ADDR_FILE" 2>/dev/null)" ] || echo "$a" > "$ADDR_FILE"
    echo "$a"
  else
    grep -E '^([0-9a-f]{2}:){5}[0-9a-f]{2}$' "$ADDR_FILE" 2>/dev/null
  fi
}
stop_raw() { [ -n "$rawpid" ] && kill "$rawpid" 2>/dev/null; rawpid=""; }
BTLOG=$DIR/bt.log
BT_PAT='webos-bluetooth|bthidmanager|btmtk|bt_time|[[]BT[]]|xhci|ehci|usb [0-9]|hidraw|Stadia|18D1:9400'
# LG rotates its system log about hourly, too soon to read a drop the next day; awk flushes each line so none is lost when it is stopped
start_bt() {
  [ -n "$btpid" ] && return
  [ -f "$BTLOG" ] && [ "$(wc -c < "$BTLOG")" -gt 4000000 ] && tail -n 20000 "$BTLOG" > "$BTLOG.tmp" && mv "$BTLOG.tmp" "$BTLOG"
  echo "$(date -u '+%FT%TZ') --- Moonlight in front" >> "$BTLOG"
  tail -n 0 -F /var/log/legacy-log 2>/dev/null | awk -v p="$BT_PAT" '$0 ~ p { print; fflush() }' >> "$BTLOG" &
  btpid=$!
}
# tail ends on its own at its next write once awk is gone
stop_bt() { [ -n "$btpid" ] && kill "$btpid" 2>/dev/null; btpid=""; }
trap 'stop_bt; stop_raw; stop_ok; exit' TERM INT
# the scan's entry for this address while it is not paired (one line per device, as in addr)
unpaired_line() {
  ls1 $B/device/getStatus '{}' | tr -d ' \n' | sed 's/{"classOfDevice"/\n&/g' | grep "\"address\":\"$1\"" | grep '"paired":false' | head -1
}
# one forget, scan, pair, connect; true when the input device came back
repair_once() {
  a=$(addr) || return 1
  [ -n "$a" ] || return 1
  stop_raw; up=0
  ls1 $B/adapter/unpair "{\"address\":\"$a\"}" >/dev/null
  sleep 2
  # a connected controller can outlast the unpair; drop the link too, and give up this try if its input device is still there
  if present; then ls1 $B/hid/disconnect "{\"address\":\"$a\"}" >/dev/null; sleep 3; fi
  if present; then log "re-pair try skipped: the old connection is still up after unpair and disconnect"; return 1; fi
  # luna-send -i keeps the scan open while its stdin is open
  sleep 40 | luna-send -i $B/adapter/startDiscovery '{}' >/dev/null 2>&1 &
  # the controller shows up in the scan only while it is in pairing mode, so pair the moment it does (2026-11-04: a fixed 4 s wait paired too early)
  n=0; line=""
  while [ "$n" -lt "$FIND_TRIES" ]; do
    line=$(unpaired_line "$a")
    [ -n "$line" ] && break
    sleep 2; n=$((n + 1))
  done
  if [ -z "$line" ]; then ls1 $B/adapter/cancelDiscovery '{}' >/dev/null; log "re-pair try: the controller did not show up in the scan (not in pairing mode)"; return 1; fi
  log "controller in the scan after $((n * 2))s: $(printf '%s' "$line" | cut -c1-300)"
  r=$(timeout 15 luna-send -n 2 $B/adapter/pair "{\"address\":\"$a\",\"subscribe\":true}" </dev/null 2>&1 | tr -d ' \n')
  sleep 2
  present || { ls1 $B/hid/connect "{\"address\":\"$a\"}" >/dev/null; sleep 3; }
  ls1 $B/adapter/cancelDiscovery '{}' >/dev/null
  if present; then log "re-paired $a"; close_alert; toast "Controller paired."; return 0; fi
  log "re-pair try failed (controller not in pairing mode yet?): $(printf '%s' "$r" | cut -c1-160)"
  return 1
}

[ -f "$LOG" ] && tail -n 300 "$LOG" > "$LOG.tmp" && mv "$LOG.tmp" "$LOG"
log "started"
up=0; ok=0; fresh=0; since=0; repair_until=0; rawpid=""; btpid=""; active=0; seen=""; okpid=""; alert_shown=0
rm -f "$ASK"
while true; do
  sleep 2
  now=$(date +%s)
  on=1
  [ -f "$DIR/no-stadia" ] && on=0
  if [ "$on" -eq 1 ] && [ ! -f "$DIR/stadia-any-app" ]; then
    case "$(ls1 luna://com.webos.applicationManager/getForegroundAppInfo '{}')" in *'"appId":"'$MOON'"'*) ;; *) on=0 ;; esac
  fi
  if [ "$on" -eq 0 ]; then
    [ "$active" -eq 1 ] && log "Moonlight left the front: watching paused"
    active=0; repair_until=0; up=0; stop_raw; stop_bt; stop_ok; close_alert; alert_shown=0; rm -f "$ASK"
    continue
  fi
  [ "$active" -eq 0 ] && log "Moonlight in front: watching" && start_bt && start_ok
  active=1
  if present; then
    cur=$(inst)
    if [ "$up" -eq 1 ] && [ -n "$cur" ] && [ "$cur" != "$seen" ]; then
      # 2026-11-03: the link dropped and came back inside one poll, dead, and nothing was shown
      up=0; stop_raw; log "controller reconnected on its own ($seen to $cur)"
    fi
    if [ "$up" -eq 0 ]; then
      up=1; since=$now; ok=$fresh; fresh=0; seen=$cur
      log "controller connected ($([ "$ok" -eq 1 ] && echo "paired here, working" || echo "checking for input"))"
      # a reconnect has always been dead so far, so offer the fix now rather than after SILENT_SECS; while a repair is already running,
      # its Stadia + Y alert stays up instead
      [ "$ok" -eq 0 ] && [ "$repair_until" -le "$now" ] && check_alert
      : > "$RAW"
    fi
    if [ "$ok" -eq 0 ]; then
      # the hidraw node can show up a moment after the input device
      if [ -z "$rawpid" ] && h=$(hidraw); then cat "$h" >> "$RAW" 2>/dev/null & rawpid=$!; fi
      if [ -s "$RAW" ]; then
        ok=1; repair_until=0; alert_shown=0; stop_raw; close_alert; log "input seen: working"
      elif [ $((now - since)) -ge "$SILENT_SECS" ] && [ "$repair_until" -le "$now" ]; then
        repair_until=$((now + REPAIR_SECS)); log "no input for ${SILENT_SECS}s: broken, re-pairing on Stadia + Y"
        alert "Controller not responding" "$HOLD_MSG"
      fi
    fi
  elif [ "$up" -eq 1 ]; then
    up=0; ok=0; stop_raw
    repair_until=$((now + REPAIR_SECS)); log "controller dropped: re-pairing on Stadia + Y"
    alert "Controller disconnected" "$HOLD_MSG"
  fi
  if [ -f "$ASK" ]; then
    rm -f "$ASK"
    # only while the check alert is up, so an OK press used to drive Moonlight's menus does nothing
    if [ "$alert_shown" -eq 1 ] && [ "$ok" -eq 0 ] && [ "$repair_until" -le "$now" ] && present; then
      log "OK on the remote: re-pairing on Stadia + Y"
      alert_shown=0; repair_until=$((now + REPAIR_SECS))
      alert "Reconnecting the controller" "$HOLD_MSG"
    fi
  fi
  if [ "$repair_until" -gt "$now" ] && [ "$ok" -eq 0 ]; then
    if repair_once; then fresh=1; repair_until=0; fi
  elif [ "$repair_until" -gt 0 ] && [ "$repair_until" -le "$now" ]; then
    repair_until=0; log "gave up re-pairing after ${REPAIR_SECS}s; switch the controller off and on to try again"
    close_alert; toast "Stopped waiting for the controller.<br>Switch it off and on to try again."
  fi
done
