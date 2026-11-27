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
uniform float u_mul;
uniform float u_print;
void main() {
  vec3 c = texture2D(u_tex, v_uv * u_uvx.xy + u_uvx.zw).rgb;
  if (u_print > 0.5) {
    float t = length((v_px - vec2(460.8, 64.8)) / vec2(1632.0, 1026.0));
    vec4 light = t < 0.38 ? mix(vec4(vec3(1.0, 0.957, 0.871) * 0.07, 0.07), vec4(0.0), t / 0.38) : vec4(0.0, 0.0, 0.0, 0.36 * min((t - 0.38) / 0.62, 1.0));
    c = light.rgb + c * (1.0 - light.a);
    c = mix(c, vec3(0.925, 0.894, 0.831), 0.09);
    c *= 0.86;
  }
  gl_FragColor = vec4(c * u_mul, 1.0);
}
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
uniform float u_edge_shape;
uniform vec2 u_side;
void main() {
  vec2 hs = u_bar.zw * 0.5;
  vec2 p = v_px - (u_bar.xy + hs);
  float d = rr(p, hs, u_radius);
  float shadow = 0.28 * (1.0 - smoothstep(-26.0, 44.0, rr(p - vec2(0.0, 20.0), hs, u_radius)));
  vec2 n = rr_normal(p, hs, u_radius);
  float side = abs(n.x);
  float e = clamp(1.0 + d / (u_bezel * mix(1.0, u_side.x, side)), 0.0, 1.0);
  vec2 off = -n * mix(1.0, u_side.y, side) * (1.0 - pow(max(1.0 - pow(e, u_edge_shape), 0.0), 1.0 / u_edge_shape));
  gl_FragColor = vec4((off * 127.0 + 128.0) / 255.0, clamp(0.5 - d, 0.0, 1.0), shadow);
}
"#;

pub const GLOSS_FRAGMENT: &str = r#"uniform vec4 u_bar;
uniform float u_radius;
uniform vec4 u_light;
uniform vec3 u_finish;
uniform float u_glow_reach;
vec4 over(vec4 under, vec3 c, float a) { return vec4(under.rgb * (1.0 - a) + c * a, under.a * (1.0 - a) + a); }
void main() {
  vec2 hs = u_bar.zw * 0.5;
  vec2 p = v_px - (u_bar.xy + hs);
  float d = rr(p, hs, u_radius);
  vec2 n = rr_normal(p, hs, u_radius);
  float s = dot(p + hs, vec2(0.7071));
  float span = (u_bar.z + u_bar.w) * 0.7071;
  float sheen = u_finish.y * (mix(0.13, 0.03, clamp(s / 200.0, 0.0, 1.0)) + 0.05 * clamp((s - span + 200.0) / 200.0, 0.0, 1.0));
  float facing = dot(n, u_light.xy);
  float tl = smoothstep(0.0, 0.5, facing);
  float br = smoothstep(0.0, 0.5, -facing);
  float line = smoothstep(-2.5, -0.5, d);
  float band = smoothstep(-5.0, -2.0, d) * (1.0 - line);
  vec4 g = vec4(1.0, 1.0, 1.0, 1.0) * clamp(sheen, 0.0, 1.0);
  g = over(g, vec3(0.43, 0.78, 1.0), clamp(band * 0.5 * tl * u_light.w, 0.0, 1.0));
  g = over(g, vec3(1.0, 0.47, 0.75), clamp(band * 0.28 * br * u_light.w, 0.0, 1.0));
  g = over(g, vec3(1.0), clamp(line * (0.55 * tl + 0.2 * br) * u_finish.x, 0.0, 1.0));
  g = over(g, vec3(1.0), clamp(0.05 * u_finish.z * smoothstep(-u_glow_reach, 0.0, d), 0.0, 1.0));
  float spark = pow(max(facing, 0.0), 6.0) * (smoothstep(-4.0, -0.5, d) + 0.35 * smoothstep(-16.0, 0.0, d)) + 0.5 * pow(max(-facing, 0.0), 6.0) * smoothstep(-4.0, -0.5, d);
  g = over(g, vec3(1.0), clamp(u_light.z * spark, 0.0, 1.0));
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
uniform float u_frost;
uniform vec3 u_mag;
uniform float u_clear;
vec4 glazed(vec3 col, vec2 q) {
  col = clamp(mix(vec3(dot(col, vec3(0.2126, 0.7152, 0.0722))), col, u_tone.x) * u_tone.y, 0.0, 1.0) * (1.0 - u_tone.z);
  float a = 1.0;
  if (u_clear > 0.0) { col = vec3(0.0); a = u_tone.z; }
  col = col * (1.0 - u_frost) + u_frost;
  a = a + (1.0 - a) * u_frost;
  vec4 g = texture2D(u_gloss, q);
  return vec4(col * (1.0 - g.a) + g.rgb, a * (1.0 - g.a) + g.a);
}
void main() { gl_FragColor = glazed(behind(v_uv + (u_mag.xy - v_px) * u_mag.z / vec2(1920.0, 1080.0)), (v_px - u_plate.xy) / u_plate.zw); }
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
uniform float u_frost;
uniform vec3 u_mag;
uniform float u_clear;
vec4 glazed(vec3 col, vec2 q) {
  col = clamp(mix(vec3(dot(col, vec3(0.2126, 0.7152, 0.0722))), col, u_tone.x) * u_tone.y, 0.0, 1.0) * (1.0 - u_tone.z);
  float a = 1.0;
  if (u_clear > 0.0) { col = vec3(0.0); a = u_tone.z; }
  col = col * (1.0 - u_frost) + u_frost;
  a = a + (1.0 - a) * u_frost;
  vec4 g = texture2D(u_gloss, q);
  return vec4(col * (1.0 - g.a) + g.rgb, a * (1.0 - g.a) + g.a);
}
uniform sampler2D u_lens;
uniform vec2 u_bend;
void main() {
  vec2 q = (v_px - u_plate.xy) / u_plate.zw;
  vec4 lens = texture2D(u_lens, q);
  if (lens.b <= 0.0) { gl_FragColor = vec4(0.0, 0.0, 0.0, lens.a); return; }
  vec2 off = (lens.rg * 255.0 - 128.0) / 127.0 * u_bend.x / vec2(1920.0, 1080.0);
  vec2 uv = v_uv + (u_mag.xy - v_px) * u_mag.z / vec2(1920.0, 1080.0);
  vec3 col;
  if (u_bend.x == 0.0) col = behind(uv);
  else if (u_bend.y > 0.0) col = vec3(behind(uv + off * (1.0 + u_bend.y)).r, behind(uv + off).g, behind(uv + off * (1.0 - u_bend.y)).b);
  else col = behind(uv + off);
  vec4 c = glazed(col, q);
  gl_FragColor = vec4(c.rgb * lens.b, c.a * lens.b + (1.0 - lens.b) * lens.a);
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
uniform float u_wide;
uniform float u_turn;
void main() {
  vec2 hs = u_box.zw * 0.5;
  vec2 p = v_px - (u_box.xy + hs);
  float cs = cos(u_turn), sn = sin(u_turn);
  p = vec2(cs * p.x + sn * p.y, -sn * p.x + cs * p.y);
  float d = rr(p, hs, u_radius);
  float a = clamp(0.5 - d, 0.0, 1.0);
  float shadow = u_shadow.z * (1.0 - smoothstep(-0.6 * u_shadow.y, u_shadow.y, rr(p - vec2(0.0, u_shadow.x), hs, u_radius)));
  if (a <= 0.0) { gl_FragColor = vec4(0.0, 0.0, 0.0, shadow); return; }
  vec3 col = u_bg;
  if (u_wide > 0.5) {
    vec4 icon = texture2D(u_icon, p / u_box.zw + 0.5);
    col = icon.rgb;
  } else if (abs(p.x) < hs.y) {
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

pub const PICTURE_FRAGMENT: &str = r#"uniform sampler2D u_tex;
uniform vec4 u_box;
uniform float u_alpha;
void main() { gl_FragColor = texture2D(u_tex, (v_px - u_box.xy) / u_box.zw) * u_alpha; }
"#;

pub const INK_FRAGMENT: &str = r#"uniform sampler2D u_tex;
uniform vec4 u_box;
uniform vec4 u_ink;
void main() { gl_FragColor = vec4(u_ink.rgb, 1.0) * texture2D(u_tex, (v_px - u_box.xy) / u_box.zw).a * u_ink.a; }
"#;

pub const GLASS_FRAGMENT: &str = r#"uniform sampler2D u_glass;
uniform float u_shadow_strength;
uniform vec4 u_box;
uniform vec4 u_look;
uniform vec3 u_tone;
uniform float u_alpha;
uniform vec3 u_bend;
uniform vec4 u_shape;
uniform vec4 u_light;
uniform vec2 u_side;
void main() {
  vec2 hs = u_box.zw * 0.5;
  vec2 p = v_px - (u_box.xy + hs);
  float r = min(u_look.x, min(hs.x, hs.y));
  float d = rr(p, hs, r);
  float a = clamp(0.5 - d, 0.0, 1.0);
  float shadow = u_shadow_strength * (1.0 - smoothstep(-10.0, 18.0, rr(p - vec2(0.0, 8.0), hs, r)));
  if (a <= 0.0) { gl_FragColor = vec4(0.0, 0.0, 0.0, shadow * u_alpha); return; }
  vec2 n = rr_normal(p, hs, r);
  vec3 seen;
  if (u_bend.x == 0.0 && u_shape.y == 0.0) seen = texture2D(u_glass, v_uv).rgb;
  else {
    float bezel = min(u_bend.z, 0.5 * min(hs.x, hs.y));
    float side = abs(n.x);
    float e = clamp(1.0 + d / (bezel * mix(1.0, u_side.x, side)), 0.0, 1.0);
    float lift = 1.0 - pow(max(1.0 - pow(e, u_shape.x), 0.0), 1.0 / u_shape.x);
    vec2 off = (-n * lift * mix(1.0, u_side.y, side) * u_bend.x * bezel / u_bend.z - p * u_shape.y) / vec2(1920.0, 1080.0);
    seen = vec3(texture2D(u_glass, v_uv + off * (1.0 + u_bend.y)).r, texture2D(u_glass, v_uv + off).g, texture2D(u_glass, v_uv + off * (1.0 - u_bend.y)).b);
  }
  float grey = dot(seen, vec3(0.299, 0.587, 0.114));
  seen = clamp(mix(vec3(grey), seen, u_tone.x) * u_tone.y, 0.0, 1.0);
  float blur = u_look.y;
  vec3 col = seen * (1.0 - u_tone.z) * blur;
  float alpha = mix(u_tone.z, 1.0, blur);
  float g = dot(p / max(hs, vec2(1.0)), vec2(0.574, 0.819)) * 0.5 + 0.5;
  float sheen = u_look.z * (g < 0.42 ? mix(0.13, 0.03, g / 0.42) : mix(0.03, 0.08, (g - 0.42) / 0.58));
  col = col * (1.0 - sheen) + sheen;
  alpha = alpha + (1.0 - alpha) * sheen;
  col = col * (1.0 - u_shape.z) + u_shape.z;
  alpha = alpha + (1.0 - alpha) * u_shape.z;
  float facing = dot(n, u_light.xy);
  float band = smoothstep(-5.0, -2.0, d) * (1.0 - smoothstep(-2.5, -0.5, d));
  float blue = clamp(band * 0.5 * smoothstep(0.0, 0.5, facing) * u_light.w, 0.0, 1.0);
  float pink = clamp(band * 0.28 * smoothstep(0.0, 0.5, -facing) * u_light.w, 0.0, 1.0);
  col = col * (1.0 - blue) + vec3(0.43, 0.78, 1.0) * blue;
  alpha = alpha + (1.0 - alpha) * blue;
  col = col * (1.0 - pink) + vec3(1.0, 0.47, 0.75) * pink;
  alpha = alpha + (1.0 - alpha) * pink;
  float edge = 1.0 - smoothstep(0.0, 2.0, -d);
  float glow = 1.0 - smoothstep(0.0, 14.0, -d);
  float rim = clamp(u_look.w * (edge * (0.55 * max(facing, 0.0) + 0.25 * max(-facing, 0.0)) + 0.05 * u_shape.w * glow), 0.0, 1.0);
  col = col * (1.0 - rim) + rim;
  alpha = alpha + (1.0 - alpha) * rim;
  float spark = clamp(u_light.z * (pow(max(facing, 0.0), 6.0) * (smoothstep(-4.0, -0.5, d) + 0.35 * glow) + 0.5 * pow(max(-facing, 0.0), 6.0) * smoothstep(-4.0, -0.5, d)), 0.0, 1.0);
  col = col * (1.0 - spark) + spark;
  alpha = alpha + (1.0 - alpha) * spark;
  gl_FragColor = vec4(col * a, alpha * a + (1.0 - alpha * a) * shadow) * u_alpha;
}
"#;

pub const DIM_FRAGMENT: &str = r#"uniform float u_alpha;
void main() { gl_FragColor = vec4(0.0, 0.0, 0.0, u_alpha); }
"#;

pub const FILL_FRAGMENT: &str = r#"uniform vec4 u_box;
uniform float u_radius;
uniform vec4 u_ink;
void main() {
  vec2 hs = u_box.zw * 0.5;
  float a = clamp(0.5 - rr(v_px - (u_box.xy + hs), hs, min(u_radius, min(hs.x, hs.y))), 0.0, 1.0);
  gl_FragColor = vec4(u_ink.rgb, 1.0) * u_ink.a * a;
}
"#;

pub const SPIN_FRAGMENT: &str = r#"uniform vec4 u_box;
uniform float u_width;
uniform float u_turn;
void main() {
  vec2 p = v_px - (u_box.xy + u_box.zw * 0.5);
  float mid = u_box.z * 0.5 - u_width * 0.5;
  float a = clamp(u_width * 0.5 + 0.5 - abs(length(p) - mid), 0.0, 1.0);
  float angle = atan(p.y, p.x) + 1.5708 - u_turn;
  angle = mod(angle + 3.14159, 6.28318) - 3.14159;
  float lit = mix(0.25, 1.0, clamp((0.7854 - abs(angle)) * mid + 0.5, 0.0, 1.0));
  gl_FragColor = vec4(a * lit);
}
"#;
