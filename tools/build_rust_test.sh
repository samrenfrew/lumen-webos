#!/bin/sh
# Builds Lumen Rust Test (native/rtest/, docs/research/spikes.md "Rust"), and packages it: prints the .ipk's path.
# With cargo on the PATH and glibc 2.36 or newer (the SDK's host tools need it), build.sh runs here. Otherwise it runs in the
# lumen-rtest image (native/rtest/Dockerfile), which has the Rust in native/rtest/rust-toolchain.toml.
# Env: WEBOS_SDK (the SDK folder), LUMEN_PRIVATE=0 (no private font), LUMEN_BUILD_DIR (where the .ipk goes).
set -e
ROOT=$(cd "$(dirname "$0")/.." && pwd)
SDK=${WEBOS_SDK:-$HOME/toolchains/arm-webos-linux-gnueabi_sdk-buildroot}
[ -x "$SDK/bin/arm-webos-linux-gnueabi-gcc" ] || { echo "The webOS SDK is not at $SDK. Unpack openlgtv/buildroot-nc4's arm-webos-linux-gnueabi_sdk-buildroot there and run relocate-sdk.sh, or set WEBOS_SDK." >&2; exit 1; }
OUT=${LUMEN_BUILD_DIR:-$ROOT/native/rtest/build}
HOST_GLIBC=$(ldd --version 2>&1 | head -1 | grep -oE '[0-9]+\.[0-9]+$' || echo 0)
if command -v cargo >/dev/null && [ "$(printf '%s\n2.36\n' "$HOST_GLIBC" | sort -V | head -1)" = 2.36 ]; then
  sh "$ROOT/native/rtest/build.sh" >&2
else
  IMAGE=$(sh "$ROOT/native/rtest/image.sh")
  mkdir -p "$ROOT/native/rtest/build/cargo"
  docker run --rm -v "$SDK:$SDK:ro" -v "$ROOT:/src" -w /src/native/rtest -e HOME="$HOME" -e WEBOS_SDK="$SDK" -e LUMEN_PRIVATE -e LUMEN_RUSTFLAGS \
    -e CARGO_HOME=/src/native/rtest/build/cargo -u "$(id -u):$(id -g)" "$IMAGE" sh build.sh >&2
fi
cd "$ROOT/app"
[ -d node_modules ] || npm ci --no-audit --no-fund >&2
mkdir -p "$OUT"
rm -f "$OUT/com.samrenfrew.lumen.rtest_"*.ipk
npx ares-package "$ROOT/native/rtest/build/pkg" --no-minify -o "$OUT" >&2
ls "$OUT/com.samrenfrew.lumen.rtest_"*.ipk
