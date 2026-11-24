#!/bin/sh
set -e
HERE=$(cd "$(dirname "$0")" && pwd)
IMAGE=$(sh "$HERE/image.sh")
mkdir -p "$HERE/build/cargo" "$HERE/build/host-target" "${LUMEN_SHOTS:=$HERE/build/shots}"
docker run --rm -v "$HERE/../..:/src" -v "$LUMEN_SHOTS:/shots" -w /src/native/lumen -u "$(id -u):$(id -g)" -e HOME=/tmp -e LUMEN_CLOCK -e LUMEN_PRIVATE \
  -e CARGO_HOME=/src/native/lumen/build/cargo -e CARGO_TARGET_DIR=/src/native/lumen/build/host-target "$IMAGE" sh -c "$1"
