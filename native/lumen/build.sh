#!/bin/sh
set -e
HERE=$(cd "$(dirname "$0")" && pwd)
SDK=${WEBOS_SDK:-$HOME/toolchains/arm-webos-linux-gnueabi_sdk-buildroot}
TARGET=armv7-unknown-linux-gnueabi
PKG=$HERE/build/pkg
SAVER_PKG=$HERE/build/pkg-saver
rm -rf "$PKG" "$SAVER_PKG" "$HERE/build/pkg-guide" "$HERE/build/pkg-quick" "$HERE/build/pkg-overlay" && mkdir -p "$PKG" "$SAVER_PKG"
export CC_armv7_unknown_linux_gnueabi="$SDK/bin/arm-webos-linux-gnueabi-gcc"
export AR_armv7_unknown_linux_gnueabi="$SDK/bin/arm-webos-linux-gnueabi-ar"
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABI_LINKER="$SDK/bin/arm-webos-linux-gnueabi-gcc"
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABI_RUSTFLAGS="${LUMEN_RUSTFLAGS--C target-feature=+vfp3,+neon,-soft-float}"
export LUMEN_SYSROOT="$SDK/arm-webos-linux-gnueabi/sysroot"
export LUMEN_SDL_INCLUDE="$LUMEN_SYSROOT/usr/include/SDL2"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HERE/build/target}"
FEATURES=webos
[ "$LUMEN_DEV" = 0 ] || FEATURES=webos,dev
cd "$HERE"
cargo build --release --locked --target $TARGET --features "$FEATURES"
cp "$CARGO_TARGET_DIR/$TARGET/release/lumen" "$PKG/"
cp "$HERE/appinfo.json" "$PKG/"
cp "$HERE/../../app/public/icon.png" "$HERE/../../app/public/largeIcon.png" "$PKG/"
sh "$HERE/assets.sh" "$PKG"
cp "$PKG/lumen" "$SAVER_PKG/lumen-saver"
cp "$HERE/appinfo-saver.json" "$SAVER_PKG/appinfo.json"
cp "$PKG/icon.png" "$PKG/largeIcon.png" "$SAVER_PKG/"
sh "$HERE/assets.sh" "$SAVER_PKG" saver
for SIDE in guide quick; do
  SIDE_PKG=$HERE/build/pkg-$SIDE
  mkdir -p "$SIDE_PKG"
  cp "$PKG/lumen" "$SIDE_PKG/lumen-$SIDE"
  cp "$HERE/appinfo-$SIDE.json" "$SIDE_PKG/appinfo.json"
  cp "$HERE/../../app/public-$SIDE/icon.png" "$HERE/../../app/public-$SIDE/largeIcon.png" "$SIDE_PKG/"
  sh "$HERE/assets.sh" "$SIDE_PKG" side
done
OVERLAY_PKG=$HERE/build/pkg-overlay
mkdir -p "$OVERLAY_PKG"
cp "$PKG/lumen" "$OVERLAY_PKG/lumen-overlay"
cp "$HERE/appinfo-overlay.json" "$OVERLAY_PKG/appinfo.json"
cp "$PKG/icon.png" "$PKG/largeIcon.png" "$OVERLAY_PKG/"
sh "$HERE/assets.sh" "$OVERLAY_PKG" saver
echo "Built: $PKG $SAVER_PKG $HERE/build/pkg-guide $HERE/build/pkg-quick $OVERLAY_PKG"
