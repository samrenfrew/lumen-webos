#!/bin/sh
set -e
HERE=$(cd "$(dirname "$0")" && pwd)
PKG=$1
APP=$HERE/../../app
mkdir -p "$PKG/assets" "$PKG/icons"
cp "$APP/public/wallpapers/default-0.jpg" "$PKG/wallpaper.jpg"
cp -r "$APP/public/icons/apps" "$PKG/icons/apps"
cp -r "$APP/public/icons/inputs" "$PKG/icons/inputs"
cp "$HERE/assets/catalog.tsv" "$PKG/assets/"
cp "$HERE/assets/wallpapers.tsv" "$HERE/assets/cities.tsv" "$PKG/assets/"
mkdir -p "$PKG/licenses" && cp "$APP/LICENSE" "$PKG/licenses/LICENSE.txt"
PRIVATE_FONTS=$APP/private/fonts
if [ "$LUMEN_PRIVATE" != 0 ] && ls "$PRIVATE_FONTS"/sf-*.ttf >/dev/null 2>&1; then mkdir -p "$PKG/fonts" && cp "$PRIVATE_FONTS"/sf-*.ttf "$PKG/fonts/"; fi
