#!/bin/sh
# Lumen on a TV in LG Developer Mode (not rooted): SSH as prisoner on port 9922, no scp on the TV.
# Settings come from .env: LUMEN_TV_IP, LUMEN_TV_KEY (a passphrase-free copy of the key the Developer Mode app serves; see README.md).
#
#   sh tools/devmode.sh deploy            back up Lumen's settings, build, install this build and start it
#   sh tools/devmode.sh home              put the apps and folders from tools/devmode/home.json on the home screen (settings backed up first)
#   sh tools/devmode.sh revert            install the release the TV had before the first deploy and put back the settings from then
#   sh tools/devmode.sh backup            save Lumen's settings from the TV into backup/
#   sh tools/devmode.sh restore <file>    put saved settings back
#   sh tools/devmode.sh find-apps         list the apps from tools/devmode/candidates.txt that the TV has
#   sh tools/devmode.sh run '<command>'   run a command on the TV
set -e
ROOT=$(cd "$(dirname "$0")/.." && pwd)
. "$ROOT/tools/env.sh"
TV_IP=${LUMEN_TV_IP:?Set LUMEN_TV_IP in .env}
TV_KEY=${LUMEN_TV_KEY:?Set LUMEN_TV_KEY in .env}
APP_ID=com.samrenfrew.lumen
BACKUP="$ROOT/backup"
# the release that was on the TV before the first deploy, and its version
ORIGINAL_IPK="$BACKUP/original.ipk"
RELEASES=https://github.com/samrenfrew/lumen-webos/releases/download
CDP_PORT=19998

SSH_OPTS="-o BatchMode=yes -o LogLevel=ERROR -o ConnectTimeout=10 -o HostKeyAlgorithms=+ssh-rsa -o PubkeyAcceptedAlgorithms=+ssh-rsa -i $TV_KEY -p 9922"
# luna-send only prints with a terminal, hence -tt
tv () { ssh $SSH_OPTS -tt "prisoner@$TV_IP" "$@"; }

# The DevTools tunnel the settings commands and tools/tv/ scripts use (127.0.0.1:19998 -> the TV's 9998)
tunnel () {
	curl -s -m 2 "http://127.0.0.1:$CDP_PORT/json" >/dev/null 2>&1 && return
	ssh $SSH_OPTS -o ExitOnForwardFailure=yes -f -N -L "$CDP_PORT:127.0.0.1:9998" "prisoner@$TV_IP"
}

launch () {
	tv "luna-send-pub -n 1 luna://com.webos.applicationManager/launch '{\"id\":\"$APP_ID\"}'" | tr -d '\r'
}

# Copies an .ipk to the TV over the SSH session's stdin (no scp there) and installs it with the developer install service.
install_ipk () {
	echo "Copying $(basename "$1") to the TV..."
	ssh $SSH_OPTS "prisoner@$TV_IP" 'mkdir -p /media/developer/temp && cat > /media/developer/temp/lumen.ipk' < "$1"
	echo "Installing..."
	tv "luna-send-pub -w 120000 -i luna://com.webos.appInstallService/dev/install '{\"id\":\"com.ares.defaultName\",\"ipkUrl\":\"/media/developer/temp/lumen.ipk\",\"subscribe\":true}' | grep -oE '\"state\":\"(installed|install failed)\"|\"errorText\":\"[^\"]*\"' | head -1; rm -f /media/developer/temp/lumen.ipk" | tr -d '\r' | tee "$BACKUP/.last-install"
	grep -q '"installed"' "$BACKUP/.last-install" || { echo "Install failed." >&2; exit 1; }
}

backup_config () {
	mkdir -p "$BACKUP"
	tunnel
	out="$BACKUP/settings-$(date +%Y%m%d-%H%M%S).json"
	# Lumen has to be open for its settings to be read: start it, give it a moment
	launch >/dev/null
	sleep 4
	node "$ROOT/tools/devmode/settings.mjs" get > "$out.part" || { rm -f "$out.part"; echo "Could not read Lumen's settings; nothing was changed." >&2; exit 1; }
	mv "$out.part" "$out"
	# the settings from before the first deploy are what revert puts back
	[ -f "$BACKUP/settings-original.json" ] || cp "$out" "$BACKUP/settings-original.json"
	echo "Settings saved: backup/$(basename "$out")"
}

# Before the first deploy: download the release the TV has now, so revert can put it back. A TV with no Lumen yet, or with a build
# that is not a release, needs an .ipk put at backup/original.ipk by hand.
keep_original () {
	mkdir -p "$BACKUP"
	[ -f "$ORIGINAL_IPK" ] && return
	have=$(tv "luna-send-pub -n 1 luna://com.webos.applicationManager/dev/listApps '{}'" | node -e "let t = ''; process.stdin.on('data', (d) => t += d).on('end', () => { const a = (JSON.parse(t).apps || []).find((x) => x.id === '$APP_ID'); process.stdout.write(a && /^[0-9.]+$/.test(a.version) ? a.version : ''); })")
	[ -n "$have" ] || { echo "Lumen is not installed yet, so there is nothing to revert to. Put an .ipk at backup/original.ipk to deploy anyway." >&2; exit 1; }
	echo "Saving Lumen $have, the version on the TV now, for revert..."
	curl -fsSL -o "$ORIGINAL_IPK.part" "$RELEASES/v$have/${APP_ID}_${have}_all.ipk" || { rm -f "$ORIGINAL_IPK.part"; echo "Lumen $have is not a published release: put its .ipk at backup/original.ipk" >&2; exit 1; }
	mv "$ORIGINAL_IPK.part" "$ORIGINAL_IPK"
	echo "$have" > "$BACKUP/original.version"
}

case "$1" in
	run) shift; tv "$@" | tr -d '\r' ;;
	find-apps)
		# Developer Mode may not list apps (listLaunchPoints is denied), but getAppLoadStatus is allowed: probe every candidate id,
		# print the ones the TV has. Ids are checked against a plain pattern before they reach the TV's shell.
		ids=$(grep -v '^#' "$ROOT/tools/devmode/candidates.txt" | grep -E '^[A-Za-z0-9._-]+$' | sort -u | tr '\n' ' ')
		tv "for id in $ids; do luna-send-pub -n 1 luna://com.webos.applicationManager/getAppLoadStatus \"{\\\"appId\\\":\\\"\$id\\\"}\" | grep -q '\"exist\":true' && echo \$id; done" | tr -d '\r'
		;;
	backup) backup_config ;;
	restore)
		[ -f "$2" ] || { echo "Usage: sh tools/devmode.sh restore <settings.json>" >&2; exit 1; }
		tunnel
		node "$ROOT/tools/devmode/settings.mjs" set "$2"
		;;
	deploy)
		keep_original
		backup_config
		LUMEN_APP_ID=$APP_ID LUMEN_APP_TITLE=Home sh "$ROOT/tools/build_ipk.sh"
		VERSION=$(sed -n 's/.*"version": *"\([^"]*\)".*/\1/p' "$ROOT/app/dist/appinfo.json" | head -1)
		install_ipk "$ROOT/build/${APP_ID}_${VERSION}_all.ipk"
		launch
		;;
	home)
		backup_config
		node "$ROOT/tools/devmode/settings.mjs" home "$ROOT/tools/devmode/home.json"
		;;
	revert)
		keep_original
		install_ipk "$ORIGINAL_IPK"
		launch
		if [ -f "$BACKUP/settings-original.json" ]; then
			sleep 6
			tunnel
			node "$ROOT/tools/devmode/settings.mjs" set "$BACKUP/settings-original.json"
		fi
		echo "Reverted to Lumen $(cat "$BACKUP/original.version" 2>/dev/null) with the settings from before the first deploy."
		;;
	*) sed -n '4,11p' "$0" | sed 's/^# *//' >&2; exit 1 ;;
esac
