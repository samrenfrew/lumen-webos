#!/bin/sh
# Builds Lumen Screensaver, the native one (native/saver/, docs/plans/native-saver.md), and packages it: prints the .ipk's path.
# The SDK's host tools need glibc 2.36 or newer, so on an older host build.sh runs inside debian:bookworm-slim.
# Env: WEBOS_SDK (the SDK folder), LUMEN_PRIVATE=0 (no private font), LUMEN_SAVER_VERSION (overrides appinfo.json's), LUMEN_BUILD_DIR (where the .ipk goes).
set -e
ROOT=$(cd "$(dirname "$0")/.." && pwd)
SDK=${WEBOS_SDK:-$HOME/toolchains/arm-webos-linux-gnueabi_sdk-buildroot}
[ -x "$SDK/bin/arm-webos-linux-gnueabi-gcc" ] || { echo "The webOS SDK is not at $SDK. Unpack openlgtv/buildroot-nc4's arm-webos-linux-gnueabi_sdk-buildroot there and run relocate-sdk.sh, or set WEBOS_SDK." >&2; exit 1; }
OUT=${LUMEN_BUILD_DIR:-$ROOT/native/saver/build}
HOST_GLIBC=$(ldd --version 2>&1 | head -1 | grep -oE '[0-9]+\.[0-9]+$' || echo 0)
if [ "$(printf '%s\n2.36\n' "$HOST_GLIBC" | sort -V | head -1)" = 2.36 ]; then
  sh "$ROOT/native/saver/build.sh" >&2
else
  docker run --rm -v "$SDK:$SDK:ro" -v "$ROOT:/src" -w /src/native/saver -e HOME="$HOME" -e WEBOS_SDK="$SDK" -e LUMEN_PRIVATE -e LUMEN_SAVER_VERSION \
    -u "$(id -u):$(id -g)" debian:bookworm-slim sh build.sh >&2
fi
cd "$ROOT/app"
[ -d node_modules ] || npm ci --no-audit --no-fund >&2
mkdir -p "$OUT"
rm -f "$OUT/com.samrenfrew.lumen.native.saver_"*.ipk
npx ares-package "$ROOT/native/saver/build/pkg" --no-minify -o "$OUT" >&2
ls "$OUT/com.samrenfrew.lumen.native.saver_"*.ipk
