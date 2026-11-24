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
if [ -n "$LUMEN_SAVER_VERSION" ]; then sed "s/\"version\": *\"[^\"]*\"/\"version\": \"$LUMEN_SAVER_VERSION\"/" "$HERE/appinfo.json" > "$PKG/appinfo.json"; else cp "$HERE/appinfo.json" "$PKG/"; fi
cp -r "$HERE/assets" "$PKG/assets"
cp "$HERE/../../app/public-saver/icon.png" "$HERE/../../app/public-saver/largeIcon.png" "$PKG/"
mkdir -p "$PKG/licenses" && cp "$HERE/../../app/LICENSE" "$PKG/licenses/LICENSE.txt"
PRIVATE_FONTS=$HERE/../../app/private/fonts
if [ "$LUMEN_PRIVATE" != 0 ] && ls "$PRIVATE_FONTS"/sf-*.ttf >/dev/null 2>&1; then mkdir -p "$PKG/fonts" && cp "$PRIVATE_FONTS"/sf-*.ttf "$PKG/fonts/"; fi
echo "Built: $PKG"
