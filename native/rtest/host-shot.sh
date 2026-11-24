#!/bin/sh
set -e
HERE=$(cd "$(dirname "$0")" && pwd)
OUT=${1:?usage: host-shot.sh <out.png> [row] [col] [mode] [slide]}
shift
IMAGE=$(sh "$HERE/image.sh")
mkdir -p "$HERE/build/host" "$HERE/build/cargo" "$HERE/build/target" "$(dirname "$OUT")"
sh "$HERE/../home/assets.sh" "$HERE/build/host"
docker run --rm -v "$HERE/../..:/src" -v "$(cd "$(dirname "$OUT")" && pwd):/shots" -w /src/native/rtest -u "$(id -u):$(id -g)" -e HOME=/tmp -e LUMEN_SHOT_ZOOM \
  -e CARGO_HOME=/src/native/rtest/build/cargo -e CARGO_TARGET_DIR=/src/native/rtest/build/target "$IMAGE" sh -c \
  "cargo build --release --locked >&2 && cp build/target/release/lumen-rtest build/host/ && cd build/host && LIBGL_ALWAYS_SOFTWARE=1 xvfb-run -a -s '-screen 0 1920x1080x24' ./lumen-rtest --shot /shots/$(basename "$OUT") $*; tail -8 /tmp/lumen-rtest-*/log"
