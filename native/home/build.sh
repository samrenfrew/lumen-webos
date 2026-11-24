#!/bin/sh
set -e
HERE=$(cd "$(dirname "$0")" && pwd)
SDK=${WEBOS_SDK:-$HOME/toolchains/arm-webos-linux-gnueabi_sdk-buildroot}
SYSROOT=$SDK/arm-webos-linux-gnueabi/sysroot
PKG=$HERE/build/pkg
rm -rf "$PKG" && mkdir -p "$PKG"
"$SDK/bin/arm-webos-linux-gnueabi-gcc" -O2 -Wall -Wextra -D_REENTRANT -DLUMEN_WEBOS -I"$SYSROOT/usr/include/SDL2" \
  -o "$PKG/lumen-ntest" "$HERE"/src/*.c \
  -lSDL2 -lSDL2_image -lSDL2_ttf -lGLESv2 -lwayland-webos-client -lwayland-client -lm
"$SDK/bin/arm-webos-linux-gnueabi-strip" "$PKG/lumen-ntest"
cp "$HERE/appinfo.json" "$PKG/"
cp "$HERE/../../app/public-saver/icon.png" "$HERE/../../app/public-saver/largeIcon.png" "$PKG/"
sh "$HERE/assets.sh" "$PKG"
echo "Built: $PKG"
