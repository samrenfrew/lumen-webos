#!/bin/sh
# Makes Lumen the TV's Home app and the app the TV opens when it is switched on. Runs on the TV as root, from /var/lib/homehook/.
# Background and sources: docs/home-and-power-on.md, docs/research/prior-art.md (Hearth 2.0, Glasshouse).
# The Home takeover is adapted from Glasshouse (rorygallagher2024/glasshouse, MIT): server/lib/home-takeover.js and samrescan.js.
#
#   sh home-setup.sh check              read only: what this TV has, and whether each method can work here. Run this first.
#   sh home-setup.sh status             read only: what is in place now.
#   sh home-setup.sh state              read only: the same as key=value lines, for Lumen's Settings (lib/tvsetup.ts).
#   sh home-setup.sh native-home on     Home, Exit and Back out of an app open Lumen: a stand-in app under LG Home's id is bind-mounted
#                                       over LG Home's folder, and the stand-in opens Lumen. Redone at each boot by the guard.
#   sh home-setup.sh native-home off    back to LG Home.
#   sh home-setup.sh power-on on        the power button, the TV's own button and a cold start open Lumen (a copy of bootd's file).
#                                       Starts working after one full restart (`reboot`).
#   sh home-setup.sh power-on off       the TV opens its last input again from the next restart.
#   sh home-setup.sh guard              run at each boot by 60-homehook: undoes both if no Lumen build is installed, and removes the
#                                       power-on copy if a firmware update changed LG's own file.
#
# Nothing here changes LG's own files: the mount only covers them. Every change is logged to /tmp/homehook.log and can be undone with the
# "off" commands, or by hand:
#   rm -f /var/luna/preferences/bootd.json; reboot
#   rm -f /var/lib/homehook/home-mount; reboot
export PATH="/bin:/sbin:/usr/bin:/usr/sbin:$PATH"

HH="${HH:-/var/lib/homehook}"
APPS="${APPS:-/media/developer/apps/usr/palm/applications}"
ME="com.samrenfrew.lumen com.samrenfrew.lumen.dev"
LGHOME=com.webos.app.home
STOCK="${STOCK:-/etc/palm/bootd.json}"
OVERRIDE="${OVERRIDE:-/var/luna/preferences/bootd.json}"
BOOTD_BIN="${BOOTD_BIN:-/usr/sbin/bootd}"
OVR_SUM="$HH/bootd-override.md5"     # the sum of the copy we wrote: a copy with another sum is someone else's and is left alone
STOCK_SUM="$HH/bootd-stock.md5"      # LG's file when we wrote the copy: if it changes (firmware update), the copy goes
HOME_BACKUP="$HH/defaultApps.backup.json"
LGHOME_DIR="${LGHOME_DIR:-/usr/palm/applications/$LGHOME}"
STAND_IN="$HH/home-app"               # the stand-in Home app, mounted over LG Home's folder
TAKEOVER="$HH/home-mount"          # holds the Lumen id while the takeover is on; the guard mounts again at each boot
BLOCK_SCHEMA=/etc/palm/schemas/sam/applicationManager.blockSystemApp.schema
LOADED="${LOADED:-/tmp/lumen-poweron.loaded}"   # what bootd read at this boot (written once per boot by the guard): an app id or "off"
REASONS="remoteKey localKey micomPower wakeOnLan wakeOnWiFi"
LOG="${LOG:-/tmp/homehook.log}"
LUNA="${LUNA:-luna-send}"

log () { echo "$(date +%T) home-setup: $*" >> "$LOG"; }
say () { echo "$*"; log "$*"; }
sum () { md5sum "$1" 2>/dev/null | cut -d ' ' -f 1; }
installed () { [ -f "$APPS/$1/appinfo.json" ]; }

# the Lumen build to point at: $1 if given (must be one of ME), else the stable build if installed, else the dev build
target () {
  if [ -n "$1" ]; then
    for id in $ME; do [ "$1" = "$id" ] && installed "$id" && { echo "$id"; return 0; }; done
    return 1
  fi
  for id in $ME; do installed "$id" && { echo "$id"; return 0; }; done
  return 1
}
any_lumen () { for id in $ME; do installed "$id" && return 0; done; [ ! -d "$APPS" ]; }   # no apps folder (not mounted yet) counts as installed

ours () { [ -s "$OVERRIDE" ] && [ -f "$OVR_SUM" ] && [ "$(sum "$OVERRIDE")" = "$(cat "$OVR_SUM")" ]; }

home_app () {   # the app general.defaultApps.home names, or ''
  $LUNA -n 1 luna://com.webos.settingsservice/getSystemSettings '{"category":"general","keys":["defaultApps"]}' 2>/dev/null \
    | tr -d '\n' | sed -n 's/.*"defaultApps": *{[^}]*"home": *"\([^"]*\)".*/\1/p'
}
# the takeover is in place: LG Home's folder is the stand-in's
mounted () { [ "$(stat -c %d:%i "$LGHOME_DIR" 2>/dev/null)" = "$(stat -c %d:%i "$STAND_IN" 2>/dev/null)" ] && [ -d "$STAND_IN" ]; }
# the app Home opens: the takeover's Lumen while mounted, else the system setting
home_now () { if mounted; then cat "$TAKEOVER" 2>/dev/null; else home_app; fi; }
app_category () {   # the appCategory array the app manager has for $1, as printed
  $LUNA -n 1 luna://com.webos.service.applicationmanager/getAppInfo "{\"id\":\"$1\",\"properties\":[\"id\",\"appCategory\"]}" 2>/dev/null \
    | tr -d '\n ' | sed -n 's/.*"appCategory":\(\[[^]]*\]\).*/\1/p'
}

do_check () {
  echo "== Lumen builds installed"
  for id in $ME; do installed "$id" && echo "  $id: yes, appCategory $(app_category "$id")" || echo "  $id: no"; done
  echo "  LG Home ($LGHOME) appCategory: $(app_category "$LGHOME")"
  echo "== Native Home (takeover)"
  if [ -f "$BLOCK_SCHEMA" ]; then echo "  blockSystemApp: yes"; else echo "  blockSystemApp: NO (native-home on will refuse)"; fi
  echo "  LG Home process: $(lghome_pids)"
  mounted && echo "  mounted: yes, opens $(cat "$TAKEOVER" 2>/dev/null)" || echo "  mounted: no"
  echo "  general.defaultApps.home: $(home_app)"
  echo "== Power-on app (bootd)"
  if grep -q "$OVERRIDE" "$BOOTD_BIN" 2>/dev/null; then echo "  bootd names $OVERRIDE: yes"; else echo "  bootd names $OVERRIDE: NO (this method will not work here)"; fi
  if grep -q firstAppByPowerOnReason "$STOCK" 2>/dev/null; then echo "  $STOCK has firstAppByPowerOnReason: yes"; else echo "  $STOCK has firstAppByPowerOnReason: NO (this method will not work here)"; fi
  if command -v node >/dev/null 2>&1; then echo "  node: yes (needed to write the copy)"; else echo "  node: NO (power-on on will refuse)"; fi
  echo "  LG's table:"
  if command -v node >/dev/null 2>&1; then
    node -e 'var t=JSON.parse(require("fs").readFileSync(process.argv[1],"utf8")).common.firstAppByPowerOnReason;Object.keys(t).forEach(function(k){console.log("    "+k+": "+t[k])})' "$STOCK" 2>&1
  else grep -A15 firstAppByPowerOnReason "$STOCK" | sed 's/^/    /'; fi
  [ -f "$OVERRIDE" ] && { ours && echo "  a copy is in place (ours)" || echo "  a copy is in place that is NOT ours: power-on on will refuse"; }
  echo "  last power-on reason: $($LUNA -n 1 luna://com.webos.service.tvpower/power/getPowerOnReason '{}' 2>&1 | tr -d '\n')"
  echo "  bootd's loaded copy is only read at a full restart; status shows what is on disk."
}

do_status () {
  echo "native-home: $(home_now)$(mounted && echo ' (takeover mounted)')"
  if ours; then
    echo "power-on: $(override_app) (ours; from the next full restart if just set)"
  elif [ -f "$OVERRIDE" ]; then echo "power-on: a copy that is not ours is in place"
  else echo "power-on: off (LG's own table)"; fi
}

override_app () { tr -d '\n' < "$OVERRIDE" 2>/dev/null | sed -n 's/.*"remoteKey" *: *"\([^"]*\)".*/\1/p'; }

# key=value lines: home (the Home app now), native (ok, or why it cannot be set), poweron (an app id, off or foreign),
# poweron_support (ok, or why not), poweron_loaded (what bootd read at this boot: an app id, off, or unknown when the guard has not run)
do_state () {
  echo "home=$(home_now)"
  if ! target >/dev/null; then echo native=no-lumen
  elif [ ! -f "$BLOCK_SCHEMA" ]; then echo native=no-block
  elif [ ! -d "$LGHOME_DIR" ]; then echo native=no-lghome
  else echo native=ok; fi
  if ours; then echo "poweron=$(override_app)"; elif [ -f "$OVERRIDE" ]; then echo poweron=foreign; else echo poweron=off; fi
  if ! grep -q "$OVERRIDE" "$BOOTD_BIN" 2>/dev/null; then echo poweron_support=no-bootd
  elif ! grep -q firstAppByPowerOnReason "$STOCK" 2>/dev/null; then echo poweron_support=no-table
  elif ! command -v node >/dev/null 2>&1; then echo poweron_support=no-node
  else echo poweron_support=ok; fi
  echo "poweron_loaded=$(cat "$LOADED" 2>/dev/null || echo unknown)"
}

lghome_pids () {   # LG's own Home: the Flutter runner given its id (webOS 25), or its own program on older firmware
  for p in /proc/[0-9]*; do
    case "$({ tr '\0' ' ' < "$p/cmdline"; } 2>/dev/null)" in
      "/usr/bin/flutter-client -i $LGHOME"*|"/usr/bin/$LGHOME"*) echo "${p#/proc/}" ;;
    esac
  done
}

# the app manager reads an appinfo.json only when it scans: blocking and releasing one app makes it read that app again, without a restart
reread () {
  f=; grep -q forceBlockSystemApp "$BLOCK_SCHEMA" 2>/dev/null && f=',"forceBlockSystemApp":false'
  $LUNA -n 1 -w 8000 luna://com.webos.applicationManager/blockSystemApp "{\"id\":\"$LGHOME\"$f}" >/dev/null 2>&1
  # always released, even after a refused block: an app left blocked cannot launch
  for i in 1 2 3; do
    $LUNA -n 1 -w 8000 luna://com.webos.applicationManager/releaseBlockSystemApp "{\"id\":\"$LGHOME\"}" 2>/dev/null | grep -q '"returnValue": *true' && break
    sleep 1
  done
}
sam_type () { $LUNA -n 1 -w 8000 luna://com.webos.applicationManager/getAppInfo "{\"id\":\"$LGHOME\"}" 2>/dev/null | tr -d '\n ' | sed -n 's/.*"type":"\([^"]*\)".*/\1/p'; }

# the stand-in: a blank page under LG Home's id that opens Lumen whenever it is shown, so Lumen stays one app
stage () {
  mkdir -p "$STAND_IN" || return 1
  [ -f "$STAND_IN/icon.png" ] || cp "$LGHOME_DIR/icon.png" "$STAND_IN/icon.png" 2>/dev/null
  cat > "$STAND_IN/appinfo.json" <<EOF
{
  "id": "$LGHOME",
  "version": "1.0.0",
  "vendor": "Lumen",
  "type": "web",
  "main": "index.html",
  "title": "Home",
  "icon": "icon.png",
  "iconColor": "#000000",
  "bgColor": "#000000",
  "resolution": "1920x1080",
  "visible": false,
  "disableBackHistoryAPI": true,
  "noSplashOnLaunch": true,
  "spinnerOnLaunch": false,
  "appCategory": ["home"],
  "requiredPermissions": ["applications", "applications.launcher", "applications.operation", "applications.internal"]
}
EOF
  cat > "$STAND_IN/index.html" <<EOF
<!doctype html><html><head><meta charset="utf-8"><style>html,body{margin:0;background:#000}</style></head><body><script>
function lumen () { try { var b = new PalmServiceBridge(); b.onservicecallback = function () {}; b.call("luna://com.webos.applicationManager/launch", JSON.stringify({id: "$1"})); } catch (e) {} }
// the preload manager starts LG Home hidden in the background; opening Lumen then would pull it over whatever is playing
function shown () { if (!document.hidden) lumen(); }
shown(); document.addEventListener("webOSRelaunch", shown, true); document.addEventListener("visibilitychange", shown);
</script></body></html>
EOF
}

# mount the stand-in, have the app manager read it, then stop LG's Home so the next Home opens the stand-in
take () {
  stage "$1" || { say "native-home: could not write the stand-in"; return 1; }
  mounted || mount --bind "$STAND_IN" "$LGHOME_DIR" || { say "native-home: the mount failed"; return 1; }
  reread
  if [ "$(sam_type)" != web ]; then give_back; say "native-home: the app manager did not take the stand-in; LG Home is back"; return 1; fi
  for pid in $(lghome_pids); do kill "$pid" 2>/dev/null; done
  return 0
}

give_back () {
  if mounted; then umount "$LGHOME_DIR" 2>/dev/null || umount -l "$LGHOME_DIR" 2>/dev/null; fi
  reread
  $LUNA -n 1 luna://com.palm.webappmanager/killApp "{\"appId\":\"$LGHOME\"}" >/dev/null 2>&1   # the stand-in's page, if it runs
}

native_on () {
  t=$(target "$1") || { say "native-home: no Lumen build installed${1:+ as $1}"; return 1; }
  [ -f "$BLOCK_SCHEMA" ] || { say "native-home: this TV has no blockSystemApp, so the app manager cannot be made to read the stand-in"; return 1; }
  [ -d "$LGHOME_DIR" ] || { say "native-home: no $LGHOME_DIR on this TV"; return 1; }
  take "$t" || return 1
  echo "$t" > "$TAKEOVER"; sync
  say "native-home on ($t): Home, Exit and Back out of an app open $t"
}

native_off () {
  rm -f "$TAKEOVER"; sync
  give_back
  # the earlier method (general.defaultApps.home) did nothing on this TV; clear it if a build of ours still has it
  case " $ME " in *" $(home_app) "*)
    $LUNA -n 1 luna://com.webos.settingsservice/setSystemSettings "{\"category\":\"general\",\"settings\":{\"defaultApps\":{\"home\":\"$LGHOME\"}}}" >/dev/null 2>&1 ;;
  esac
  say "native-home off: LG Home is back ($(sam_type))"
}

power_on () {
  t=$(target "$1") || { say "power-on: no Lumen build installed${1:+ as $1}"; return 1; }
  grep -q "$OVERRIDE" "$BOOTD_BIN" 2>/dev/null || { say "power-on: bootd on this TV does not read $OVERRIDE; nothing changed"; return 1; }
  grep -q firstAppByPowerOnReason "$STOCK" 2>/dev/null || { say "power-on: $STOCK has no firstAppByPowerOnReason table; nothing changed"; return 1; }
  command -v node >/dev/null 2>&1 || { say "power-on: no node on this TV to write the JSON safely; nothing changed"; return 1; }
  if [ -f "$OVERRIDE" ] && ! ours; then say "power-on: $OVERRIDE exists and is not ours; left alone"; return 1; fi
  new="$OVERRIDE.lumen-new"
  # LG's file with the power-on reasons pointed at Lumen; every other reason (casting, HDMI-CEC, timers) stays LG's
  node -e '
    var fs = require("fs"), a = process.argv;
    var o = JSON.parse(fs.readFileSync(a[1], "utf8"));
    if (!o.common || typeof o.common.firstAppByPowerOnReason !== "object") process.exit(3);
    a[4].split(" ").forEach(function (r) { o.common.firstAppByPowerOnReason[r] = a[3]; });
    fs.writeFileSync(a[2], JSON.stringify(o, null, 4) + "\n");
    JSON.parse(fs.readFileSync(a[2], "utf8"));
  ' "$STOCK" "$new" "$t" "$REASONS" 2>>"$LOG" || { rm -f "$new"; say "power-on: could not build the copy; nothing changed"; return 1; }
  # the sums reach storage before the copy is moved, so a power cut never leaves our copy without its sum
  sum "$new" > "$OVR_SUM" && sum "$STOCK" > "$STOCK_SUM" && sync && mv -f "$new" "$OVERRIDE" && sync \
    || { rm -f "$new"; say "power-on: writing the copy failed"; return 1; }
  say "power-on on ($t): written to $OVERRIDE. It starts working after a full restart (reboot)."
}

power_off () {
  rm -f "$OVERRIDE.lumen-new"
  if ours; then rm -f "$OVERRIDE" "$OVR_SUM" "$STOCK_SUM"; sync; say "power-on off: copy removed; the TV opens its last input from the next restart"
  elif [ -f "$OVERRIDE" ]; then say "power-on off: $OVERRIDE is not ours; left alone"
  else rm -f "$OVR_SUM" "$STOCK_SUM"; say "power-on off: nothing in place"; fi
}

do_guard () {
  # once per boot (/tmp is empty after a restart): note what bootd read, so Settings can say when a restart is still needed
  if [ ! -f "$LOADED" ]; then if ours; then override_app > "$LOADED"; else echo off > "$LOADED"; fi; fi
  # an empty copy (a power cut right after writing) is never valid
  if [ -f "$OVERRIDE" ] && [ ! -s "$OVERRIDE" ] && [ -f "$OVR_SUM" ]; then rm -f "$OVERRIDE" "$OVR_SUM" "$STOCK_SUM"; log "guard: empty power-on copy removed"; fi
  if ! any_lumen; then
    sleep 15   # a slow mount must not look like an uninstall
    if ! any_lumen; then
      log "guard: no Lumen build installed"
      ours && { rm -f "$OVERRIDE" "$OVR_SUM" "$STOCK_SUM"; sync; log "guard: power-on copy removed"; }
      { [ -f "$TAKEOVER" ] || mounted; } && native_off
      return 0
    fi
  fi
  # the mount does not outlive a restart: put the takeover back once the app manager answers for LG Home (about 20 s into a boot)
  if [ -f "$TAKEOVER" ] && ! mounted; then
    i=0; while [ $i -lt 60 ] && [ -z "$(sam_type)" ]; do sleep 2; i=$((i + 1)); done
    t=$(target "$(cat "$TAKEOVER")") || t=$(target)
    if [ -n "$t" ] && take "$t"; then log "guard: Home takeover mounted again ($t)"; else log "guard: Home takeover could not be mounted again"; fi
  fi
  if ours && [ "$(sum "$STOCK")" != "$(cat "$STOCK_SUM" 2>/dev/null)" ]; then
    rm -f "$OVERRIDE" "$OVR_SUM" "$STOCK_SUM"; sync
    log "guard: LG's $STOCK changed (firmware update?): power-on copy removed; run power-on on again"
  fi
}

case "$1" in
  check) do_check ;;
  status) do_status ;;
  state) do_state ;;
  native-home) case "$2" in on) native_on "$3" ;; off) native_off ;; *) echo "usage: native-home on [app-id] | off"; exit 2 ;; esac ;;
  power-on) case "$2" in on) power_on "$3" ;; off) power_off ;; *) echo "usage: power-on on [app-id] | off"; exit 2 ;; esac ;;
  guard) do_guard ;;
  *) sed -n '2,20p' "$0"; exit 2 ;;
esac
