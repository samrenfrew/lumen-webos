#ifdef LUMEN_WEBOS
#define SDL_VIDEO_DRIVER_WAYLAND 1
#endif
#include <SDL.h>
#include <SDL_image.h>
#include <SDL_ttf.h>
#ifdef LUMEN_WEBOS
#include <SDL_syswm.h>
#include <wayland-client-core.h>
#include <wayland-client-protocol.h>
#include <wayland-webos-shell-client-protocol.h>
#endif
#include <glob.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include "gfx.h"
#include "shaders.h"
#include "util.h"

#define TUNING_PATH "/tmp/ntest-tune"

enum { ROWS = 3, COLS = 6, TILES = ROWS * COLS, IDLE_WAIT_MS = 250, PLATE_MARGIN = 70, TILE_MARGIN = 28 };
enum { LIQUID, BLUR_ONLY, NO_BLUR, GLASS_MODES };

static const float TILE_W = 240, TILE_H = 144, TILE_GAP = 46, TILE_RADIUS = 34, ROWS_EDGE = 125;
static const float ROW_Y[ROWS] = {846, 1060, 1317};
static const float BAR_X = 85, BAR_Y = 806, BAR_W = 1750, BAR_H = 224, BAR_RADIUS = 74;
static const float LOWER_SHIFT = 770, GRID_DROP = 40, LIFT_BAR = 1.12f, LIFT_GRID = 1.14f, LIFT_RISE = 7.5f;
static const float SHADOW_REST[3] = {8, 16, 0.30f}, SHADOW_BAR[3] = {12, 22, 0.38f}, SHADOW_GRID[3] = {26, 40, 0.46f};
static const float VEIL_TINT = 0.26f;
static const char *const MODE_NAMES[GLASS_MODES] = {"Liquid glass", "Blur only", "No blur"};

static const float EASE_UP[4] = {0.65f, 0, 0.25f, 1}, EASE_DOWN[4] = {0.25f, 0.1f, 0.25f, 1}, EASE_VEIL[4] = {0.45f, 0, 0.55f, 1};
static const float EASE_OUT[4] = {0, 0, 0.58f, 1}, EASE_SCROLL[4] = {0.2f, 0.7f, 0.2f, 1}, EASE[4] = {0.25f, 0.1f, 0.25f, 1};

static struct {
    float down_ms, up_ms, veil_down_ms, veil_up_ms, focus_ms;
    float glass_halvings, glass_passes, veil_halvings, veil_passes;
    float bezel, bend, fringe, saturate, brighten, tint;
    float mode, hud, finish;
    char wallpaper[256];
} tune = {675, 600, 600, 800, 160, 1, 2, 2, 3, 28, 44, 0.12f, 1.5f, 1.1f, 0.3f, LIQUID, 1, 1, ""};

static const struct { const char *name; float *value; float min, max; } TUNABLES[] = {
    {"down", &tune.down_ms, 1, 5000}, {"up", &tune.up_ms, 1, 5000},
    {"veil_down", &tune.veil_down_ms, 1, 5000}, {"veil_up", &tune.veil_up_ms, 1, 5000}, {"focus", &tune.focus_ms, 1, 2000},
    {"glass_halvings", &tune.glass_halvings, 0, 4}, {"glass_passes", &tune.glass_passes, 1, 8},
    {"veil_halvings", &tune.veil_halvings, 0, 4}, {"veil_passes", &tune.veil_passes, 1, 8},
    {"bezel", &tune.bezel, 1, 110}, {"bend", &tune.bend, -200, 200}, {"fringe", &tune.fringe, 0, 1},
    {"saturate", &tune.saturate, 0, 3}, {"brighten", &tune.brighten, 0, 3}, {"tint", &tune.tint, 0, 1},
    {"mode", &tune.mode, 0, GLASS_MODES - 1}, {"hud", &tune.hud, 0, 1}, {"finish", &tune.finish, 0, 1},
};

typedef struct { float from, to; Uint32 start, dur; const float *ease; } Anim;
typedef struct { Tex icon; float bg[3]; Anim focus; Target resting; } Tile;

static char app_dir[256] = ".";
static GLuint copy_program, blur_program, backdrop_program, lens_program, gloss_program, bar_middle_program, bar_rim_program, shine_program, tile_program, fill_program, text_program;
static Target sharp, glass, veil, lens_plate, gloss_plate;
static float baked_bezel, refresh_ms = 16.7f;
static Tile tiles[TILES];
static Tex hud;
static TTF_Font *font;
static Anim slide, veil_fade, drop, shine_x, shine_alpha;
static int focus_row, focus_col;
static char last_move[160] = "Down and Up slide, Left and Right move, OK changes the glass, Back closes";

enum { NO_BACKDROP = 1, NO_GRID = 2, NO_BAR = 4, NO_BAR_TILES = 8, NO_HUD = 16, NO_CLEAR = 32 };
static int leave_out;

static struct { int active, frames, late; Uint32 started, last, slowest; double draw_ms, draw_worst; const char *name; } move;

static float bezier(float a, float b, float t) { float u = 1 - t; return 3 * u * u * t * a + 3 * u * t * t * b + t * t * t; }

static float eased(const float *e, float x) {
    if (x <= 0) return 0;
    if (x >= 1) return 1;
    float lo = 0, hi = 1, t = x;
    for (int i = 0; i < 20; i++) {
        float at = bezier(e[0], e[2], t);
        if (at < x) lo = t; else hi = t;
        t = (lo + hi) / 2;
    }
    return bezier(e[1], e[3], t);
}

static float anim_at(const Anim *a, Uint32 now) {
    if (!a->dur || now - a->start >= a->dur) return a->to;
    return a->from + (a->to - a->from) * eased(a->ease, (float)(now - a->start) / a->dur);
}

static int anim_busy(const Anim *a, Uint32 now) { return a->dur && now - a->start < a->dur; }

static void anim_go(Anim *a, float to, float ms, const float *ease, Uint32 now) {
    if (a->to == to) return;
    a->from = anim_at(a, now);
    a->to = to;
    a->start = now;
    a->dur = (Uint32)ms;
    a->ease = ease;
}

static void anim_set(Anim *a, float value) { a->from = a->to = value; a->dur = 0; }

static int anything_moving(Uint32 now) {
    if (anim_busy(&slide, now) || anim_busy(&veil_fade, now) || anim_busy(&drop, now) || anim_busy(&shine_x, now) || anim_busy(&shine_alpha, now)) return 1;
    for (int i = 0; i < TILES; i++) if (anim_busy(&tiles[i].focus, now)) return 1;
    return 0;
}

static void read_tuning(void) {
    FILE *f = fopen(TUNING_PATH, "r");
    if (!f) return;
    char line[320];
    while (fgets(line, sizeof line, f)) {
        char *eq = strchr(line, '=');
        if (!eq) continue;
        *eq++ = 0;
        eq[strcspn(eq, "\r\n")] = 0;
        if (!strcmp(line, "wallpaper")) { snprintf(tune.wallpaper, sizeof tune.wallpaper, "%s", eq); continue; }
        for (size_t i = 0; i < sizeof TUNABLES / sizeof *TUNABLES; i++) {
            if (strcmp(line, TUNABLES[i].name)) continue;
            float v = (float)atof(eq);
            if (v >= TUNABLES[i].min && v <= TUNABLES[i].max) *TUNABLES[i].value = v;
        }
    }
    fclose(f);
}

static void copy(const Tex *from, const Target *into, float sx, float sy, float ox, float oy) {
    gfx_into(into);
    gfx_bind(copy_program, 0, "u_tex", from->tex);
    glUniform4f(gfx_loc(copy_program, "u_uvx"), sx, sy, ox, oy);
    gfx_quad(copy_program, 0, 0, DESIGN_W, DESIGN_H);
}

static Target blurred(const Target *from, int halvings, int passes, float mul) {
    Target cur = {0, {0, 0, 0}};
    const Tex *src = &from->t;
    for (int i = 0; i < halvings || !cur.fbo; i++) {
        int shrink = halvings > 0 ? 2 : 1;
        Target next = gfx_target(src->w / shrink, src->h / shrink);
        copy(src, &next, 1, 1, 0, 0);
        if (cur.fbo) gfx_target_free(&cur);
        cur = next;
        src = &cur.t;
    }
    Target tmp = gfx_target(cur.t.w, cur.t.h);
    for (int i = 0; i < passes; i++) {
        gfx_into(&tmp);
        gfx_bind(blur_program, 0, "u_tex", cur.t.tex);
        glUniform2f(gfx_loc(blur_program, "u_step"), 1.0f / cur.t.w, 0);
        glUniform1f(gfx_loc(blur_program, "u_mul"), 1);
        gfx_quad(blur_program, 0, 0, DESIGN_W, DESIGN_H);
        gfx_into(&cur);
        gfx_bind(blur_program, 0, "u_tex", tmp.t.tex);
        glUniform2f(gfx_loc(blur_program, "u_step"), 0, 1.0f / cur.t.h);
        glUniform1f(gfx_loc(blur_program, "u_mul"), i == passes - 1 ? mul : 1);
        gfx_quad(blur_program, 0, 0, DESIGN_W, DESIGN_H);
    }
    gfx_target_free(&tmp);
    return cur;
}

static int build_backdrops(int screen_w, int screen_h) {
    Uint32 t0 = SDL_GetTicks();
    char path[400];
    if (tune.wallpaper[0]) snprintf(path, sizeof path, "%s", tune.wallpaper);
    else snprintf(path, sizeof path, "%s/wallpaper.jpg", app_dir);
    SDL_Surface *picture = IMG_Load(path);
    if (!picture) { note("wallpaper %s: %s", path, IMG_GetError()); return 0; }
    Uint32 decoded = SDL_GetTicks();
    Tex raw = gfx_upload(picture, 0);
    SDL_FreeSurface(picture);
    glDisable(GL_BLEND);
    if (sharp.fbo) { gfx_target_free(&sharp); gfx_target_free(&glass); gfx_target_free(&veil); }
    sharp = gfx_target(screen_w, screen_h);
    float shape = ((float)raw.w / raw.h) / ((float)DESIGN_W / DESIGN_H);
    if (shape > 1) copy(&raw, &sharp, 1 / shape, 1, (1 - 1 / shape) / 2, 0);
    else copy(&raw, &sharp, 1, shape, 0, (1 - shape) / 2);
    gfx_free(&raw);
    Target half = gfx_target(screen_w / 2, screen_h / 2);
    copy(&sharp.t, &half, 1, 1, 0, 0);
    glass = blurred(&half, (int)tune.glass_halvings, (int)tune.glass_passes, 1);
    veil = blurred(&half, (int)tune.veil_halvings, (int)tune.veil_passes, 1 - VEIL_TINT);
    gfx_target_free(&half);
    glFinish();
    gfx_into(NULL);
    note("backdrops: decode %u ms, upload and blur %u ms, glass %dx%d, veil %dx%d", decoded - t0, SDL_GetTicks() - decoded, glass.t.w, glass.t.h, veil.t.w, veil.t.h);
    return 1;
}

static void load_tiles(void) {
    char pattern[300];
    snprintf(pattern, sizeof pattern, "%s/icons/*.png", app_dir);
    glob_t found;
    size_t count = glob(pattern, 0, NULL, &found) == 0 ? found.gl_pathc : 0;
    for (int i = 0; i < TILES; i++) {
        Tile *t = &tiles[i];
        t->bg[0] = 0.16f; t->bg[1] = 0.17f; t->bg[2] = 0.22f;
        if (!count) continue;
        SDL_Surface *loaded = IMG_Load(found.gl_pathv[(size_t)i % count]);
        SDL_Surface *icon = loaded ? SDL_ConvertSurfaceFormat(loaded, SDL_PIXELFORMAT_RGBA32, 0) : NULL;
        if (loaded) SDL_FreeSurface(loaded);
        if (!icon) continue;
        const Uint8 *edge = (const Uint8 *)icon->pixels + (size_t)(icon->h / 2) * icon->pitch + 3 * 4;
        if (edge[3] > 200) for (int c = 0; c < 3; c++) t->bg[c] = edge[c] / 255.0f;
        t->icon = gfx_upload(icon, 0);
        SDL_FreeSurface(icon);
    }
    if (count) globfree(&found);
    note("tiles: %d icons", (int)count);
}

static void open_font(void) {
    const char *const places[] = {"%s/fonts/sf-500.ttf", "/usr/share/fonts/LG_Smart_UI-Regular.ttf", "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"};
    for (size_t i = 0; i < sizeof places / sizeof *places && !font; i++) {
        char path[400];
        snprintf(path, sizeof path, places[i], app_dir);
        font = TTF_OpenFont(path, 24);
    }
    if (!font) note("hud: no font");
}

static void draw_hud_text(void) {
    gfx_free(&hud);
    hud.w = hud.h = 0;
    if (!font) return;
    char first[120];
    snprintf(first, sizeof first, "Glass: %s", MODE_NAMES[(int)tune.mode]);
    const SDL_Color white = {255, 255, 255, 255};
    SDL_Surface *lines[2] = {TTF_RenderUTF8_Blended(font, first, white), TTF_RenderUTF8_Blended(font, last_move, white)};
    if (lines[0] && lines[1]) {
        int w = lines[0]->w > lines[1]->w ? lines[0]->w : lines[1]->w;
        SDL_Surface *both = SDL_CreateRGBSurfaceWithFormat(0, w, lines[0]->h + lines[1]->h + 6, 32, SDL_PIXELFORMAT_RGBA32);
        if (both) {
            SDL_SetSurfaceBlendMode(lines[0], SDL_BLENDMODE_NONE);
            SDL_SetSurfaceBlendMode(lines[1], SDL_BLENDMODE_NONE);
            SDL_Rect second = {0, lines[0]->h + 6, 0, 0};
            SDL_BlitSurface(lines[0], NULL, both, NULL);
            SDL_BlitSurface(lines[1], NULL, both, &second);
            hud = gfx_upload(both, 1);
            SDL_FreeSurface(both);
        }
    }
    if (lines[0]) SDL_FreeSurface(lines[0]);
    if (lines[1]) SDL_FreeSurface(lines[1]);
}

static void tile_uniforms(const Tile *t, float x, float y, float scale, const float *shadow, float focus) {
    gfx_bind(tile_program, 0, "u_icon", t->icon.tex);
    glUniform4f(gfx_loc(tile_program, "u_box"), x, y, TILE_W * scale, TILE_H * scale);
    glUniform1f(gfx_loc(tile_program, "u_radius"), TILE_RADIUS * scale);
    glUniform3f(gfx_loc(tile_program, "u_bg"), t->bg[0], t->bg[1], t->bg[2]);
    glUniform3f(gfx_loc(tile_program, "u_shadow"), shadow[0], shadow[1], shadow[2]);
    glUniform1f(gfx_loc(tile_program, "u_focus"), focus);
}

static void bake_tiles(void) {
    glDisable(GL_BLEND);
    float w = TILE_W + 2 * TILE_MARGIN, h = TILE_H + 2 * TILE_MARGIN;
    for (int i = 0; i < TILES; i++) {
        tiles[i].resting = gfx_target((int)w, (int)h);
        gfx_into_plate(&tiles[i].resting);
        tile_uniforms(&tiles[i], TILE_MARGIN, TILE_MARGIN, 1, SHADOW_REST, 0);
        gfx_quad(tile_program, 0, 0, w, h);
    }
    gfx_into(NULL);
}

static void bake_bar(void) {
    float w = BAR_W + 2 * PLATE_MARGIN, h = BAR_H + 2 * PLATE_MARGIN;
    if (!lens_plate.fbo) { lens_plate = gfx_target((int)w, (int)h); gloss_plate = gfx_target((int)w, (int)h); }
    glDisable(GL_BLEND);
    gfx_into_plate(&lens_plate);
    glUseProgram(lens_program);
    glUniform4f(gfx_loc(lens_program, "u_bar"), PLATE_MARGIN, PLATE_MARGIN, BAR_W, BAR_H);
    glUniform1f(gfx_loc(lens_program, "u_radius"), BAR_RADIUS);
    glUniform1f(gfx_loc(lens_program, "u_bezel"), tune.bezel);
    gfx_quad(lens_program, 0, 0, w, h);
    gfx_into_plate(&gloss_plate);
    glUseProgram(gloss_program);
    glUniform4f(gfx_loc(gloss_program, "u_bar"), PLATE_MARGIN, PLATE_MARGIN, BAR_W, BAR_H);
    glUniform1f(gfx_loc(gloss_program, "u_radius"), BAR_RADIUS);
    gfx_quad(gloss_program, 0, 0, w, h);
    gfx_into(NULL);
    baked_bezel = tune.bezel;
}

static void draw_tile(int index, float shift, float grid_drop, Uint32 now) {
    Tile *t = &tiles[index];
    int row = index / COLS, col = index % COLS;
    float f = anim_at(&t->focus, now);
    float left = ROWS_EDGE + col * (TILE_W + TILE_GAP), top = ROW_Y[row] - shift + (row ? grid_drop : 0);
    if (top - 80 > DESIGN_H || top + TILE_H + 80 < 0) return;
    if (f <= 0 && t->resting.fbo) {
        float w = TILE_W + 2 * TILE_MARGIN, h = TILE_H + 2 * TILE_MARGIN;
        gfx_bind(text_program, 0, "u_tex", t->resting.t.tex);
        glUniform4f(gfx_loc(text_program, "u_box"), left - TILE_MARGIN, top - TILE_MARGIN, w, h);
        gfx_quad(text_program, left - TILE_MARGIN, top - TILE_MARGIN, w, h);
        return;
    }
    float scale = 1 + ((row ? LIFT_GRID : LIFT_BAR) - 1) * f;
    float w = TILE_W * scale, h = TILE_H * scale;
    float x = left + TILE_W / 2 - w / 2, y = top + TILE_H / 2 - LIFT_RISE * f * scale - h / 2;
    const float *lifted = row ? SHADOW_GRID : SHADOW_BAR;
    float shadow[3];
    for (int i = 0; i < 3; i++) shadow[i] = SHADOW_REST[i] + (lifted[i] - SHADOW_REST[i]) * f;
    float margin = shadow[0] + shadow[1] + 4;
    tile_uniforms(t, x, y, scale, shadow, f);
    gfx_quad(tile_program, x - margin, y - margin, w + 2 * margin, h + 2 * margin);
}

static void draw_row(int row, float shift, float grid_drop, Uint32 now) {
    for (int col = 0; col < COLS; col++) if (!(row == focus_row && col == focus_col)) draw_tile(row * COLS + col, shift, grid_drop, now);
    if (row == focus_row) draw_tile(row * COLS + focus_col, shift, grid_drop, now);
}

static void bar_uniforms(GLuint program, float plate_x, float plate_y, float veiled) {
    int mode = (int)tune.mode;
    gfx_bind(program, 0, "u_glass", mode == NO_BLUR ? sharp.t.tex : glass.t.tex);
    gfx_bind(program, 1, "u_veil", veil.t.tex);
    gfx_bind(program, 2, "u_gloss", gloss_plate.t.tex);
    glUniform1f(gfx_loc(program, "u_t"), veiled);
    glUniform4f(gfx_loc(program, "u_plate"), plate_x, plate_y, (float)lens_plate.t.w, (float)lens_plate.t.h);
    glUniform3f(gfx_loc(program, "u_tone"), mode == NO_BLUR ? 1 : tune.saturate, mode == NO_BLUR ? 1 : tune.brighten, tune.tint);
}

static void draw_bar(float bar_y, float veiled, Uint32 now) {
    float px = BAR_X - PLATE_MARGIN, py = bar_y - PLATE_MARGIN, pw = BAR_W + 2 * PLATE_MARGIN, ph = BAR_H + 2 * PLATE_MARGIN;
    float inset = (baked_bezel > 22 ? baked_bezel : 22) + 1;
    float ix = BAR_X + inset, iy = bar_y + inset, iw = BAR_W - 2 * inset, ih = BAR_H - 2 * inset;

    glDisable(GL_BLEND);
    bar_uniforms(bar_middle_program, px, py, veiled);
    gfx_quad(bar_middle_program, ix, iy, iw, ih);

    glEnable(GL_BLEND);
    bar_uniforms(bar_rim_program, px, py, veiled);
    gfx_bind(bar_rim_program, 3, "u_lens", lens_plate.t.tex);
    glUniform2f(gfx_loc(bar_rim_program, "u_bend"), (int)tune.mode == LIQUID ? tune.bend : 0, tune.fringe);
    gfx_quad(bar_rim_program, px, py, pw, iy - py);
    gfx_quad(bar_rim_program, px, iy + ih, pw, py + ph - iy - ih);
    gfx_quad(bar_rim_program, px, iy, ix - px, ih);
    gfx_quad(bar_rim_program, ix + iw, iy, px + pw - ix - iw, ih);

    float glow = anim_at(&shine_alpha, now);
    if (glow > 0) {
        float reach = BAR_W / 2 - BAR_RADIUS - 130, x = anim_at(&shine_x, now) - DESIGN_W / 2;
        x = (x < -reach ? -reach : x > reach ? reach : x) + DESIGN_W / 2 - 130;
        glUseProgram(shine_program);
        glUniform4f(gfx_loc(shine_program, "u_box"), x, bar_y, 260, 28);
        glUniform1f(gfx_loc(shine_program, "u_alpha"), glow);
        gfx_quad(shine_program, x, bar_y, 260, 28);
    }
}

static void draw_scene(Uint32 now) {
    float shift = anim_at(&slide, now) * LOWER_SHIFT;
    float veiled = anim_at(&veil_fade, now);
    float grid_drop = anim_at(&drop, now) * GRID_DROP;

    if (!(leave_out & NO_CLEAR)) glClear(GL_COLOR_BUFFER_BIT);
    glDisable(GL_BLEND);
    if (leave_out & NO_BACKDROP) {}
    else if (veiled <= 0) copy(&sharp.t, NULL, 1, 1, 0, 0);
    else if (veiled >= 1) copy(&veil.t, NULL, 1, 1, 0, 0);
    else {
        gfx_bind(backdrop_program, 0, "u_wall", sharp.t.tex);
        gfx_bind(backdrop_program, 1, "u_veil", veil.t.tex);
        glUniform1f(gfx_loc(backdrop_program, "u_t"), veiled);
        gfx_quad(backdrop_program, 0, 0, DESIGN_W, DESIGN_H);
    }

    glEnable(GL_BLEND);
    glBlendFuncSeparate(GL_ONE, GL_ONE_MINUS_SRC_ALPHA, GL_ZERO, GL_ONE);
    if (!(leave_out & NO_GRID)) for (int row = ROWS - 1; row >= 1; row--) draw_row(row, shift, grid_drop, now);
    if (!(leave_out & NO_BAR)) draw_bar(BAR_Y - shift, veiled, now);
    glEnable(GL_BLEND);
    if (!(leave_out & NO_BAR_TILES)) draw_row(0, shift, grid_drop, now);

    if (tune.hud && hud.tex && !(leave_out & NO_HUD)) {
        float hud_y = focus_row ? DESIGN_H - 68 - (float)hud.h : 40;
        glUseProgram(fill_program);
        glUniform4f(gfx_loc(fill_program, "u_box"), 40, hud_y, (float)hud.w + 40, (float)hud.h + 28);
        glUniform1f(gfx_loc(fill_program, "u_radius"), 18);
        glUniform4f(gfx_loc(fill_program, "u_color"), 0, 0, 0, 0.55f);
        gfx_quad(fill_program, 40, hud_y, (float)hud.w + 40, (float)hud.h + 28);
        gfx_bind(text_program, 0, "u_tex", hud.tex);
        glUniform4f(gfx_loc(text_program, "u_box"), 60, hud_y + 14, (float)hud.w, (float)hud.h);
        gfx_quad(text_program, 60, hud_y + 14, (float)hud.w, (float)hud.h);
    }
}

static float tile_centre(int col) { return ROWS_EDGE + col * (TILE_W + TILE_GAP) + TILE_W / 2; }

static void focus_on(int row, int col, Uint32 now, int at_once) {
    int was_lower = focus_row > 0, lower = row > 0;
    Tile *old = &tiles[focus_row * COLS + focus_col], *next = &tiles[row * COLS + col];
    focus_row = row;
    focus_col = col;
    anim_go(&old->focus, 0, tune.focus_ms, EASE_OUT, now);
    anim_go(&next->focus, 1, tune.focus_ms, EASE_OUT, now);
    if (!row) anim_go(&shine_x, tile_centre(col), 260, EASE_SCROLL, now);
    anim_go(&shine_alpha, row ? 0 : 1, 200, EASE, now);
    if (lower != was_lower) {
        anim_go(&slide, lower, lower ? tune.down_ms : tune.up_ms, lower ? EASE_DOWN : EASE_UP, now);
        anim_go(&veil_fade, lower, lower ? tune.veil_down_ms : tune.veil_up_ms, EASE_VEIL, now);
        anim_go(&drop, lower, 200, EASE_OUT, now);
    }
    if (at_once) {
        anim_set(&old->focus, 0);
        anim_set(&next->focus, 1);
        anim_set(&shine_x, tile_centre(col));
        anim_set(&shine_alpha, row ? 0 : 1);
        anim_set(&slide, lower);
        anim_set(&veil_fade, lower);
        anim_set(&drop, lower);
    }
}

static void move_begins(const char *name, Uint32 now) {
    if (move.active) return;
    memset(&move, 0, sizeof move);
    move.active = 1;
    move.name = name;
    move.started = move.last = now;
}

static void move_ends(Uint32 now) {
    if (!move.frames) { move.active = 0; return; }
    snprintf(last_move, sizeof last_move, "%s: %d frames of %d, %d late, slowest %u ms, draw %.1f ms (worst %.1f)",
        move.name, move.frames, (int)((float)(now - move.started) / refresh_ms + 0.5f), move.late, move.slowest, move.draw_ms / move.frames, move.draw_worst);
    note("%s, %s", MODE_NAMES[(int)tune.mode], last_move);
    move.active = 0;
    draw_hud_text();
}

static float measured_refresh(SDL_Window *window) {
    enum { SWAPS = 20 };
    for (int i = 0; i < 4; i++) { glClear(GL_COLOR_BUFFER_BIT); SDL_GL_SwapWindow(window); }
    Uint64 began = SDL_GetPerformanceCounter();
    for (int i = 0; i < SWAPS; i++) { glClear(GL_COLOR_BUFFER_BIT); SDL_GL_SwapWindow(window); }
    float ms = (float)((double)(SDL_GetPerformanceCounter() - began) * 1000.0 / (double)SDL_GetPerformanceFrequency() / SWAPS);
    note("refresh: a frame every %.2f ms", ms);
    return ms > 5 && ms < 50 ? ms : 16.7f;
}

static void bench_case(SDL_Window *window, const char *name, int without, float slid, int interval, int finish) {
    enum { FRAMES = 45 };
    leave_out = without;
    anim_set(&slide, slid);
    anim_set(&veil_fade, slid);
    anim_set(&drop, slid);
    SDL_GL_SetSwapInterval(interval);
    double per_ms = 1000.0 / (double)SDL_GetPerformanceFrequency(), draw = 0, worst = 0;
    for (int i = 0; i < 3; i++) { draw_scene(0); SDL_GL_SwapWindow(window); }
    Uint64 began = SDL_GetPerformanceCounter();
    for (int i = 0; i < FRAMES; i++) {
        Uint64 from = SDL_GetPerformanceCounter();
        draw_scene(0);
        if (finish) glFinish();
        double ms = (double)(SDL_GetPerformanceCounter() - from) * per_ms;
        draw += ms;
        if (ms > worst) worst = ms;
        SDL_GL_SwapWindow(window);
    }
    glFinish();
    double frame = (double)(SDL_GetPerformanceCounter() - began) * per_ms / FRAMES;
    note("bench %-34s swap wait %d, finish %d: %.1f ms a frame, draw %.1f ms (worst %.1f)", name, interval, finish, frame, draw / FRAMES, worst);
}

static void bench(SDL_Window *window) {
    static const struct { const char *name; int without; float slid; } cases[] = {
        {"nothing but the clear", NO_BACKDROP | NO_GRID | NO_BAR | NO_BAR_TILES | NO_HUD, 0},
        {"wallpaper only", NO_GRID | NO_BAR | NO_BAR_TILES | NO_HUD, 0},
        {"wallpaper only, no clear", NO_CLEAR | NO_GRID | NO_BAR | NO_BAR_TILES | NO_HUD, 0},
        {"wallpaper and bar", NO_GRID | NO_BAR_TILES | NO_HUD, 0},
        {"wallpaper, bar and its tiles", NO_GRID | NO_HUD, 0},
        {"all, at rest", 0, 0},
        {"mid slide: two pictures mixed, only", NO_GRID | NO_BAR | NO_BAR_TILES | NO_HUD, 0.5f},
        {"all, mid slide", 0, 0.5f},
        {"all, down in the grid", 0, 1},
    };
    for (size_t i = 0; i < sizeof cases / sizeof *cases; i++) bench_case(window, cases[i].name, cases[i].without, cases[i].slid, 0, 1);
    bench_case(window, "all, at rest", 0, 0, 0, 0);
    bench_case(window, "all, at rest", 0, 0, 1, 0);
    bench_case(window, "all, at rest", 0, 0, 1, 1);
    bench_case(window, "wallpaper only", NO_GRID | NO_BAR | NO_BAR_TILES | NO_HUD, 0, 1, 0);
    bench_case(window, "all, mid slide", 0, 0.5f, 1, 0);
    leave_out = 0;
    SDL_GL_SetSwapInterval(1);
    focus_on(0, 0, SDL_GetTicks(), 1);
}

#ifdef LUMEN_WEBOS
static struct wl_webos_shell *webos_shell;

static void shell_global(void *data, struct wl_registry *reg, uint32_t name, const char *iface, uint32_t version) {
    (void)data; (void)version;
    if (!strcmp(iface, "wl_webos_shell")) webos_shell = wl_registry_bind(reg, name, &wl_webos_shell_interface, 1);
}
static void shell_removed(void *data, struct wl_registry *reg, uint32_t name) { (void)data; (void)reg; (void)name; }

static void claim_window(SDL_Window *window) {
    SDL_SysWMinfo info;
    SDL_VERSION(&info.version);
    if (!SDL_GetWindowWMInfo(window, &info) || info.subsystem != SDL_SYSWM_WAYLAND) { note("window: no wayland info"); return; }
    struct wl_registry *registry = wl_display_get_registry(info.info.wl.display);
    static const struct wl_registry_listener listener = {shell_global, shell_removed};
    wl_registry_add_listener(registry, &listener, NULL);
    wl_display_roundtrip(info.info.wl.display);
    wl_registry_destroy(registry);
    if (!webos_shell) { note("window: no wl_webos_shell"); return; }
    struct wl_webos_shell_surface *shell_surface = wl_webos_shell_get_shell_surface(webos_shell, info.info.wl.surface);
    wl_webos_shell_surface_set_property(shell_surface, "_WEBOS_ACCESS_POLICY_KEYS_BACK", "true");
    wl_display_flush(info.info.wl.display);
}
#endif

static int closes(SDL_Keycode key) { return key == SDLK_ESCAPE || key == SDLK_BACKSPACE || key == SDLK_AC_BACK || key == SDLK_q; }

int main(int argc, char **argv) {
    Uint32 started_at = SDL_GetTicks();
    setenv("EGL_PLATFORM", "wayland", 0);
    setenv("XDG_RUNTIME_DIR", "/tmp/xdg", 0);
    util_init();
    const char *shot_path = argc > 2 && !strcmp(argv[1], "--shot") ? argv[2] : NULL;

    SDL_SetHint("SDL_WEBOS_ACCESS_POLICY_KEYS_BACK", "true");
    if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_EVENTS) != 0) { note("SDL_Init: %s", SDL_GetError()); return 1; }
    IMG_Init(IMG_INIT_JPG | IMG_INIT_PNG);
    TTF_Init();
    char *base = SDL_GetBasePath();
    if (base) { snprintf(app_dir, sizeof app_dir, "%s", base); SDL_free(base); }
    size_t n = strlen(app_dir);
    if (n > 1 && app_dir[n - 1] == '/') app_dir[n - 1] = 0;
    read_tuning();

    SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_ES);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 2);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 0);
    SDL_GL_SetAttribute(SDL_GL_RED_SIZE, 8);
    SDL_GL_SetAttribute(SDL_GL_GREEN_SIZE, 8);
    SDL_GL_SetAttribute(SDL_GL_BLUE_SIZE, 8);
    SDL_GL_SetAttribute(SDL_GL_ALPHA_SIZE, 8);
    SDL_GL_SetAttribute(SDL_GL_DOUBLEBUFFER, 1);
#ifdef LUMEN_WEBOS
    SDL_DisplayMode display = {0};
    if (SDL_GetCurrentDisplayMode(0, &display) != 0 || !display.w) { display.w = DESIGN_W; display.h = DESIGN_H; }
    SDL_Window *window = SDL_CreateWindow("Lumen Native Test", 0, 0, display.w, display.h, SDL_WINDOW_OPENGL | SDL_WINDOW_FULLSCREEN | SDL_WINDOW_SHOWN);
#else
    SDL_Window *window = SDL_CreateWindow("Lumen Native Test", 0, 0, DESIGN_W, DESIGN_H, SDL_WINDOW_OPENGL | SDL_WINDOW_SHOWN);
#endif
    if (!window) { note("window: %s", SDL_GetError()); return 1; }
#ifdef LUMEN_WEBOS
    claim_window(window);
#endif
    SDL_GLContext context = SDL_GL_CreateContext(window);
    if (!context) { note("context: %s", SDL_GetError()); return 1; }
    SDL_GL_SetSwapInterval(1);
    int screen_w, screen_h;
    SDL_GL_GetDrawableSize(window, &screen_w, &screen_h);
    note("open: %dx%d, %s, %s, %u ms after start", screen_w, screen_h, (const char *)glGetString(GL_RENDERER), (const char *)glGetString(GL_VERSION), SDL_GetTicks() - started_at);

    gfx_init(screen_w, screen_h);
    copy_program = gfx_program("copy", COPY_FRAGMENT);
    blur_program = gfx_program("blur", BLUR_FRAGMENT);
    backdrop_program = gfx_program("backdrop", BACKDROP_FRAGMENT);
    lens_program = gfx_program("lens", LENS_FRAGMENT);
    gloss_program = gfx_program("gloss", GLOSS_FRAGMENT);
    bar_middle_program = gfx_program("bar middle", BAR_MIDDLE_FRAGMENT);
    bar_rim_program = gfx_program("bar rim", BAR_RIM_FRAGMENT);
    shine_program = gfx_program("shine", SHINE_FRAGMENT);
    tile_program = gfx_program("tile", TILE_FRAGMENT);
    fill_program = gfx_program("fill", FILL_FRAGMENT);
    text_program = gfx_program("text", TEXT_FRAGMENT);
    if (!build_backdrops(screen_w, screen_h)) return 1;
    load_tiles();
    bake_tiles();
    bake_bar();
    open_font();
    anim_set(&shine_x, tile_centre(0));
    focus_on(0, 0, SDL_GetTicks(), 1);

    if (shot_path) {
        focus_on(argc > 3 ? atoi(argv[3]) % ROWS : 0, argc > 4 ? atoi(argv[4]) % COLS : 0, 0, 1);
        if (argc > 5) tune.mode = (float)(atoi(argv[5]) % GLASS_MODES);
        if (argc > 6) { anim_set(&slide, (float)atof(argv[6])); anim_set(&veil_fade, (float)atof(argv[6])); }
        draw_hud_text();
        draw_scene(0);
        glFinish();
        SDL_Surface *shot = gfx_read_screen();
        SDL_Rect crop;
        const char *zoom = getenv("LUMEN_SHOT_ZOOM");
        if (shot && zoom && sscanf(zoom, "%d,%d,%d,%d", &crop.x, &crop.y, &crop.w, &crop.h) == 4) {
            SDL_Surface *close_up = SDL_CreateRGBSurfaceWithFormat(0, crop.w * 3, crop.h * 3, 32, SDL_PIXELFORMAT_RGBA32);
            SDL_SetSurfaceBlendMode(shot, SDL_BLENDMODE_NONE);
            SDL_BlitScaled(shot, &crop, close_up, NULL);
            shot = close_up;
        }
        int failed = !shot || IMG_SavePNG(shot, shot_path) != 0;
        printf("%s %s\n", failed ? "could not save" : "saved", shot_path);
        return failed;
    }

    draw_hud_text();
    refresh_ms = measured_refresh(window);
    if (access("/tmp/ntest-bench", F_OK) == 0) bench(window);
    note("first frame ready %u ms after start, RSS %ld kB", SDL_GetTicks() - started_at, status_kb("VmRSS"));
    int redraw = 1, running = 1;
    double per_ms = 1000.0 / (double)SDL_GetPerformanceFrequency();

    while (running) {
        SDL_Event e;
        while (SDL_PollEvent(&e)) {
            if (e.type == SDL_QUIT) { note("close: quit"); running = 0; }
            if (e.type == SDL_WINDOWEVENT) redraw = 1;
            if (e.type != SDL_KEYDOWN || e.key.repeat) continue;
            SDL_Keycode key = e.key.keysym.sym;
            Uint32 now = SDL_GetTicks();
            int glass_before[4] = {(int)tune.glass_halvings, (int)tune.glass_passes, (int)tune.veil_halvings, (int)tune.veil_passes};
            char wallpaper_before[sizeof tune.wallpaper];
            memcpy(wallpaper_before, tune.wallpaper, sizeof wallpaper_before);
            int mode_before = (int)tune.mode;
            read_tuning();
            int glass_after[4] = {(int)tune.glass_halvings, (int)tune.glass_passes, (int)tune.veil_halvings, (int)tune.veil_passes};
            if (memcmp(glass_before, glass_after, sizeof glass_before) || strcmp(wallpaper_before, tune.wallpaper)) build_backdrops(screen_w, screen_h);
            if (tune.bezel != baked_bezel) bake_bar();
            redraw = 1;
            if (closes(key)) { note("close: key %d", (int)key); running = 0; }
            else if (key == SDLK_LEFT && focus_col > 0) { move_begins("Left", now); focus_on(focus_row, focus_col - 1, now, 0); }
            else if (key == SDLK_RIGHT && focus_col < COLS - 1) { move_begins("Right", now); focus_on(focus_row, focus_col + 1, now, 0); }
            else if (key == SDLK_DOWN && focus_row < ROWS - 1) { move_begins("Down", now); focus_on(focus_row + 1, focus_col, now, 0); }
            else if (key == SDLK_UP && focus_row > 0) { move_begins("Up", now); focus_on(focus_row - 1, focus_col, now, 0); }
            else if (key == SDLK_RETURN || key == SDLK_KP_ENTER) tune.mode = (float)(((int)tune.mode + 1) % GLASS_MODES);
            else note("key %d, scancode %d", (int)key, (int)e.key.keysym.scancode);
            if ((int)tune.mode != mode_before) draw_hud_text();
        }
        if (!running) break;

        Uint32 now = SDL_GetTicks();
        int moving = anything_moving(now);
        if (!moving && move.active) { move_ends(now); redraw = 1; }
        if (!moving && !redraw) {
            SDL_WaitEventTimeout(NULL, IDLE_WAIT_MS);
            continue;
        }
        redraw = 0;

        Uint64 draw_from = SDL_GetPerformanceCounter();
        draw_scene(now);
        if (tune.finish) glFinish();
        double draw_ms = (double)(SDL_GetPerformanceCounter() - draw_from) * per_ms;
        SDL_GL_SwapWindow(window);

        Uint32 after = SDL_GetTicks();
        if (move.active) {
            Uint32 gap = after - move.last;
            move.frames++;
            if ((float)gap > refresh_ms * 1.5f) move.late++;
            if (gap > move.slowest) move.slowest = gap;
            move.draw_ms += draw_ms;
            if (draw_ms > move.draw_worst) move.draw_worst = draw_ms;
            move.last = after;
        }
    }

    note("RSS %ld kB, peak %ld kB", status_kb("VmRSS"), status_kb("VmHWM"));
    SDL_GL_DeleteContext(context);
    SDL_DestroyWindow(window);
    TTF_Quit();
    IMG_Quit();
    SDL_Quit();
    return 0;
}
