# tvOS UI + Liquid Glass research, and what it means for this TV

Sections 1 to 4 were written for the original project's TV: an LG UQ80 (65UQ8040PSB), webOS 22, alpha 5 Gen5 chip, 1.5 GB RAM, Chromium 87.
Section 5 was measured on this fork's TV: an LG 43UT91006LA, webOS 25, Chromium 120. Treat the decisions in section 4 as assumptions until they are tested on that TV.
More measurements from the webOS 25 TV are in `docs/research/frame-rate.md`, `docs/research/down-up-lag-review.md` and `docs/research/startup-perf.md`.

## 1. tvOS design rules (what the launcher copies)
- Icons are 400x240 pt (800x480 px @2x), 5:3 landscape; the system adds the rounded corners, rim light and shadow.
  Back layer opaque brand colour, logo layers on top. Focused size is about 1.23x the resting size (300x180 -> 370x222).
- Focus is shown by scale (1.05x-1.1x), a lifted shadow, a white glare sweeping across the surface, and (on Apple TV) parallax tilt.
  Focusing animations should be prominent, unfocusing ones subtler. Colour is never the only focus cue.
- Safe area: 60 pt top/bottom, 80 pt left/right. Prefer one row of 5-7 cards over dense grids. Minimum card target 250x150 pt.
- Type: body 29 pt minimum (31-35 recommended), titles 48+ pt, medium/semibold weight, no hairlines. Viewing distance is about 10 ft.
- Home: a top row of apps with the focused app's name shown under it; no cursor; Menu/Back always goes up one level.
- Avoid: dense text, hamburger menus, tiny targets, excess motion, animation without user interaction.

## 2. Liquid Glass (Apple, WWDC25 "Meet Liquid Glass", tvOS 26)
- Real properties: lensing (bends and concentrates light, not just blur), specular highlights that follow geometry/motion,
  adaptive shadows and tint, content-aware light/dark, thicker-looking material when larger.
- Rules: glass belongs to the navigation/control layer floating above content; never glass on glass; never glass in the content
  layer; tint sparingly; regular variant for legibility; honour Reduce Transparency/Motion.
- tvOS 26: translucent dock/top row that slightly warps the video behind it, "glassy edge" icons, glass scrub bar.
  Reviewers found legibility/contrast weaker. Apple only ships it on the 2nd/3rd-gen Apple TV 4K (too heavy for older chips).
- In CSS only blur + saturate + rim-light shadows are reproducible; real refraction needs an SVG displacement filter and a duplicated
  background (fragile and expensive). The cheap, effective part is the specular edge (inset box-shadows).

## 3. TV browser performance (why the launcher lagged)
- Each compositing layer costs width x height x 4 bytes of GPU memory (a full-HD layer is about 8 MB); dozens exhaust a TV GPU.
- backdrop-filter / filter: blur() are among the most expensive operations (every pixel behind the element is sampled); a
  full-screen blur is the worst case. Each glass element is another blur pass. Guidance: 2-3 at most, never animate them.
- Use will-change only while animating, never as a default; animate transform and opacity only (compositor-only).
- Keep visible DOM small (~100 nodes); window long lists; decode images at display size; use decoding="async".
- Do not re-render large trees on a timer; update small nodes directly.
- LG's own table: webOS 22 = Chromium 87. 2022 entry-level TV chips have a weak GPU (Mali-T820-class on the O22 platform).

## 4. Decisions made
1. The glass is a sheen over a tint with static rim-light shadows, and a light live blur only on the focused control (`docs/research/frame-rate.md`
   has the costs). The full-screen backdrops behind the lower rows, Settings and the Guide are a small, strongly
   blurred copy of the wallpaper, drawn once into a canvas: a full-screen live blur lagged.
2. Only `transform` animates on focus; the shadow/shine appear instantly; `will-change` only on the focused tile.
3. The clock is its own component (the home screen no longer re-renders every second).
4. The wallpaper picker draws only the rows near the one in view, not all 233 thumbnails.
5. Default 5 wallpapers bundled (each with a 480x270 blurred copy for the backdrops); the 233-image library loads from the cloud on demand.

## 5. Liquid Glass refraction on the webOS 25 TV (tested 2026-10-18)
- Chromium 120 accepts `backdrop-filter: blur(10px) saturate(1.8) brightness(1.06) url(#lg)`, where `#lg` is an SVG `feImage` + `feDisplacementMap` with a convex-bezel map made once on a canvas (the kube.io method).
- On the dock glass bar (1750 x 224) the frame rate stayed at 56-58 fps against 51 fps without it. The cost is small.
- The video wallpaper is on a hardware video layer under the page. The page reads every video pixel as 0 while the video plays. Thus no page effect (CSS blur, SVG refraction or WebGL) can blur or bend the video. Over the video, the glass shows only its tint and rims.
- Refraction can work over still pictures, Immich photos and plain backgrounds, because those are page content.
- The colour fringe (one displacement pass per colour, nikdelvin/liquid-glass) cost about 8 fps standing still, with 250 ms hitches. One pass costs 1-2 fps standing still and about half the frame rate while focus moves along the dock.
- Refraction on the panels (Control Center, Inputs) dropped to 7 fps while moving in them. Only the dock can take it. It is a setting (Liquid glass dock), off by default.
- The rest of the glass keeps a plain live blur, now 2px with a 30% black tint and the sheen and rims at about half strength. Sam preferred this look.

- Second round (2026-10-21), tried live over DevTools. Effects drawn on the glass itself cost nothing per frame and show over video too, as they read nothing behind the glass.
  - Kept: a colour fringe faked with coloured inset shadows (blue top left, pink bottom right at about 0.28). Sam found the pink works on the bar because some app icons are pink, so it reads as their reflection.
  - Kept: a highlight on the bar's top edge that slides to the focused app (transform only).
  - Rejected: the same fringe on the Control Center blocks. A fixed pink there reads as a coloured edge, not a reflection, and a strong bottom-right shadow makes a pill look inset, not rounded.
  - Rejected: a fringe coloured from the blurred wallpaper under each corner (sampled once on a canvas). It did not read as a reflection, and it cannot work over video.
  - Not tried: thickness shading and corner highlights (the same inset risk), and grain.
- Third round (2026-10-22), with `tools/tv/glass-lab.mjs` (sliders on the Mac that restyle the glass bar live, with an fps check), still wallpaper, focus moving along the dock:
  - Standing still, every setting holds about 60 fps. The cost only shows while something moves over the glass.
  - Any backdrop filter halves the frame rate while moving: the 3px live blur 16-24 fps, refraction with no blur 14-27 fps. Frost only (a tint, no backdrop filter) 40-47 fps.
  - Refraction strength (60 or 140) and bezel depth (14 or 40) make no clear difference. The cost is reading the backdrop at all, not the lens settings.
  - The colour fringe (3 passes) drops to 4-7 fps while moving.
  - Each layer alone, moving (3 runs each): tint, sheen and rims only 41 fps (45-47 without the focus highlight); colour boost (saturate and brightness) 33; 3px blur 25; refraction 18; refraction with the boost 19; refraction with blur and boost 15; refraction with the fringe 7. The first layer that reads the backdrop pays most of the cost.
- Native (Lumen Native, 2026-11-24, Glass lab with the focus moving along the bar): the colour fringe is one extra picture read per colour in the glass's rim only, and costs nothing measurable. Fringe 0, 0.12, 0.5 and 1 all held 50 frames a second at 14.7-14.9 ms a frame (the screen's rate), worst frame about 23 ms. The web fringe was slow because the browser ran three full SVG filter passes over the backdrop each frame, not because of the colour split itself.
- Online, only SVG displacement or WebGL does real refraction. The static rim and highlight tricks are from https://dev.to/kevinbism/recreating-apples-liquid-glass-effect-with-pure-css-3gpl and https://dev.to/afif/border-with-gradient-and-radius-387f.

## Sources
- https://webostv.developer.lge.com/develop/specifications/web-api-and-web-engine (Chromium per webOS version)
- https://developer.samsung.com/smarttv/develop/guides/web-app-memory-optimization-guide.html (TV memory/GPU rules)
- https://www.tothenew.com/blog/optimizing-performance-in-smarttv-html-tv-apps (TV CSS/DOM tips)
- https://developer.apple.com/videos/play/wwdc2025/219/ (Meet Liquid Glass)
- https://theplusaddons.com/blog/liquid-glass-ui/ (what CSS can and cannot reproduce)
- https://appleinsider.com/articles/25/06/18/tvos-26-hands-on-sleek-liquid-glass-redesign-new-control-center-and-more (tvOS 26)
- https://github.com/BasThomas/tvOS-guidelines, https://github.com/ehmo/platform-design-skills (tvOS rules summarised)
