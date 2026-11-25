#!/bin/sh
set -e
HERE=$(cd "$(dirname "$0")" && pwd)
RUST=$(sed -n 's/^channel = "\(.*\)"/\1/p' "$HERE/rust-toolchain.toml")
IMAGE=lumen-native:$RUST-$(cksum < "$HERE/Dockerfile" | cut -d" " -f1)
docker image inspect "$IMAGE" >/dev/null 2>&1 || docker build -t "$IMAGE" --build-arg RUST="$RUST" - < "$HERE/Dockerfile" >&2
echo "$IMAGE"
