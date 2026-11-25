#!/bin/sh
set -e
HERE=$(cd "$(dirname "$0")" && pwd)
OUT=${1:?usage: host-shot.sh <out.png> [row] [col] [mode] [slide]}
shift
IMAGE=lumen-native-host
docker image inspect $IMAGE >/dev/null 2>&1 || docker build -t $IMAGE - <<'DOCKER'
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends gcc libc6-dev libsdl2-dev libsdl2-image-dev libsdl2-ttf-dev libgles2-mesa-dev libgl1-mesa-dri xvfb xauth fonts-dejavu-core && rm -rf /var/lib/apt/lists/*
DOCKER
mkdir -p "$HERE/build/host" "$(dirname "$OUT")"
sh "$HERE/assets.sh" "$HERE/build/host"
docker run --rm -v "$HERE/../..:/src:ro" -v "$HERE/build/host:/out" -v "$(cd "$(dirname "$OUT")" && pwd):/shots" -u "$(id -u):$(id -g)" -e HOME=/tmp -e LUMEN_SHOT_ZOOM $IMAGE sh -c \
  "gcc -O2 -Wall -Wextra -D_REENTRANT -I/usr/include/SDL2 -o /out/lumen-ntest /src/native/home/src/*.c -lSDL2 -lSDL2_image -lSDL2_ttf -lGLESv2 -lm && cd /out && LIBGL_ALWAYS_SOFTWARE=1 xvfb-run -a -s '-screen 0 1920x1080x24' ./lumen-ntest --shot /shots/$(basename "$OUT") $*; tail -8 /tmp/lumen-ntest-*/log"
