#!/bin/sh
set -e
HERE=$(cd "$(dirname "$0")" && pwd)
OUT=$HERE/build/shots
mkdir -p "$OUT"
failed=0
while read -r name keys; do
  [ -n "$name" ] || continue
  if [ "$1" = "--keep" ]; then
    sh "$HERE/host-shot.sh" "$OUT/$name.png" $keys && cp "$OUT/$name.png" "$HERE/shots/$name.png"
  else
    sh "$HERE/host-shot.sh" "$OUT/$name.png" --against "shots/$name.png" $keys || failed=1
  fi
done < "$HERE/shots/list"
exit $failed
