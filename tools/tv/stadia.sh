#!/bin/sh
# Stadia controller pairing, for Moonlight. A reconnect never sends input on this TV (docs/research/spikes.md), so the fix is a fresh pairing.
# Each time Moonlight comes to the front, a popup asks to pair. OK on the remote starts it, Back (or 60 s with no answer) skips it.
# After OK it forgets the controller and tries to pair again and again, for up to PAIR_SECS, so Stadia + Y can be held at any time.
# A pairing counts only once the controller sends a report, so a pair made before Stadia + Y is undone and tried again.
# After RESTART_AFTER failed tries in a row it restarts the TV's Bluetooth service, which unsticks a pairing the stack has hung on.
# To try again at any time: go Home, then back to Moonlight.
# Log: /var/lib/homehook/stadia.log. Off: /var/lib/homehook/no-stadia.
B=luna://com.webos.service.bluetooth2
MOON=com.limelight.webos
DIR=/var/lib/homehook
LOG=$DIR/stadia.log
ADDR_FILE=$DIR/stadia-address
KEYS=/tmp/stadia-keys
ASK_SECS=60
PAIR_SECS=300
INPUT_SECS=20
# failed tries in a row before the Bluetooth service restart; the restart drops the remote for about 20 s
RESTART_AFTER=3

log() { echo "$(date '+%F %T') $*" >> "$LOG"; }
# every bus call has a time limit: on 2026-11-08 an unanswered hid/connect froze the old script for good
ls1() { t=$1; shift; timeout "$t" luna-send -n 1 "$@" </dev/null 2>/dev/null; }
toast() { ls1 5 luna://com.webos.notification/createToast "{\"sourceId\":\"com.samrenfrew.lumen\",\"message\":\"$1\"}" >/dev/null; }
alert_id=""
alert() {
  close_alert
  alert_id=$(ls1 5 luna://com.webos.notification/createAlert "{\"sourceId\":\"com.samrenfrew.lumen\",\"title\":\"$1\",\"message\":\"$2\",\"buttons\":[{\"label\":\"OK\"}]}" \
    | sed -n 's/.*"alertId": *"\([^"]*\)".*/\1/p')
}
close_alert() {
  [ -n "$alert_id" ] && ls1 5 luna://com.webos.notification/closeAlert "{\"alertId\":\"$alert_id\"}" >/dev/null
  alert_id=""
}
front() {
  [ -f "$DIR/no-stadia" ] && return 1
  case "$(ls1 5 luna://com.webos.applicationManager/getForegroundAppInfo '{}')" in *'"appId":"'$MOON'"'*) return 0 ;; esac
  return 1
}

# the alert's own button cannot start a root command (tried 2026-11-04), so OK and Back are read from the remote's input node
rcu() { grep -A4 '^N: Name="LGE RCU"' /proc/bus/input/devices | sed -n 's/^H: Handlers=.*\(event[0-9]*\).*/\/dev\/input\/\1/p' | head -1; }
keypid=""
start_keys() {
  stop_keys; : > "$KEYS"
  node=$(rcu); [ -n "$node" ] || node=/dev/input/event1
  # one 16-byte event at a time (32-bit timeval): type 1 is a key, value 1 a press; 28 is OK, 412 is Back
  ( exec 3<"$node" || exit
    while ev=$(dd bs=16 count=1 <&3 2>/dev/null | od -An -t x2 -v); do
      [ -n "$ev" ] || break
      case "$ev" in *"0001 001c 0001 0000"*) echo ok >> "$KEYS" ;; *"0001 019c 0001 0000"*) echo back >> "$KEYS" ;; esac
    done ) &
  keypid=$!
}
stop_keys() { [ -n "$keypid" ] && kill "$keypid" 2>/dev/null; keypid=""; }

present() { grep -q '^N: Name="Stadia' /proc/bus/input/devices; }
hidraw() {
  for x in /sys/class/hidraw/hidraw*; do
    readlink "$x" 2>/dev/null | grep -qi '18d1:9400' && grep -q '^HID_NAME=Stadia' "$x/device/uevent" 2>/dev/null \
      && { echo "/dev/${x##*/}"; return 0; }
  done
  return 1
}
# the controller's address from the paired list, else the last one seen (it leaves the list once forgotten)
addr() {
  a=$(ls1 10 $B/device/getStatus '{}' | tr -d ' \n' | sed 's/{"classOfDevice"/\n&/g' | grep '"name":"Stadia' | grep '"paired":true' \
    | sed -n 's/.*"address":"\([0-9a-f:]*\)".*/\1/p' | head -1)
  if printf '%s' "$a" | grep -qE '^([0-9a-f]{2}:){5}[0-9a-f]{2}$'; then
    [ "$a" = "$(cat "$ADDR_FILE" 2>/dev/null)" ] || echo "$a" > "$ADDR_FILE"
    echo "$a"
  else
    grep -E '^([0-9a-f]{2}:){5}[0-9a-f]{2}$' "$ADDR_FILE" 2>/dev/null
  fi
}
# Moonlight's jail has a fixed copy of hidraw0 only; a controller on another number is invisible to it until its node is copied in
JAIL=/var/palm/jail/com.limelight.webos/dev
jail_node() {
  h=$(hidraw) || return
  mm=$(cat "/sys/class/hidraw/${h##*/}/dev" 2>/dev/null); [ -n "$mm" ] && [ -d "$JAIL" ] || return
  j=$JAIL/${h##*/}
  [ -c "$j" ] && [ "$(stat -c '%t:%T' "$j" 2>/dev/null)" = "$(printf '%x:%x' "${mm%:*}" "${mm#*:}")" ] && return
  rm -f "$j"; mknod "$j" c "${mm%:*}" "${mm#*:}" && chown root:jailer "$j" && chmod 660 "$j" && log "added ${h##*/} to Moonlight's jail"
}
# true once the controller sends a report; a bonded but silent link is not a pairing
input_within() {
  end=$(( $(date +%s) + $1 ))
  while [ "$(date +%s)" -lt "$end" ]; do
    if h=$(hidraw); then
      got=$(timeout 3 dd if="$h" bs=64 count=1 2>/dev/null | wc -c)
      [ "${got:-0}" -gt 0 ] && return 0
    else sleep 1; fi
  done
  return 1
}
# one forget, scan, pair, connect; true when the controller's input device is there afterwards
attempt() {
  a=$(addr); [ -n "$a" ] || { log "no controller address known: pair it once in LG settings"; sleep 5; return 1; }
  ls1 10 $B/adapter/unpair "{\"address\":\"$a\"}" >/dev/null
  sleep 1
  present && { ls1 10 $B/hid/disconnect "{\"address\":\"$a\"}" >/dev/null; sleep 2; }
  # luna-send -i keeps the scan open while its stdin is open; timeout ends it whatever happens
  ( sleep 30 | timeout 32 luna-send -i $B/adapter/startDiscovery '{}' >/dev/null 2>&1 ) &
  scan=$!
  sleep 4
  r=$(timeout 20 luna-send -n 2 $B/adapter/pair "{\"address\":\"$a\",\"subscribe\":true}" </dev/null 2>&1 | tr -d ' \n')
  sleep 2
  present || { ls1 10 $B/hid/connect "{\"address\":\"$a\"}" >/dev/null; sleep 2; }
  ls1 5 $B/adapter/cancelDiscovery '{}' >/dev/null
  kill "$scan" 2>/dev/null
  if present; then log "try $n: paired, waiting for input"; return 0; fi
  log "try $n: no pairing: $(printf '%s' "$r" | cut -c1-160)"
  return 1
}
HOLD="Hold Stadia + Y until the light flashes orange. It pairs by itself, however long you take."
pair() {
  log "OK pressed: pairing"
  alert "Pairing the controller" "$HOLD"
  until=$(( $(date +%s) + PAIR_SECS )); n=0; fails=0
  while [ "$(date +%s)" -lt "$until" ]; do
    front || { log "Moonlight left the front: pairing stopped"; close_alert; return; }
    # 2026-11-08: pairs hung in the TV's stack (SMP never finished) until a Bluetooth service restart, then the next try worked
    if [ "$fails" -ge "$RESTART_AFTER" ]; then
      fails=0; log "$RESTART_AFTER tries failed: restarting the TV's Bluetooth service"
      timeout 30 systemctl restart webos-bluetooth-service; sleep 10
      alert "Pairing the controller" "$HOLD"
    fi
    n=$((n + 1))
    fails=$((fails + 1))
    if attempt; then
      jail_node
      alert "Almost there" "Press any button on the controller."
      if input_within "$INPUT_SECS"; then
        log "try $n: input seen, done"; close_alert; toast "Controller paired."; return
      fi
      log "try $n: paired but no input in ${INPUT_SECS}s, trying again"
      alert "Pairing the controller" "$HOLD"
    fi
    sleep 1
  done
  log "gave up after ${PAIR_SECS}s"
  close_alert; toast "Could not pair the controller.<br>Go Home and back to Moonlight to try again."
}

trap 'stop_keys; close_alert; exit' TERM INT
[ -f "$LOG" ] && tail -n 300 "$LOG" > "$LOG.tmp" && mv "$LOG.tmp" "$LOG"
log "started"
was=0; asking=0
while true; do
  sleep 1
  if ! front; then
    [ "$was" -eq 1 ] && { log "Moonlight left the front"; stop_keys; close_alert; asking=0; }
    was=0; continue
  fi
  jail_node
  if [ "$was" -eq 0 ]; then
    was=1; asking=1; asked=$(date +%s)
    log "Moonlight in front: asking"
    start_keys
    alert "Pair the controller?" "Press OK to pair the Stadia controller. Press Back to skip."
  fi
  [ "$asking" -eq 1 ] || continue
  if grep -q ok "$KEYS" 2>/dev/null; then
    asking=0; stop_keys; pair
  elif grep -q back "$KEYS" 2>/dev/null || [ $(( $(date +%s) - asked )) -ge "$ASK_SECS" ]; then
    asking=0; stop_keys; close_alert; log "skipped"
  fi
done
