#!/bin/sh
set -e
HERE=$(cd "$(dirname "$0")" && pwd)
SDK=${WEBOS_SDK:-$HOME/toolchains/arm-webos-linux-gnueabi_sdk-buildroot}
TARGET=armv7-unknown-linux-gnueabi
PKG=$HERE/build/pkg
rm -rf "$PKG" && mkdir -p "$PKG"
export CC_armv7_unknown_linux_gnueabi="$SDK/bin/arm-webos-linux-gnueabi-gcc"
export AR_armv7_unknown_linux_gnueabi="$SDK/bin/arm-webos-linux-gnueabi-ar"
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABI_LINKER="$SDK/bin/arm-webos-linux-gnueabi-gcc"
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABI_RUSTFLAGS="${LUMEN_RUSTFLAGS--C target-feature=+vfp3,+neon,-soft-float}"
export LUMEN_SYSROOT="$SDK/arm-webos-linux-gnueabi/sysroot"
export LUMEN_SDL_INCLUDE="$LUMEN_SYSROOT/usr/include/SDL2"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HERE/build/target}"
cd "$HERE"
cargo build --release --locked --target $TARGET --features webos
cp "$CARGO_TARGET_DIR/$TARGET/release/lumen-rtest" "$PKG/"
cp "$HERE/appinfo.json" "$PKG/"
cp "$HERE/../../app/public-saver/icon.png" "$HERE/../../app/public-saver/largeIcon.png" "$PKG/"
sh "$HERE/../home/assets.sh" "$PKG"
echo "Built: $PKG"
