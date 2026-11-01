#!/bin/sh
# Turns off LG's ads, tracking and data collection on this TV. Runs on the TV as root, from /var/lib/homehook/; Lumen's Settings > Privacy
# runs it through the Homebrew Channel, and it works the same over SSH. How each part works, and where it comes from: docs/privacy.md.
#
#   sh privacy.sh state                  read only: key=value lines for Settings (lib/tvsetup.ts)
#   sh privacy.sh agreements off|restore withdraw LG's optional data agreements (screen recognition, ad personalisation, sharing,
#                                        diagnostics, marketing); restore puts back what was there before the first withdrawal
#   sh privacy.sh lmt on|off             Limit ad tracking (the TV's own setting)
#   sh privacy.sh liveplus on|off        Live Plus (LG's screen recognition setting)
#   sh privacy.sh hosts on|off           block LG's ad, tracking and screen recognition servers in /etc/hosts
#   sh privacy.sh services on|off        stop LG's logging, suggestion and screen recognition services (on = stopped)
#   sh privacy.sh updates on|off         block TV software updates (the Homebrew Channel's own flag; from the next restart)
#   sh privacy.sh boot                   run by 60-homehook at each boot: stops the services again while that is on
#
# Every change is logged to /tmp/homehook.log. Voice, the Content Store, app updates and LG's admanager are left alone (HBO Max and
# Disney+ wait for admanager at start: cleanyourtv-WebOS issue 47).
export PATH="/bin:/sbin:/usr/bin:/usr/sbin:$PATH"

HH="${HH:-/var/lib/homehook}"
LOG="${LOG:-/tmp/homehook.log}"
LUNA="${LUNA:-luna-send}"
SETTINGS=luna://com.webos.settingsservice
HOSTS="${HOSTS:-/etc/hosts}"
HOSTS_FILE="$HH/hosts-privacy"            # the blocked names; watch.sh keeps them in /etc/hosts
SERVICES_FLAG="$HH/privacy-services"      # services stopped at each boot
UPDATES_FLAG="${UPDATES_FLAG:-/var/luna/preferences/webosbrew_block_updates}"
EULA_BACKUP="$HH/eula.backup.json"
STUB="$HH/stub-noop"
TRANSIENT="${TRANSIENT:-/run/systemd/transient}"
SBIN="${SBIN:-/usr/sbin}"

# systemd units: masked under /run (memory only) and stopped. Luna on-demand services: a do-nothing stub is bound over the program, so the
# bus cannot start them (cleanyourtv-WebOS's method, claimed on webOS 25). Both are undone by a restart; the boot run puts them back.
UNITS="uploadd rdxd remotelogger service-logger contentminer nudge"
BINS="acr2 adoverlay-service sportsalarm"

# LG's ad, tracking and screen recognition (Alphonso) servers: Glasshouse's "ads and telemetry" tier (MIT), plus cleanyourtv-WebOS's.
# Leaves LG's platform reachable, so the Content Store and app updates keep working.
NAMES="ad.lgsmartad.com ibis.lgappstv.com ibs.lgappstv.com lgsmartad.com rdx.lgtvcommon.com aic.lgtvcommon.com smartad.lge.com
smartclip.com smartclip-services.com yumenetworks.com alphonso.tv prov-lg.alphonso.tv prov.alphonso.tv eulacheck.alphonso.tv
bl-server.alphonso.tv clockskew.alphonso.tv acrdb.alphonso.tv ads.alphonso.tv api.alphonso.tv tvads.alphonso.tv sync.alphonso.tv
tr.alphonso.tv tn.alphonso.tv insights.alphonso.tv bwlkup.alphonso.tv eu-acr86.alphonso.tv eic.ads.lgtvcommon.com"
# the firmware puts a region or country in front of these, and a hosts file only matches whole names
REGIONAL="info.lgsmartad.com ad.lgsmartad.com rdx2.lgtvsdp.com ibs.lgappstv.com ibsstat.lgappstv.com cdpbeacon.lgtvcommon.com
cdpsvc.lgtvcommon.com wau.lgtvcommon.com"
PREFIXES="aic eic kic eu gb uk nl"

# Agreement flags withdrawn by "agreements off" (Glasshouse's list, voice left on). The terms themselves, network use and first use are
# never touched; chpAllowed is LG Channels' channel list, which the TV accepts again at every start.
WITHDRAW="acrAllowed acrGdprAllowed acrAdAllowed acrOnAllowed customAdAllowed customadsAllowed cookiesAllowed thirdPartySharingAllowed
additionalDataAllowed remoteDiagAllowed marketingOnAllowed shoppingOnAllowed takeOnAllowed"
LOCKED="generalTermsAllowed networkAllowed firstUseAllowed allAllowed chpAllowed"
# agreements LG names only by document: data partners, Who.Where.What?, marketing
WITHDRAW_DOCS="S_DPA S_TAK S_MKT"
EULA_MAP="${EULA_MAP:-/var/palm/license/eulaInfoNetwork.json}"

log () { echo "$(date +%T) privacy: $*" >> "$LOG"; }
say () { echo "$*"; log "$*"; }
has_node () { command -v node >/dev/null 2>&1; }

get_setting () {   # $1 category ('' for none), $2 key: the value as printed (strings without quotes)
  if [ -n "$1" ]; then p="{\"category\":\"$1\",\"keys\":[\"$2\"]}"; else p="{\"keys\":[\"$2\"]}"; fi
  $LUNA -n 1 "$SETTINGS/getSystemSettings" "$p" 2>/dev/null | tr -d '\n' | sed -n "s/.*\"$2\": *\"\\{0,1\\}\\([^\",}]*\\)\"\\{0,1\\}.*/\\1/p"
}
set_setting () {   # $1 category, $2 key, $3 value (a plain word)
  r=$($LUNA -n 1 "$SETTINGS/setSystemSettings" "{\"category\":\"$1\",\"settings\":{\"$2\":\"$3\"}}" 2>&1)
  log "set $1.$2=$3: $r"
  case "$r" in *'"returnValue":true'*|*'"returnValue": true'*) return 0 ;; esac
  return 1
}

blocklist () {
  for n in $NAMES; do echo "$n"; done
  for n in $REGIONAL; do echo "$n"; for p in $PREFIXES; do echo "$p.$n"; done; done
}
hosts_lines () { blocklist | sort -u | while read -r n; do echo "0.0.0.0 $n"; echo ":: $n"; done; }

# ---- agreements -----------------------------------------------------------------------------------------------------------------

# Reads eulaStatus and eulaInfoNetwork into $T, for node.
read_eula () {
  T=$(mktemp -d /tmp/lumen-privacy.XXXXXX) || return 1
  $LUNA -n 1 "$SETTINGS/getSystemSettings" '{"keys":["eulaStatus"]}' > "$T/status.json" 2>/dev/null
  $LUNA -n 1 "$SETTINGS/getSystemSettings" '{"keys":["eulaInfoNetwork"]}' > "$T/docs.json" 2>/dev/null
}

# Withdraws each flag in WITHDRAW (and each flag LG maps to a document in WITHDRAW_DOCS), the way Glasshouse's planConsent does: a flag's
# agreement documents are marked not accepted, unless a locked flag (the terms themselves) still needs them, and every other flag whose
# documents are no longer all accepted goes off with it. Prints the setSystemSettings parameters, or nothing when nothing would change.
# ES5 only: the TV's node is old.
plan_withdraw () {
  node -e '
    var fs = require("fs"), a = process.argv;
    function read (f) { try { return JSON.parse(fs.readFileSync(f, "utf8")); } catch (e) { return null; } }
    var st = read(a[1]), dc = read(a[2]), map = read(a[3]);
    var flags = st && st.settings && st.settings.eulaStatus, eln = dc && dc.settings && dc.settings.eulaInfoNetwork;
    if (!flags || typeof flags !== "object" || !eln || !(eln.eulaList instanceof Array)) { console.error("no-read"); process.exit(3); }
    var withdraw = a[4].split(/\s+/), locked = a[5].split(/\s+/), byDoc = a[6].split(/\s+/);
    var groups = {}, list = (map && map.eulaMappingList && map.eulaMappingList.eulaInfo) || [];
    list.forEach(function (e) { if (e && e.settingKey) groups[e.settingKey] = (e.mandatory || e.generalSelectAll || []).slice(); });
    Object.keys(groups).forEach(function (k) {
      if (locked.indexOf(k) === -1 && withdraw.indexOf(k) === -1 && groups[k].some(function (d) { return byDoc.indexOf(d) !== -1; })) withdraw.push(k);
    });
    var accepted = {};
    eln.eulaList.forEach(function (d) { if (d.accepted) accepted[d.id] = true; });
    var keep = {};
    locked.forEach(function (k) { if (flags[k] && groups[k]) groups[k].forEach(function (d) { keep[d] = true; }); });
    var next = {}, changed = [];
    Object.keys(flags).forEach(function (k) { next[k] = flags[k]; });
    withdraw.forEach(function (k) {
      if (!next.hasOwnProperty(k) || locked.indexOf(k) !== -1) return;
      (groups[k] || []).forEach(function (d) { if (!keep[d]) delete accepted[d]; });
      if (next[k] === true) { next[k] = false; changed.push(k); }
    });
    Object.keys(groups).forEach(function (k) {
      if (locked.indexOf(k) !== -1 || next[k] !== true || !groups[k].length) return;
      if (!groups[k].every(function (d) { return accepted[d]; })) { next[k] = false; changed.push(k); }
    });
    if (!changed.length) process.exit(0);
    var docs = JSON.parse(JSON.stringify(eln));
    docs.eulaList.forEach(function (d) { d.accepted = !!accepted[d.id]; });
    console.error("withdrawing: " + changed.join(" "));
    console.log(JSON.stringify({settings: {eulaInfoNetwork: docs, eulaStatus: next}}));
  ' "$T/status.json" "$T/docs.json" "$EULA_MAP" "$WITHDRAW" "$LOCKED" "$WITHDRAW_DOCS"
}

# How many withdrawable flags are on now ('' when unreadable).
agreements_on () {
  has_node || return 0
  read_eula || return 0
  node -e '
    var fs = require("fs"), a = process.argv;
    try { var f = JSON.parse(fs.readFileSync(a[1], "utf8")).settings.eulaStatus; } catch (e) { process.exit(0); }
    if (!f) process.exit(0);
    console.log(a[2].split(/\s+/).filter(function (k) { return f[k] === true; }).length);
  ' "$T/status.json" "$WITHDRAW" 2>/dev/null
  rm -rf "$T"
}

agreements_off () {
  has_node || { say "agreements: no node on this TV; nothing changed"; return 1; }
  read_eula || { say "agreements: no temporary folder"; return 1; }
  params=$(plan_withdraw 2>>"$LOG"); rc=$?
  if [ $rc -ne 0 ]; then rm -rf "$T"; say "agreements: could not read them; nothing changed"; return 1; fi
  if [ -z "$params" ]; then rm -rf "$T"; say "agreements: already withdrawn"; return 0; fi
  # the first time only: what was there, for "restore"
  if [ ! -f "$EULA_BACKUP" ]; then
    node -e '
      var fs = require("fs"), a = process.argv;
      var s = JSON.parse(fs.readFileSync(a[1], "utf8")).settings, d = JSON.parse(fs.readFileSync(a[2], "utf8")).settings;
      fs.writeFileSync(a[3], JSON.stringify({settings: {eulaInfoNetwork: d.eulaInfoNetwork, eulaStatus: s.eulaStatus}}));
    ' "$T/status.json" "$T/docs.json" "$EULA_BACKUP" 2>>"$LOG" || { rm -rf "$T"; say "agreements: could not save a backup; nothing changed"; return 1; }
  fi
  rm -rf "$T"
  r=$($LUNA -n 1 "$SETTINGS/setSystemSettings" "$params" 2>&1)
  log "agreements off: $r"
  left=$(agreements_on)
  if [ "$left" = 0 ]; then say "agreements: withdrawn"; else say "agreements: the TV left ${left:-some} on ($r)"; return 1; fi
}

agreements_restore () {
  [ -s "$EULA_BACKUP" ] || { say "agreements: no backup to restore"; return 1; }
  r=$($LUNA -n 1 "$SETTINGS/setSystemSettings" "$(cat "$EULA_BACKUP")" 2>&1)
  say "agreements restore: $r"
  case "$r" in *'"returnValue":true'*|*'"returnValue": true'*) rm -f "$EULA_BACKUP"; return 0 ;; esac
  return 1
}

# ---- hosts ------------------------------------------------------------------------------------------------------------------------

hosts_on () {
  hosts_lines > "$HOSTS_FILE.tmp" && mv -f "$HOSTS_FILE.tmp" "$HOSTS_FILE" || { say "hosts: could not write $HOSTS_FILE"; return 1; }
  # at once; watch.sh keeps them there (the Homebrew Channel rewrites /etc/hosts at boot)
  if [ -w "$HOSTS" ]; then
    while IFS= read -r l; do grep -qxF "$l" "$HOSTS" || echo "$l" >> "$HOSTS"; done < "$HOSTS_FILE"
    say "hosts: $(grep -c . "$HOSTS_FILE") lines blocked"
  else
    say "hosts: $HOSTS cannot be written on this TV; the list is saved but nothing is blocked"; return 1
  fi
}

hosts_off () {
  if [ -f "$HOSTS_FILE" ] && [ -w "$HOSTS" ]; then
    grep -vxF -f "$HOSTS_FILE" "$HOSTS" > "$HOSTS.lumen-tmp" && cat "$HOSTS.lumen-tmp" > "$HOSTS"; rm -f "$HOSTS.lumen-tmp"
  fi
  rm -f "$HOSTS_FILE"
  say "hosts: unblocked"
}

# ---- services ---------------------------------------------------------------------------------------------------------------------

services_apply () {
  if command -v systemctl >/dev/null 2>&1; then
    mkdir -p "$TRANSIENT"
    for u in $UNITS; do ln -sf /dev/null "$TRANSIENT/$u.service"; done
    systemctl daemon-reload 2>/dev/null
    # --no-block: contentminer can take a minute to stop
    for u in $UNITS; do systemctl --no-block stop "$u.service" 2>/dev/null; done
  fi
  printf '#!/bin/sh\nexit 0\n' > "$STUB" && chmod +x "$STUB"
  for b in $BINS; do
    [ -e "$SBIN/$b" ] || continue
    grep -q " $SBIN/$b " /proc/mounts || mount --bind "$STUB" "$SBIN/$b"
    pkill -9 -f "^$SBIN/$b" 2>/dev/null
  done
}

services_on () {
  : > "$SERVICES_FLAG"
  services_apply
  say "services: stopped ($(services_running) still running)"
}

services_off () {
  rm -f "$SERVICES_FLAG"
  for u in $UNITS; do [ "$(readlink "$TRANSIENT/$u.service" 2>/dev/null)" = /dev/null ] && rm -f "$TRANSIENT/$u.service"; done
  command -v systemctl >/dev/null 2>&1 && systemctl daemon-reload 2>/dev/null
  for b in $BINS; do grep -q " $SBIN/$b " /proc/mounts && umount "$SBIN/$b" 2>/dev/null; done
  say "services: allowed again (they start when the TV asks for them, or at the next restart)"
}

services_running () {   # how many of them are running now (one read of every process name; the kernel cuts names to 15 characters)
  comms=$(cat /proc/[0-9]*/comm 2>/dev/null)
  n=0
  for name in $UNITS $BINS; do
    printf '%s\n' "$comms" | grep -qxF "$(printf '%.15s' "$name")" && n=$((n + 1))
  done
  echo $n
}

# ---- state ------------------------------------------------------------------------------------------------------------------------

# key=value lines: agreements (how many withdrawable flags are on; '' unreadable), backup (yes when restore can put them back),
# lmt and liveplus (on, off, or '' when the TV has no such setting), hosts (on/off), hosts_writable, services (on/off),
# services_running (how many still run), updates (on/off), node (yes/no)
do_state () {
  echo "agreements=$(agreements_on)"
  [ -s "$EULA_BACKUP" ] && echo backup=yes || echo backup=no
  echo "lmt=$(get_setting general lmt)"
  echo "liveplus=$(get_setting option livePlus)"
  [ -f "$HOSTS_FILE" ] && echo hosts=on || echo hosts=off
  [ -w "$HOSTS" ] && echo hosts_writable=yes || echo hosts_writable=no
  [ -f "$SERVICES_FLAG" ] && echo services=on || echo services=off
  echo "services_running=$(services_running)"
  [ -f "$UPDATES_FLAG" ] && echo updates=on || echo updates=off
  has_node && echo node=yes || echo node=no
}

onoff () { case "$1" in on|off) return 0 ;; esac; echo "usage: $0 $2 on|off"; exit 2; }

case "$1" in
  state) do_state ;;
  agreements) case "$2" in off) agreements_off ;; restore) agreements_restore ;; *) echo "usage: agreements off|restore"; exit 2 ;; esac ;;
  lmt) onoff "$2" lmt; set_setting general lmt "$2" && say "lmt: $2" || { say "lmt: the TV refused"; exit 1; } ;;
  liveplus) onoff "$2" liveplus; set_setting option livePlus "$2" && say "liveplus: $2" || { say "liveplus: the TV refused"; exit 1; } ;;
  hosts) onoff "$2" hosts; if [ "$2" = on ]; then hosts_on; else hosts_off; fi ;;
  services) onoff "$2" services; if [ "$2" = on ]; then services_on; else services_off; fi ;;
  updates) onoff "$2" updates; if [ "$2" = on ]; then : > "$UPDATES_FLAG" && say "updates: blocked from the next restart"; else rm -f "$UPDATES_FLAG"; say "updates: allowed from the next restart"; fi ;;
  boot) [ -f "$SERVICES_FLAG" ] && { services_apply; log "boot: services stopped again"; } ;;
  *) sed -n '2,16p' "$0"; exit 2 ;;
esac
