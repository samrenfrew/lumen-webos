#!/bin/sh
# Builds Lumen Native (native/lumen/, docs/plans/native-rewrite.md), and packages it: prints the .ipk's path.
# With cargo on the PATH and glibc 2.36 or newer (the SDK's host tools need it), build.sh runs here. Otherwise it runs in the
# lumen-native image (native/lumen/Dockerfile), which has the Rust in native/lumen/rust-toolchain.toml.
# Env: WEBOS_SDK (the SDK folder), LUMEN_PRIVATE=0 (no private font), LUMEN_BUILD_DIR (where the .ipk goes), LUMEN_DEV=0 (no control channel).
set -e
ROOT=$(cd "$(dirname "$0")/.." && pwd)
SDK=${WEBOS_SDK:-$HOME/toolchains/arm-webos-linux-gnueabi_sdk-buildroot}
[ -x "$SDK/bin/arm-webos-linux-gnueabi-gcc" ] || { echo "The webOS SDK is not at $SDK. Unpack openlgtv/buildroot-nc4's arm-webos-linux-gnueabi_sdk-buildroot there and run relocate-sdk.sh, or set WEBOS_SDK." >&2; exit 1; }
OUT=${LUMEN_BUILD_DIR:-$ROOT/native/lumen/build}
HOST_GLIBC=$(ldd --version 2>&1 | head -1 | grep -oE '[0-9]+\.[0-9]+$' || echo 0)
if command -v cargo >/dev/null && [ "$(printf '%s\n2.36\n' "$HOST_GLIBC" | sort -V | head -1)" = 2.36 ]; then
  sh "$ROOT/native/lumen/build.sh" >&2
else
  IMAGE=$(sh "$ROOT/native/lumen/image.sh")
  mkdir -p "$ROOT/native/lumen/build/cargo"
  docker run --rm -v "$SDK:$SDK:ro" -v "$ROOT:/src" -w /src/native/lumen -e HOME="$HOME" -e WEBOS_SDK="$SDK" -e LUMEN_PRIVATE -e LUMEN_RUSTFLAGS -e LUMEN_DEV \
    -e CARGO_HOME=/src/native/lumen/build/cargo -u "$(id -u):$(id -g)" "$IMAGE" sh build.sh >&2
fi
cd "$ROOT/app"
[ -d node_modules ] || npm ci --no-audit --no-fund >&2
mkdir -p "$OUT"
rm -f "$OUT/com.samrenfrew.lumen.native_"*.ipk
npx ares-package "$ROOT/native/lumen/build/pkg" --no-minify -o "$OUT" >&2
ls "$OUT/com.samrenfrew.lumen.native_"*.ipk
