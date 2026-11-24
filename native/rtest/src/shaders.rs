pub const VERTEX: &str = r#"attribute vec2 a_pos;
uniform vec4 u_quad;
uniform float u_flip;
varying vec2 v_px;
varying vec2 v_uv;
void main() {
  v_px = u_quad.xy + a_pos * u_quad.zw;
  v_uv = v_px / vec2(1920.0, 1080.0);
  vec2 c = v_uv * 2.0 - 1.0;
  gl_Position = vec4(c.x, c.y * u_flip, 0.0, 1.0);
}
"#;

pub const FRAGMENT_HEAD: &str = r#"#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif
varying vec2 v_px;
varying vec2 v_uv;
float rr(vec2 p, vec2 half_size, float r) {
  vec2 q = abs(p) - half_size + r;
  return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;
}
vec2 rr_normal(vec2 p, vec2 half_size, float r) {
  vec2 q = abs(p) - half_size + r;
  vec2 n = (q.x > 0.0 && q.y > 0.0) ? normalize(q) : (q.x > q.y ? vec2(1.0, 0.0) : vec2(0.0, 1.0));
  return n * vec2(p.x < 0.0 ? -1.0 : 1.0, p.y < 0.0 ? -1.0 : 1.0);
}
"#;

pub const COPY_FRAGMENT: &str = r#"uniform sampler2D u_tex;
uniform vec4 u_uvx;
void main() { gl_FragColor = vec4(texture2D(u_tex, v_uv * u_uvx.xy + u_uvx.zw).rgb, 1.0); }
"#;

pub const BLUR_FRAGMENT: &str = r#"uniform sampler2D u_tex;
uniform vec2 u_step;
uniform float u_mul;
void main() {
  vec3 c = texture2D(u_tex, v_uv).rgb * 0.2042;
  c += (texture2D(u_tex, v_uv + u_step * 1.4073).rgb + texture2D(u_tex, v_uv - u_step * 1.4073).rgb) * 0.3040;
  c += (texture2D(u_tex, v_uv + u_step * 3.2940).rgb + texture2D(u_tex, v_uv - u_step * 3.2940).rgb) * 0.0939;
  gl_FragColor = vec4(c * u_mul, 1.0);
}
"#;

pub const BACKDROP_FRAGMENT: &str = r#"uniform sampler2D u_wall;
uniform sampler2D u_veil;
uniform float u_t;
void main() { gl_FragColor = vec4(mix(texture2D(u_wall, v_uv).rgb, texture2D(u_veil, v_uv).rgb, u_t), 1.0); }
"#;

pub const LENS_FRAGMENT: &str = r#"uniform vec4 u_bar;
uniform float u_radius;
uniform float u_bezel;
void main() {
  vec2 hs = u_bar.zw * 0.5;
  vec2 p = v_px - (u_bar.xy + hs);
  float d = rr(p, hs, u_radius);
  float shadow = 0.28 * (1.0 - smoothstep(-26.0, 44.0, rr(p - vec2(0.0, 20.0), hs, u_radius)));
  float e = clamp(1.0 + d / u_bezel, 0.0, 1.0);
  vec2 off = -rr_normal(p, hs, u_radius) * (1.0 - sqrt(1.0 - e * e));
  gl_FragColor = vec4((off * 127.0 + 128.0) / 255.0, clamp(0.5 - d, 0.0, 1.0), shadow);
}
"#;

pub const GLOSS_FRAGMENT: &str = r#"uniform vec4 u_bar;
uniform float u_radius;
vec4 over(vec4 under, vec3 c, float a) { return vec4(under.rgb * (1.0 - a) + c * a, under.a * (1.0 - a) + a); }
void main() {
  vec2 hs = u_bar.zw * 0.5;
  vec2 p = v_px - (u_bar.xy + hs);
  float d = rr(p, hs, u_radius);
  vec2 n = rr_normal(p, hs, u_radius);
  float s = dot(p + hs, vec2(0.7071));
  float span = (u_bar.z + u_bar.w) * 0.7071;
  float sheen = mix(0.13, 0.03, clamp(s / 200.0, 0.0, 1.0)) + 0.05 * clamp((s - span + 200.0) / 200.0, 0.0, 1.0);
  float tl = smoothstep(0.0, 0.5, dot(n, vec2(-0.7071)));
  float br = smoothstep(0.0, 0.5, dot(n, vec2(0.7071)));
  float line = smoothstep(-2.5, -0.5, d);
  float band = smoothstep(-5.0, -2.0, d) * (1.0 - line);
  vec4 g = vec4(1.0, 1.0, 1.0, 1.0) * sheen;
  g = over(g, vec3(0.43, 0.78, 1.0), band * 0.5 * tl);
  g = over(g, vec3(1.0, 0.47, 0.75), band * 0.28 * br);
  g = over(g, vec3(1.0), line * (0.55 * tl + 0.2 * br));
  g = over(g, vec3(1.0), 0.05 * smoothstep(-14.0, 0.0, d));
  gl_FragColor = g;
}
"#;

pub const BAR_MIDDLE_FRAGMENT: &str = r#"uniform sampler2D u_glass;
uniform sampler2D u_veil;
uniform sampler2D u_gloss;
uniform float u_t;
uniform vec4 u_plate;
uniform vec3 u_tone;
vec3 behind(vec2 uv) {
  if (u_t <= 0.0) return texture2D(u_glass, uv).rgb;
  if (u_t >= 1.0) return texture2D(u_veil, uv).rgb;
  return mix(texture2D(u_glass, uv).rgb, texture2D(u_veil, uv).rgb, u_t);
}
vec3 glazed(vec3 col, vec2 q) {
  col = clamp(mix(vec3(dot(col, vec3(0.2126, 0.7152, 0.0722))), col, u_tone.x) * u_tone.y, 0.0, 1.0) * (1.0 - u_tone.z);
  vec4 g = texture2D(u_gloss, q);
  return col * (1.0 - g.a) + g.rgb;
}
void main() { gl_FragColor = vec4(glazed(behind(v_uv), (v_px - u_plate.xy) / u_plate.zw), 1.0); }
"#;

pub const BAR_RIM_FRAGMENT: &str = r#"uniform sampler2D u_glass;
uniform sampler2D u_veil;
uniform sampler2D u_gloss;
uniform float u_t;
uniform vec4 u_plate;
uniform vec3 u_tone;
vec3 behind(vec2 uv) {
  if (u_t <= 0.0) return texture2D(u_glass, uv).rgb;
  if (u_t >= 1.0) return texture2D(u_veil, uv).rgb;
  return mix(texture2D(u_glass, uv).rgb, texture2D(u_veil, uv).rgb, u_t);
}
vec3 glazed(vec3 col, vec2 q) {
  col = clamp(mix(vec3(dot(col, vec3(0.2126, 0.7152, 0.0722))), col, u_tone.x) * u_tone.y, 0.0, 1.0) * (1.0 - u_tone.z);
  vec4 g = texture2D(u_gloss, q);
  return col * (1.0 - g.a) + g.rgb;
}
uniform sampler2D u_lens;
uniform vec2 u_bend;
void main() {
  vec2 q = (v_px - u_plate.xy) / u_plate.zw;
  vec4 lens = texture2D(u_lens, q);
  if (lens.b <= 0.0) { gl_FragColor = vec4(0.0, 0.0, 0.0, lens.a); return; }
  vec2 off = (lens.rg * 255.0 - 128.0) / 127.0 * u_bend.x / vec2(1920.0, 1080.0);
  vec3 col;
  if (u_bend.x == 0.0) col = behind(v_uv);
  else if (u_bend.y > 0.0) col = vec3(behind(v_uv + off * (1.0 + u_bend.y)).r, behind(v_uv + off).g, behind(v_uv + off * (1.0 - u_bend.y)).b);
  else col = behind(v_uv + off);
  gl_FragColor = vec4(glazed(col, q) * lens.b, lens.b + (1.0 - lens.b) * lens.a);
}
"#;

pub const SHINE_FRAGMENT: &str = r#"uniform vec4 u_box;
uniform float u_alpha;
void main() {
  vec2 r = v_px - u_box.xy;
  float fall = 1.0 - abs(r.x / u_box.z * 2.0 - 1.0);
  gl_FragColor = vec4(u_alpha * fall * (0.9 * (1.0 - smoothstep(0.5, 2.0, r.y)) + 0.2 * fall * max(0.0, 1.0 - r.y / u_box.w)));
}
"#;

pub const TILE_FRAGMENT: &str = r#"uniform sampler2D u_icon;
uniform vec4 u_box;
uniform float u_radius;
uniform vec3 u_bg;
uniform vec3 u_shadow;
uniform float u_focus;
void main() {
  vec2 hs = u_box.zw * 0.5;
  vec2 p = v_px - (u_box.xy + hs);
  float d = rr(p, hs, u_radius);
  float a = clamp(0.5 - d, 0.0, 1.0);
  float shadow = u_shadow.z * (1.0 - smoothstep(-0.6 * u_shadow.y, u_shadow.y, rr(p - vec2(0.0, u_shadow.x), hs, u_radius)));
  if (a <= 0.0) { gl_FragColor = vec4(0.0, 0.0, 0.0, shadow); return; }
  vec3 col = u_bg;
  if (abs(p.x) < hs.y) {
    vec4 icon = texture2D(u_icon, p / u_box.w + 0.5);
    col = mix(col, icon.rgb, icon.a);
  }
  float g = dot(p, vec2(0.342, 0.940)) / (u_box.z * 0.342 + u_box.w * 0.940) + 0.5;
  float light = g < 0.3 ? mix(0.20, 0.06, g / 0.3) : mix(0.06, 0.0, clamp((g - 0.3) / 0.25, 0.0, 1.0));
  col = mix(col, vec3(1.0), light);
  col *= 1.0 - 0.08 * clamp((g - 0.55) / 0.45, 0.0, 1.0);
  float f = dot(p, vec2(0.906, 0.423)) / (u_box.z * 0.906 + u_box.w * 0.423) + 0.5;
  float lift = f < 0.28 ? mix(0.36, 0.10, f / 0.28) : mix(0.10, 0.0, clamp((f - 0.28) / 0.18, 0.0, 1.0));
  col = mix(col, vec3(1.0), lift * u_focus);
  vec2 n = rr_normal(p, hs, u_radius);
  float edge = 1.0 - smoothstep(0.5, 2.0, -d);
  col = mix(col, vec3(1.0), edge * (0.55 * max(-n.y, 0.0) + 0.28 * max(-n.x, 0.0)));
  col *= 1.0 - 0.18 * edge * max(n.y, 0.0);
  gl_FragColor = vec4(col * a, a + (1.0 - a) * shadow);
}
"#;

pub const FILL_FRAGMENT: &str = r#"uniform vec4 u_box;
uniform float u_radius;
uniform vec4 u_color;
void main() {
  vec2 hs = u_box.zw * 0.5;
  float a = clamp(0.5 - rr(v_px - (u_box.xy + hs), hs, u_radius), 0.0, 1.0) * u_color.a;
  gl_FragColor = vec4(u_color.rgb * a, a);
}
"#;

pub const TEXT_FRAGMENT: &str = r#"uniform sampler2D u_tex;
uniform vec4 u_box;
void main() { gl_FragColor = texture2D(u_tex, (v_px - u_box.xy) / u_box.zw); }
"#;
