#pragma once

static const char *const COPY_FRAGMENT =
    "uniform sampler2D u_tex;\n"
    "uniform vec4 u_uvx;\n"
    "void main() { gl_FragColor = vec4(texture2D(u_tex, v_uv * u_uvx.xy + u_uvx.zw).rgb, 1.0); }\n";

static const char *const BLUR_FRAGMENT =
    "uniform sampler2D u_tex;\n"
    "uniform vec2 u_step;\n"
    "uniform float u_mul;\n"
    "void main() {\n"
    "  vec3 c = texture2D(u_tex, v_uv).rgb * 0.2042;\n"
    "  c += (texture2D(u_tex, v_uv + u_step * 1.4073).rgb + texture2D(u_tex, v_uv - u_step * 1.4073).rgb) * 0.3040;\n"
    "  c += (texture2D(u_tex, v_uv + u_step * 3.2940).rgb + texture2D(u_tex, v_uv - u_step * 3.2940).rgb) * 0.0939;\n"
    "  gl_FragColor = vec4(c * u_mul, 1.0);\n"
    "}\n";

static const char *const BACKDROP_FRAGMENT =
    "uniform sampler2D u_wall;\n"
    "uniform sampler2D u_veil;\n"
    "uniform float u_t;\n"
    "void main() { gl_FragColor = vec4(mix(texture2D(u_wall, v_uv).rgb, texture2D(u_veil, v_uv).rgb, u_t), 1.0); }\n";

static const char *const LENS_FRAGMENT =
    "uniform vec4 u_bar;\n"
    "uniform float u_radius;\n"
    "uniform float u_bezel;\n"
    "void main() {\n"
    "  vec2 hs = u_bar.zw * 0.5;\n"
    "  vec2 p = v_px - (u_bar.xy + hs);\n"
    "  float d = rr(p, hs, u_radius);\n"
    "  float shadow = 0.28 * (1.0 - smoothstep(-26.0, 44.0, rr(p - vec2(0.0, 20.0), hs, u_radius)));\n"
    "  float e = clamp(1.0 + d / u_bezel, 0.0, 1.0);\n"
    "  vec2 off = -rr_normal(p, hs, u_radius) * (1.0 - sqrt(1.0 - e * e));\n"
    "  gl_FragColor = vec4((off * 127.0 + 128.0) / 255.0, clamp(0.5 - d, 0.0, 1.0), shadow);\n"
    "}\n";

static const char *const GLOSS_FRAGMENT =
    "uniform vec4 u_bar;\n"
    "uniform float u_radius;\n"
    "vec4 over(vec4 under, vec3 c, float a) { return vec4(under.rgb * (1.0 - a) + c * a, under.a * (1.0 - a) + a); }\n"
    "void main() {\n"
    "  vec2 hs = u_bar.zw * 0.5;\n"
    "  vec2 p = v_px - (u_bar.xy + hs);\n"
    "  float d = rr(p, hs, u_radius);\n"
    "  vec2 n = rr_normal(p, hs, u_radius);\n"
    "  float s = dot(p + hs, vec2(0.7071));\n"
    "  float span = (u_bar.z + u_bar.w) * 0.7071;\n"
    "  float sheen = mix(0.13, 0.03, clamp(s / 200.0, 0.0, 1.0)) + 0.05 * clamp((s - span + 200.0) / 200.0, 0.0, 1.0);\n"
    "  float tl = smoothstep(0.0, 0.5, dot(n, vec2(-0.7071)));\n"
    "  float br = smoothstep(0.0, 0.5, dot(n, vec2(0.7071)));\n"
    "  float line = smoothstep(-2.5, -0.5, d);\n"
    "  float band = smoothstep(-5.0, -2.0, d) * (1.0 - line);\n"
    "  vec4 g = vec4(1.0, 1.0, 1.0, 1.0) * sheen;\n"
    "  g = over(g, vec3(0.43, 0.78, 1.0), band * 0.5 * tl);\n"
    "  g = over(g, vec3(1.0, 0.47, 0.75), band * 0.28 * br);\n"
    "  g = over(g, vec3(1.0), line * (0.55 * tl + 0.2 * br));\n"
    "  g = over(g, vec3(1.0), 0.05 * smoothstep(-14.0, 0.0, d));\n"
    "  gl_FragColor = g;\n"
    "}\n";

#define BEHIND_AND_TONE \
    "uniform sampler2D u_glass;\n" \
    "uniform sampler2D u_veil;\n" \
    "uniform sampler2D u_gloss;\n" \
    "uniform float u_t;\n" \
    "uniform vec4 u_plate;\n" \
    "uniform vec3 u_tone;\n" \
    "vec3 behind(vec2 uv) {\n" \
    "  if (u_t <= 0.0) return texture2D(u_glass, uv).rgb;\n" \
    "  if (u_t >= 1.0) return texture2D(u_veil, uv).rgb;\n" \
    "  return mix(texture2D(u_glass, uv).rgb, texture2D(u_veil, uv).rgb, u_t);\n" \
    "}\n" \
    "vec3 glazed(vec3 col, vec2 q) {\n" \
    "  col = clamp(mix(vec3(dot(col, vec3(0.2126, 0.7152, 0.0722))), col, u_tone.x) * u_tone.y, 0.0, 1.0) * (1.0 - u_tone.z);\n" \
    "  vec4 g = texture2D(u_gloss, q);\n" \
    "  return col * (1.0 - g.a) + g.rgb;\n" \
    "}\n"

static const char *const BAR_MIDDLE_FRAGMENT =
    BEHIND_AND_TONE
    "void main() { gl_FragColor = vec4(glazed(behind(v_uv), (v_px - u_plate.xy) / u_plate.zw), 1.0); }\n";

static const char *const BAR_RIM_FRAGMENT =
    BEHIND_AND_TONE
    "uniform sampler2D u_lens;\n"
    "uniform vec2 u_bend;\n"
    "void main() {\n"
    "  vec2 q = (v_px - u_plate.xy) / u_plate.zw;\n"
    "  vec4 lens = texture2D(u_lens, q);\n"
    "  if (lens.b <= 0.0) { gl_FragColor = vec4(0.0, 0.0, 0.0, lens.a); return; }\n"
    "  vec2 off = (lens.rg * 255.0 - 128.0) / 127.0 * u_bend.x / vec2(1920.0, 1080.0);\n"
    "  vec3 col;\n"
    "  if (u_bend.x == 0.0) col = behind(v_uv);\n"
    "  else if (u_bend.y > 0.0) col = vec3(behind(v_uv + off * (1.0 + u_bend.y)).r, behind(v_uv + off).g, behind(v_uv + off * (1.0 - u_bend.y)).b);\n"
    "  else col = behind(v_uv + off);\n"
    "  gl_FragColor = vec4(glazed(col, q) * lens.b, lens.b + (1.0 - lens.b) * lens.a);\n"
    "}\n";

static const char *const SHINE_FRAGMENT =
    "uniform vec4 u_box;\n"
    "uniform float u_alpha;\n"
    "void main() {\n"
    "  vec2 r = v_px - u_box.xy;\n"
    "  float fall = 1.0 - abs(r.x / u_box.z * 2.0 - 1.0);\n"
    "  gl_FragColor = vec4(u_alpha * fall * (0.9 * (1.0 - smoothstep(0.5, 2.0, r.y)) + 0.2 * fall * max(0.0, 1.0 - r.y / u_box.w)));\n"
    "}\n";

static const char *const TILE_FRAGMENT =
    "uniform sampler2D u_icon;\n"
    "uniform vec4 u_box;\n"
    "uniform float u_radius;\n"
    "uniform vec3 u_bg;\n"
    "uniform vec3 u_shadow;\n"
    "uniform float u_focus;\n"
    "void main() {\n"
    "  vec2 hs = u_box.zw * 0.5;\n"
    "  vec2 p = v_px - (u_box.xy + hs);\n"
    "  float d = rr(p, hs, u_radius);\n"
    "  float a = clamp(0.5 - d, 0.0, 1.0);\n"
    "  float shadow = u_shadow.z * (1.0 - smoothstep(-0.6 * u_shadow.y, u_shadow.y, rr(p - vec2(0.0, u_shadow.x), hs, u_radius)));\n"
    "  if (a <= 0.0) { gl_FragColor = vec4(0.0, 0.0, 0.0, shadow); return; }\n"
    "  vec3 col = u_bg;\n"
    "  if (abs(p.x) < hs.y) {\n"
    "    vec4 icon = texture2D(u_icon, p / u_box.w + 0.5);\n"
    "    col = mix(col, icon.rgb, icon.a);\n"
    "  }\n"
    "  float g = dot(p, vec2(0.342, 0.940)) / (u_box.z * 0.342 + u_box.w * 0.940) + 0.5;\n"
    "  float light = g < 0.3 ? mix(0.20, 0.06, g / 0.3) : mix(0.06, 0.0, clamp((g - 0.3) / 0.25, 0.0, 1.0));\n"
    "  col = mix(col, vec3(1.0), light);\n"
    "  col *= 1.0 - 0.08 * clamp((g - 0.55) / 0.45, 0.0, 1.0);\n"
    "  float f = dot(p, vec2(0.906, 0.423)) / (u_box.z * 0.906 + u_box.w * 0.423) + 0.5;\n"
    "  float lift = f < 0.28 ? mix(0.36, 0.10, f / 0.28) : mix(0.10, 0.0, clamp((f - 0.28) / 0.18, 0.0, 1.0));\n"
    "  col = mix(col, vec3(1.0), lift * u_focus);\n"
    "  vec2 n = rr_normal(p, hs, u_radius);\n"
    "  float edge = 1.0 - smoothstep(0.5, 2.0, -d);\n"
    "  col = mix(col, vec3(1.0), edge * (0.55 * max(-n.y, 0.0) + 0.28 * max(-n.x, 0.0)));\n"
    "  col *= 1.0 - 0.18 * edge * max(n.y, 0.0);\n"
    "  gl_FragColor = vec4(col * a, a + (1.0 - a) * shadow);\n"
    "}\n";

static const char *const FILL_FRAGMENT =
    "uniform vec4 u_box;\n"
    "uniform float u_radius;\n"
    "uniform vec4 u_color;\n"
    "void main() {\n"
    "  vec2 hs = u_box.zw * 0.5;\n"
    "  float a = clamp(0.5 - rr(v_px - (u_box.xy + hs), hs, u_radius), 0.0, 1.0) * u_color.a;\n"
    "  gl_FragColor = vec4(u_color.rgb * a, a);\n"
    "}\n";

static const char *const TEXT_FRAGMENT =
    "uniform sampler2D u_tex;\n"
    "uniform vec4 u_box;\n"
    "void main() { gl_FragColor = texture2D(u_tex, (v_px - u_box.xy) / u_box.zw); }\n";
