#!/bin/sh
set -e
HERE=$(cd "$(dirname "$0")" && pwd)
SDK=${WEBOS_SDK:-$HOME/toolchains/arm-webos-linux-gnueabi_sdk-buildroot}
SYSROOT=$SDK/arm-webos-linux-gnueabi/sysroot
PKG=$HERE/build/pkg
rm -rf "$PKG" && mkdir -p "$PKG"
"$SDK/bin/arm-webos-linux-gnueabi-gcc" -O2 -Wall -Wextra -D_REENTRANT -I"$SYSROOT/usr/include/SDL2" \
  -o "$PKG/lumen-saver" "$HERE"/src/*.c "$HERE/vendor/cJSON.c" \
  -lSDL2 -lSDL2_image -lSDL2_ttf -lcurl -lwayland-webos-client -lwayland-client -lm
"$SDK/bin/arm-webos-linux-gnueabi-strip" "$PKG/lumen-saver"
cp "$HERE/appinfo.json" "$PKG/"
cp -r "$HERE/assets" "$PKG/assets"
cp "$HERE/../../app/public/icon.png" "$PKG/"
echo "Built: $PKG"
