#!/bin/sh
set -e
HERE=$(cd "$(dirname "$0")" && pwd)
[ $# -ge 1 ] || { echo "usage: host-shot.sh <out.png> [--as lumen-guide|lumen-quick] [--at <ms>] [--against <kept.png, from native/lumen>] [left|right|up|down|back ...]" >&2; exit 2; }
OUT=$1
shift
PROGRAM=lumen
if [ "$1" = --as ]; then PROGRAM=$2; shift 2; fi
mkdir -p "$(dirname "$OUT")"
export LUMEN_SHOTS="$(cd "$(dirname "$OUT")" && pwd)"
HOST=$HERE/build/host
rm -rf "$HOST" && mkdir -p "$HOST/store"
LUMEN_PRIVATE=0 sh "$HERE/assets.sh" "$HOST"
cp "$HERE/fixtures/lumen-config.json" "$HERE/fixtures/weather-Leeds.json" "$HERE/fixtures/livetv.json" "$HOST/store/"
export LUMEN_CLOCK="${LUMEN_CLOCK-21:17|Tuesday November 24}"
sh "$HERE/host.sh" "cargo build --release --locked >&2 && cp build/host-target/release/lumen build/host/$PROGRAM && LUMEN_FIXTURES=fixtures LUMEN_STORE=build/host/store LIBGL_ALWAYS_SOFTWARE=1 xvfb-run -a -s '-screen 0 1920x1080x24' build/host/$PROGRAM --shot /shots/$(basename "$OUT") $*; code=\$?; tail -4 /tmp/lumen-native-*/log >&2; exit \$code"
