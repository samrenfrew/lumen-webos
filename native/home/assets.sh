#!/bin/sh
set -e
HERE=$(cd "$(dirname "$0")" && pwd)
PKG=$1
APP=$HERE/../../app
cp "$APP/public/wallpapers/default-0.jpg" "$PKG/wallpaper.jpg"
mkdir -p "$PKG/icons"
i=10
for name in bbc.iplayer.3.0 youtube.leanback.v4 cdp-30 com.fvp.ch4 com.fvp.itv demand5 com.apple.appletv amazon now.tv bbc.sounds.1.0 tv.twitch.tv.starshot.lg com.fvp.uktv org.webosbrew.hbchannel com.lennylxx.iptv com.webos.app.browser com.webos.app.mediadiscovery com.webos.app.igallery amazon.alexa.view; do
  cp "$APP/public/icons/apps/$name.png" "$PKG/icons/$i.png"
  i=$((i + 1))
done
PRIVATE_FONTS=$APP/private/fonts
if [ "$LUMEN_PRIVATE" != 0 ] && [ -f "$PRIVATE_FONTS/sf-500.ttf" ]; then mkdir -p "$PKG/fonts" && cp "$PRIVATE_FONTS/sf-500.ttf" "$PKG/fonts/"; fi
