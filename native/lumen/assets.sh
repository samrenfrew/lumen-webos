#!/bin/sh
set -e
HERE=$(cd "$(dirname "$0")" && pwd)
PKG=$1
APP=$HERE/../../app
mkdir -p "$PKG/assets"
[ "$2" = side ] || cp "$APP/public/wallpapers/default-0.jpg" "$PKG/wallpaper.jpg"
if [ -z "$2" ] || [ "$2" = overlay ]; then
  mkdir -p "$PKG/icons" && cp -r "$APP/public/icons/apps" "$PKG/icons/apps"
  cp "$HERE/assets/catalog.tsv" "$PKG/assets/"
fi
if [ -z "$2" ]; then
  cp -r "$APP/public/icons/inputs" "$PKG/icons/inputs"
  mkdir -p "$PKG/tv" && for f in watch.sh stadia.sh resume.sh home-setup.sh privacy.sh gear.sh home.sh 60-homehook hosts-extra; do cp "$HERE/../../tools/tv/$f" "$PKG/tv/"; done
fi
[ "$2" = side ] || cp "$HERE/assets/wallpapers.tsv" "$HERE/assets/cities.tsv" "$PKG/assets/"
mkdir -p "$PKG/licenses" && cp "$APP/LICENSE" "$PKG/licenses/LICENSE.txt"
PRIVATE_FONTS=$APP/private/fonts
if [ "$LUMEN_PRIVATE" != 0 ] && ls "$PRIVATE_FONTS"/sf-*.ttf >/dev/null 2>&1; then mkdir -p "$PKG/fonts" && cp "$PRIVATE_FONTS"/sf-*.ttf "$PKG/fonts/"; fi
