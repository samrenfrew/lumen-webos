#include "status.h"
#include "util.h"
#include <SDL_image.h>
#include <SDL_ttf.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

enum { CLOCK, PERIOD, LINE, CAST_TEMP, CAST_TIME, CAST_BIG, CAST_NOTE, ROLES };
enum { RAIN_LIKELY = 50, LABELS = 11 };

static const struct { const char *sf, *lg; int size; } ROLE[ROLES] = {
    {"sf-600-clock", "LG_Smart_UI-SemiBold", 120},
    {"sf-400", "LG_Smart_UI-Regular", 32},
    {"sf-500", "LG_Smart_UI-Regular", 32},
    {"sf-600", "LG_Smart_UI-SemiBold", 26},
    {"sf-400", "LG_Smart_UI-Regular", 18},
    {"sf-600", "LG_Smart_UI-SemiBold", 56},
    {"sf-500", "LG_Smart_UI-Regular", 22},
};
static const char *const LG_FONTS = "/usr/share/fonts";
static const char *const DEVANAGARI = "LG_Smart_UI_Devanagari";
static const char *const LABEL_KEYS[LABELS] = {"Clear", "Partly cloudy", "Cloudy", "Fog", "Rain", "Snow", "Showers", "Storm", "High", "Low", "Rain from"};
static const char *const KIND_NAMES[] = {"clear", "partly", "cloudy", "fog", "rain", "snow", "storm"};
static const int ICON_SIZES[] = {30, 32, 36, 50};

typedef struct { float dx, dy, sigma, alpha; } Shadow;
static const Shadow CLOCK_SHADOW[] = {{0, 4, 12, 0.28f}};
static const Shadow SCRIM_STATUS[] = {{0, 1, 8, 0.5f}, {0, 0, 1, 0.22f}};
static const Shadow LIGHT_SHADOW[] = {{0, 1, 4, 0.45f}};
static const Shadow STATUS_ICON_SHADOW[] = {{0, 2, 3, 0.6f}};
static const Shadow CAST_ICON_SHADOW[] = {{0, 1, 3, 0.35f}};
static const Shadow PILL_SHADOW[] = {{0, 1, 3, 0.25f}};

enum { INK_DARK = 0xEEEEEE, WHITE = 0xFFFFFF, RAIN_TEXT = 0xA8D8FF, RAIN_BLUE = 0x7CC4FF };

typedef struct { TTF_Font *main, *alt[2]; int alt_tried; } Font;

static char dir[256];
static Settings settings;
static Font fonts[ROLES];
static int sf_clock;
static float k;
static SDL_Surface *icons[7][2][4];
static struct { char date[64], weekday[7][48], month[12][48], label[LABELS][64]; } words;
static int (*hours_ahead)(const Weather *, time_t, int, WeatherHour *) = weather_hours_ahead;

typedef struct { int w, h, x0, y0, x1, y1; Uint8 *a; } Mask;
static SDL_Surface *band;
static Mask mask;
static float band_top;

static TTF_Font *open_font(const char *folder, const char *name, int size) {
    char path[384];
    snprintf(path, sizeof path, "%s/%s.ttf", folder, name);
    TTF_Font *f = TTF_OpenFont(path, size);
    if (f) TTF_SetFontHinting(f, TTF_HINTING_LIGHT);
    return f;
}

static void close_fonts(void) {
    for (int r = 0; r < ROLES; r++) {
        if (fonts[r].main) TTF_CloseFont(fonts[r].main);
        for (int i = 0; i < 2; i++) if (fonts[r].alt[i]) TTF_CloseFont(fonts[r].alt[i]);
    }
    memset(fonts, 0, sizeof fonts);
}

static int open_fonts(float scale) {
    close_fonts();
    k = scale;
    char own[300];
    snprintf(own, sizeof own, "%s/fonts", dir);
    int any = 0;
    for (int r = 0; r < ROLES; r++) {
        int size = (int)lroundf(ROLE[r].size * k);
        fonts[r].main = open_font(own, ROLE[r].sf, size);
        if (r == CLOCK) sf_clock = fonts[r].main != NULL;
        if (!fonts[r].main) fonts[r].main = open_font(LG_FONTS, ROLE[r].lg, size);
        any |= fonts[r].main != NULL;
    }
    return any;
}

static TTF_Font *font_for(int role, int cp) {
    Font *f = &fonts[role];
    if (TTF_GlyphIsProvided(f->main, (Uint16)cp)) return f->main;
    if (!f->alt_tried) {
        f->alt_tried = 1;
        int size = (int)lroundf(ROLE[role].size * k);
        f->alt[0] = open_font(LG_FONTS, ROLE[role].lg, size);
        f->alt[1] = open_font(LG_FONTS, DEVANAGARI, size);
    }
    for (int i = 0; i < 2; i++) if (f->alt[i] && TTF_GlyphIsProvided(f->alt[i], (Uint16)cp)) return f->alt[i];
    return f->main;
}

static void load_words(void) {
    for (int i = 0; i < 7; i++) {
        struct tm t = {.tm_wday = i};
        strftime(words.weekday[i], sizeof words.weekday[i], "%A", &t);
    }
    for (int i = 0; i < 12; i++) {
        struct tm t = {.tm_mon = i};
        strftime(words.month[i], sizeof words.month[i], "%B", &t);
    }
    snprintf(words.date, sizeof words.date, "%%W %%M %%D");
    for (int i = 0; i < LABELS; i++) snprintf(words.label[i], sizeof words.label[i], "%s", LABEL_KEYS[i]);

    const char *pref = strcmp(settings.language, "auto") ? settings.language : settings.tv_locale;
    char code[3] = {(char)(pref[0] | 0x20), pref[0] ? (char)(pref[1] | 0x20) : 0, 0}, path[320];
    snprintf(path, sizeof path, "%s/assets/strings/%s.txt", dir, code);
    char *text = read_file(path, NULL);
    if (!text) {
        snprintf(path, sizeof path, "%s/assets/strings/en.txt", dir);
        text = read_file(path, NULL);
    }
    if (!text) { note("status: no strings in %s/assets/strings", dir); return; }
    for (char *line = strtok(text, "\n"); line; line = strtok(NULL, "\n")) {
        char *eq = strchr(line, '=');
        if (!eq) continue;
        *eq = 0;
        const char *v = eq + 1;
        int n;
        if (!strcmp(line, "date")) snprintf(words.date, sizeof words.date, "%s", v);
        else if (sscanf(line, "weekday%d", &n) == 1 && n >= 0 && n < 7) snprintf(words.weekday[n], sizeof words.weekday[n], "%s", v);
        else if (sscanf(line, "month%d", &n) == 1 && n >= 0 && n < 12) snprintf(words.month[n], sizeof words.month[n], "%s", v);
        else for (int i = 0; i < LABELS; i++) if (!strcmp(line, LABEL_KEYS[i])) snprintf(words.label[i], sizeof words.label[i], "%s", v);
    }
    free(text);
}

static const char *tr(const char *english) {
    for (int i = 0; i < LABELS; i++) if (!strcmp(english, LABEL_KEYS[i])) return words.label[i];
    return english;
}

int status_init(const char *app_dir, const Settings *s) {
    snprintf(dir, sizeof dir, "%s", app_dir);
    settings = *s;
    load_words();
    int ok = open_fonts(1);
    note("status: font %s, language %s", sf_clock ? "SF" : ok ? "LG Smart UI" : "none", settings.language);
    return ok;
}

void status_quit(void) {
    close_fonts();
    for (int i = 0; i < 7; i++)
        for (int d = 0; d < 2; d++)
            for (int z = 0; z < 4; z++)
                if (icons[i][d][z]) { SDL_FreeSurface(icons[i][d][z]); icons[i][d][z] = NULL; }
}

static SDL_Surface *icon(WeatherKind kind, int day, int css_size) {
    int z = 0;
    while (z < 3 && ICON_SIZES[z] != css_size) z++;
    SDL_Surface **slot = &icons[kind][day ? 1 : 0][z];
    if (!*slot) {
        char path[384];
        snprintf(path, sizeof path, "%s/assets/icons/%s-%s-%d.png", dir, KIND_NAMES[kind], day ? "day" : "night", css_size);
        SDL_Surface *raw = IMG_Load(path);
        if (!raw) { note("status: %s", IMG_GetError()); return NULL; }
        *slot = SDL_ConvertSurfaceFormat(raw, SDL_PIXELFORMAT_ARGB8888, 0);
        SDL_FreeSurface(raw);
    }
    return *slot;
}

static void mask_touch(int x0, int y0, int x1, int y1) {
    if (x0 < mask.x0) mask.x0 = x0;
    if (y0 < mask.y0) mask.y0 = y0;
    if (x1 > mask.x1) mask.x1 = x1;
    if (y1 > mask.y1) mask.y1 = y1;
}

static void mask_reset(void) {
    for (int y = mask.y0; y < mask.y1; y++) memset(mask.a + (size_t)y * mask.w + mask.x0, 0, (size_t)(mask.x1 - mask.x0));
    mask.x0 = mask.w; mask.y0 = mask.h; mask.x1 = 0; mask.y1 = 0;
}

static void mask_surface(SDL_Surface *s, int x, int y, int scaled_w, int scaled_h) {
    int x0 = x < 0 ? 0 : x, y0 = y < 0 ? 0 : y;
    int x1 = x + scaled_w > mask.w ? mask.w : x + scaled_w, y1 = y + scaled_h > mask.h ? mask.h : y + scaled_h;
    if (x0 >= x1 || y0 >= y1) return;
    for (int j = y0; j < y1; j++) {
        const Uint32 *row = (const Uint32 *)((const Uint8 *)s->pixels + (size_t)((j - y) * s->h / scaled_h) * s->pitch);
        Uint8 *out = mask.a + (size_t)j * mask.w;
        for (int i = x0; i < x1; i++) {
            int g = row[(i - x) * s->w / scaled_w] >> 24;
            out[i] = (Uint8)(out[i] + g - out[i] * g / 255);
        }
    }
    mask_touch(x0, y0, x1, y1);
}

static int decode(const char *s, int *cps, int max) {
    const unsigned char *p = (const unsigned char *)s;
    int n = 0;
    while (*p && n < max) {
        int c = *p, len = c < 0x80 ? 1 : c < 0xE0 ? 2 : c < 0xF0 ? 3 : 4;
        int cp = len == 1 ? c : len == 2 ? c & 0x1F : len == 3 ? c & 0x0F : c & 0x07;
        int i = 1;
        for (; i < len && (p[i] & 0xC0) == 0x80; i++) cp = (cp << 6) | (p[i] & 0x3F);
        p += i;
        cps[n++] = cp;
    }
    for (int i = 0; i + 1 < n; i++)
        if (cps[i + 1] == 0x093F) { cps[i + 1] = cps[i]; cps[i] = 0x093F; i++; }
    return n;
}

static float run(int role, const char *text, int draw, float x, float base) {
    int cps[160], n = decode(text, cps, 160);
    int pen = (int)lroundf(x * k), start = pen, line = (int)lroundf(base * k - band_top);
    SDL_Color white = {255, 255, 255, 255};
    for (int i = 0; i < n; i++) {
        if (cps[i] > 0xFFFF) continue;
        TTF_Font *f = font_for(role, cps[i]);
        int minx, maxx, miny, maxy, advance;
        if (!f || TTF_GlyphMetrics(f, (Uint16)cps[i], &minx, &maxx, &miny, &maxy, &advance) < 0) continue;
        if (draw && maxx > minx && cps[i] != ' ') {
            SDL_Surface *g = TTF_RenderGlyph_Blended(f, (Uint16)cps[i], white);
            if (g) {
                mask_surface(g, pen - (minx < 0 ? -minx : 0), line - TTF_FontAscent(f), g->w, g->h);
                SDL_FreeSurface(g);
            }
        }
        pen += advance;
    }
    return (pen - start) / k;
}

static float text_w(int role, const char *text) { return run(role, text, 0, 0, 0); }
static void text_at(int role, const char *text, float x, float base) { run(role, text, 1, x, base); }
static float ascent(int role) { return TTF_FontAscent(fonts[role].main) / k; }
static float descent(int role) { return -TTF_FontDescent(fonts[role].main) / k; }
static float normal_line(int role) { return TTF_FontLineSkip(fonts[role].main) / k; }
static float baseline_in(int role, float line_h) { return (line_h - ascent(role) - descent(role)) / 2 + ascent(role); }

static void icon_at(WeatherKind kind, int day, int css_size, float x, float y) {
    SDL_Surface *s = icon(kind, day, css_size);
    int size = (int)lroundf(css_size * k);
    if (s) mask_surface(s, (int)lroundf(x * k), (int)lroundf(y * k - band_top), size, size);
}

typedef struct { float x, y, w, h, top_r, bottom_r; } Box;

static int in_box(const Box *b, float px, float py) {
    if (px < b->x || px >= b->x + b->w || py < b->y || py >= b->y + b->h) return 0;
    float r = py < b->y + b->h / 2 ? b->top_r : b->bottom_r;
    float cx = px < b->x + r ? b->x + r : px > b->x + b->w - r ? b->x + b->w - r : px;
    float cy = py < b->y + r ? b->y + r : py > b->y + b->h - r ? b->y + b->h - r : py;
    return (px - cx) * (px - cx) + (py - cy) * (py - cy) <= r * r;
}

static void box_at(Box css, const Box *clip_css) {
    Box b = {css.x * k, css.y * k - band_top, css.w * k, css.h * k, css.top_r * k, css.bottom_r * k}, c = b;
    if (clip_css) c = (Box){clip_css->x * k, clip_css->y * k - band_top, clip_css->w * k, clip_css->h * k, clip_css->top_r * k, clip_css->bottom_r * k};
    int x0 = (int)floorf(b.x), y0 = (int)floorf(b.y), x1 = (int)ceilf(b.x + b.w), y1 = (int)ceilf(b.y + b.h);
    if (x0 < 0) x0 = 0;
    if (y0 < 0) y0 = 0;
    if (x1 > mask.w) x1 = mask.w;
    if (y1 > mask.h) y1 = mask.h;
    if (x0 >= x1 || y0 >= y1) return;
    for (int j = y0; j < y1; j++)
        for (int i = x0; i < x1; i++) {
            int hits = 0;
            for (int sy = 0; sy < 4; sy++)
                for (int sx = 0; sx < 4; sx++) {
                    float px = i + (sx + 0.5f) / 4, py = j + (sy + 0.5f) / 4;
                    hits += in_box(&b, px, py) && in_box(&c, px, py);
                }
            Uint8 *a = &mask.a[(size_t)j * mask.w + i];
            int g = hits * 255 / 16;
            *a = (Uint8)(*a + g - *a * g / 255);
        }
    mask_touch(x0, y0, x1, y1);
}

static void blend(int x, int y, Uint32 rgb, float alpha) {
    if (alpha <= 0 || x < 0 || y < 0 || x >= band->w || y >= band->h) return;
    Uint32 *p = (Uint32 *)((Uint8 *)band->pixels + (size_t)y * band->pitch) + x;
    int a = (int)(alpha * 255 + 0.5f), keep = 255 - a;
    int da = *p >> 24, dr = (*p >> 16) & 255, dg = (*p >> 8) & 255, db = *p & 255;
    int r = (rgb >> 16) & 255, g = (rgb >> 8) & 255, b = rgb & 255;
    *p = (Uint32)((a + da * keep / 255) << 24 | ((r * a + dr * keep) / 255) << 16 | ((g * a + dg * keep) / 255) << 8 | (b * a + db * keep) / 255);
}

static void box_blur(float *v, float *tmp, int n, int stride, int r) {
    for (int i = 0; i < n; i++) tmp[i] = v[(size_t)i * stride];
    float sum = 0, scale = 1.0f / (2 * r + 1);
    for (int i = 0; i < r && i < n; i++) sum += tmp[i];
    for (int i = 0; i < n; i++) {
        if (i + r < n) sum += tmp[i + r];
        v[(size_t)i * stride] = sum * scale;
        if (i - r >= 0) sum -= tmp[i - r];
    }
}

static void paint_shadow(const Shadow *s, float opacity, int outside_only) {
    float sigma = s->sigma * k;
    int r = (int)lroundf((sqrtf(1 + 4 * sigma * sigma) - 1) / 2), pad = 3 * r + 1;
    int ox = mask.x0 - pad, oy = mask.y0 - pad, w = mask.x1 - mask.x0 + 2 * pad, h = mask.y1 - mask.y0 + 2 * pad;
    float *v = calloc((size_t)w * h, sizeof *v), *tmp = malloc(sizeof *tmp * (size_t)(w > h ? w : h));
    if (!v || !tmp) { free(v); free(tmp); return; }
    for (int j = mask.y0; j < mask.y1; j++)
        for (int i = mask.x0; i < mask.x1; i++) v[(size_t)(j - oy) * w + (i - ox)] = mask.a[(size_t)j * mask.w + i] / 255.0f;
    for (int pass = 0; pass < 3 && r > 0; pass++) {
        for (int j = 0; j < h; j++) box_blur(v + (size_t)j * w, tmp, w, 1, r);
        for (int i = 0; i < w; i++) box_blur(v + i, tmp, h, w, r);
    }
    int dx = (int)lroundf(s->dx * k), dy = (int)lroundf(s->dy * k);
    for (int j = 0; j < h; j++)
        for (int i = 0; i < w; i++) {
            int bx = ox + i + dx, by = oy + j + dy;
            float a = v[(size_t)j * w + i] * s->alpha * opacity;
            if (outside_only && bx >= 0 && by >= 0 && bx < mask.w && by < mask.h) a *= 1 - mask.a[(size_t)by * mask.w + bx] / 255.0f;
            blend(bx, by, 0, a);
        }
    free(v);
    free(tmp);
}

static void paint(Uint32 rgb, float opacity, const Shadow *shadows, int count, int outside_only) {
    if (mask.x1 <= mask.x0) return;
    for (int i = count - 1; i >= 0; i--) paint_shadow(&shadows[i], opacity, outside_only);
    for (int j = mask.y0; j < mask.y1; j++)
        for (int i = mask.x0; i < mask.x1; i++) {
            Uint8 a = mask.a[(size_t)j * mask.w + i];
            if (a) blend(i, j, rgb, a / 255.0f * opacity);
        }
    mask_reset();
}

#define SHADOWS(list) list, (int)(sizeof list / sizeof list[0])

static void hour_text(const WeatherHour *h, int clock_24h, char *out, size_t n) {
    int hour = atoi(h->t + 11);
    if (clock_24h) snprintf(out, n, "%02d:00", hour);
    else snprintf(out, n, "%d %s", hour % 12 ? hour % 12 : 12, hour < 12 ? "AM" : "PM");
}

static void date_text(const struct tm *t, char *out, size_t n) {
    size_t at = 0;
    for (const char *p = words.date; *p && at + 1 < n; p++) {
        char day[4];
        const char *put = NULL;
        if (p[0] == '%' && p[1] == 'W') put = words.weekday[t->tm_wday];
        else if (p[0] == '%' && p[1] == 'M') put = words.month[t->tm_mon];
        else if (p[0] == '%' && p[1] == 'D') { snprintf(day, sizeof day, "%d", t->tm_mday); put = day; }
        if (put) { at += (size_t)snprintf(out + at, n - at, "%s", put); p++; if (at >= n) at = n - 1; }
        else out[at++] = *p;
    }
    out[at] = 0;
}

typedef struct { float hours_w, colon_w, minutes_w, period_w, w, below, height; char hours[4], minutes[4], period[3]; } ClockLayout;

static ClockLayout clock_layout(time_t now) {
    ClockLayout c = {0};
    struct tm t = *localtime(&now);
    int h24 = settings_clock_24h(&settings);
    snprintf(c.hours, sizeof c.hours, h24 ? "%02d" : "%d", h24 ? t.tm_hour : (t.tm_hour % 12 ? t.tm_hour % 12 : 12));
    snprintf(c.minutes, sizeof c.minutes, "%02d", t.tm_min);
    if (!h24 && settings.show_am_pm) snprintf(c.period, sizeof c.period, "%s", t.tm_hour < 12 ? "AM" : "PM");
    c.hours_w = text_w(CLOCK, c.hours);
    c.colon_w = sf_clock ? text_w(CLOCK, ":") + 2 * 0.02f * 120 : 0.34f * 120;
    c.minutes_w = text_w(CLOCK, c.minutes);
    c.period_w = c.period[0] ? 10 + text_w(PERIOD, c.period) : 0;
    c.w = c.hours_w + c.colon_w + c.minutes_w + c.period_w;
    float line_h = 0.95f * 120, base = baseline_in(CLOCK, line_h);
    float strut_a = roundf(ascent(CLOCK) * 16 / 120), strut_d = roundf(descent(CLOCK) * 16 / 120);
    float strut_above = (20 - strut_a - strut_d) / 2 + strut_a, strut_below = (20 - strut_a - strut_d) / 2 + strut_d;
    c.below = line_h - base > strut_below ? line_h - base : strut_below;
    c.height = (base > strut_above ? base : strut_above) + c.below;
    return c;
}

static void draw_clock(const ClockLayout *c, float x, float bottom) {
    float line_h = 0.95f * 120, base = bottom - c->below, top = base - baseline_in(CLOCK, line_h);
    text_at(CLOCK, c->hours, x, base);
    float colon_x = x + c->hours_w;
    if (sf_clock) text_at(CLOCK, ":", colon_x + 0.02f * 120, base);
    else text_at(CLOCK, ":", colon_x + (c->colon_w - text_w(CLOCK, ":")) / 2, base);
    text_at(CLOCK, c->minutes, colon_x + c->colon_w, base);
    if (c->period[0]) {
        float period_line = 0.95f * 32, period_top = top + line_h - 14 - period_line;
        text_at(PERIOD, c->period, x + c->hours_w + c->colon_w + c->minutes_w + 10, period_top + baseline_in(PERIOD, period_line));
    }
    if (settings.dark) paint(INK_DARK, 1, SHADOWS(CLOCK_SHADOW), 0);
    else paint(WHITE, 1, SHADOWS(LIGHT_SHADOW), 0);
}

static void paint_text(Uint32 dark_rgb, float opacity) {
    if (settings.dark) paint(dark_rgb, opacity, SHADOWS(SCRIM_STATUS), 0);
    else paint(WHITE, opacity, SHADOWS(LIGHT_SHADOW), 0);
}

enum { SMALL_COL = 72, SMALL_GAP = 6, LARGE_COL = 76 };

static float small_width(int n) { return n * SMALL_COL + (n - 1) * SMALL_GAP; }

static void draw_small(const WeatherHour *hours, int n, float left, float bottom) {
    int h24 = settings_clock_24h(&settings);
    float height = 30 + 4 + 26 * 1.2f + 4 + 18 * 1.2f + 8 + 5, top = bottom - height;
    char text[16];
    for (int i = 0; i < n; i++) icon_at(hours[i].kind, hours[i].day, 30, left + i * (SMALL_COL + SMALL_GAP) + (SMALL_COL - 30) / 2.0f, top);
    paint(WHITE, 1, SHADOWS(CAST_ICON_SHADOW), 0);
    for (int i = 0; i < n; i++) {
        snprintf(text, sizeof text, "%d°", hours[i].temp);
        text_at(CAST_TEMP, text, left + i * (SMALL_COL + SMALL_GAP) + (SMALL_COL - text_w(CAST_TEMP, text)) / 2, top + 34 + baseline_in(CAST_TEMP, 26 * 1.2f));
    }
    paint_text(WHITE, 1);
    for (int i = 0; i < n; i++) {
        hour_text(&hours[i], h24, text, sizeof text);
        text_at(CAST_TIME, text, left + i * (SMALL_COL + SMALL_GAP) + (SMALL_COL - text_w(CAST_TIME, text)) / 2, top + 34 + 26 * 1.2f + 4 + baseline_in(CAST_TIME, 18 * 1.2f));
    }
    paint_text(WHITE, 0.6f);
    float pill_y = bottom - 5;
    for (int i = 0; i < n; i++) box_at((Box){left + i * (SMALL_COL + SMALL_GAP) + (SMALL_COL - 44) / 2.0f, pill_y, 44, 5, 3, 3}, NULL);
    paint(WHITE, 0.24f, SHADOWS(PILL_SHADOW), 1);
    for (int i = 0; i < n; i++) {
        Box track = {left + i * (SMALL_COL + SMALL_GAP) + (SMALL_COL - 44) / 2.0f, pill_y, 44, 5, 3, 3};
        if (hours[i].rain <= 0) continue;
        box_at((Box){track.x, track.y, 44 * hours[i].rain / 100.0f, 5, 3, 3}, &track);
        paint(RAIN_BLUE, hours[i].rain >= RAIN_LIKELY ? 1 : 0.6f, NULL, 0, 0);
    }
}

typedef struct { float now_w, now_h, height, width; char big[12], meta[96], note[96]; const char *cond; const WeatherHour *wet; } LargeLayout;

static float large_hours_h(void) { return 18 * 1.2f + 2 + 3 + 32 + 3 + 26 * 1.2f + 8 + 46; }

static LargeLayout large_layout(const Weather *w, const WeatherHour *hours, int n) {
    LargeLayout l = {0};
    snprintf(l.big, sizeof l.big, "%d°", w->temp);
    l.cond = w->label[0] ? tr(w->label) : NULL;
    if (w->has_range) snprintf(l.meta, sizeof l.meta, "%s %d°\xE2\x80\x82%s %d°", tr("High"), w->high, tr("Low"), w->low);
    for (int i = 0; i < n && !l.wet; i++) if (hours[i].rain >= RAIN_LIKELY) l.wet = &hours[i];
    if (l.wet) {
        char when[16];
        hour_text(l.wet, settings_clock_24h(&settings), when, sizeof when);
        snprintf(l.note, sizeof l.note, "%s %s", tr("Rain from"), when);
    }
    l.now_w = 50 + 10 + text_w(CAST_BIG, l.big);
    l.now_h = 56;
    if (l.cond) { float cw = text_w(CAST_TEMP, l.cond); if (cw > l.now_w) l.now_w = cw; l.now_h += 10 + normal_line(CAST_TEMP); }
    if (l.meta[0]) { float mw = text_w(CAST_NOTE, l.meta); if (mw > l.now_w) l.now_w = mw; l.now_h += 2 + normal_line(CAST_NOTE); }
    if (l.note[0]) { float nw = text_w(CAST_NOTE, l.note); if (nw > l.now_w) l.now_w = nw; l.now_h += 2 + normal_line(CAST_NOTE); }
    l.height = l.now_h > large_hours_h() ? l.now_h : large_hours_h();
    l.width = l.now_w + 28 + 1 + 28 + n * LARGE_COL;
    return l;
}

static void draw_large(const Weather *w, const WeatherHour *hours, int n, const LargeLayout *l, float right, float bottom) {
    int h24 = settings_clock_24h(&settings);
    float left = right - l->width, top = bottom - l->height;
    float now_top = top + (l->height - l->now_h) / 2, hours_top = top + (l->height - large_hours_h()) / 2;
    float cols = left + l->now_w + 28 + 1 + 28;
    char text[16];

    icon_at(w->kind, w->day, 50, left, now_top + 3);
    for (int i = 0; i < n; i++) icon_at(hours[i].kind, hours[i].day, 32, cols + i * LARGE_COL + (LARGE_COL - 32) / 2.0f, hours_top + 18 * 1.2f + 2 + 3);
    paint(WHITE, 1, SHADOWS(CAST_ICON_SHADOW), 0);

    text_at(CAST_BIG, l->big, left + 60, now_top + baseline_in(CAST_BIG, 56));
    float y = now_top + 56;
    if (l->cond) { text_at(CAST_TEMP, l->cond, left, y + 10 + baseline_in(CAST_TEMP, normal_line(CAST_TEMP))); y += 10 + normal_line(CAST_TEMP); }
    for (int i = 0; i < n; i++) {
        snprintf(text, sizeof text, "%d°", hours[i].temp);
        text_at(CAST_TEMP, text, cols + i * LARGE_COL + (LARGE_COL - text_w(CAST_TEMP, text)) / 2, hours_top + 18 * 1.2f + 2 + 3 + 32 + 3 + baseline_in(CAST_TEMP, 26 * 1.2f));
    }
    paint_text(WHITE, 1);
    if (l->meta[0]) { text_at(CAST_NOTE, l->meta, left, y + 2 + baseline_in(CAST_NOTE, normal_line(CAST_NOTE))); y += 2 + normal_line(CAST_NOTE); }
    paint_text(WHITE, 0.82f);
    if (l->note[0]) { text_at(CAST_NOTE, l->note, left, y + 2 + baseline_in(CAST_NOTE, normal_line(CAST_NOTE))); paint_text(RAIN_TEXT, 1); }

    for (int i = 0; i < n; i++) {
        hour_text(&hours[i], h24, text, sizeof text);
        text_at(CAST_TIME, text, cols + i * LARGE_COL + (LARGE_COL - text_w(CAST_TIME, text)) / 2, hours_top + baseline_in(CAST_TIME, 18 * 1.2f));
    }
    paint_text(WHITE, 0.6f);

    box_at((Box){left + l->now_w + 28, top, 1, l->height, 0, 0}, NULL);
    paint(WHITE, 0.28f, NULL, 0, 0);

    float bars_bottom = hours_top + large_hours_h() - 2;
    for (int wet = 0; wet < 2; wet++) {
        for (int i = 0; i < n; i++) {
            if ((hours[i].rain >= RAIN_LIKELY) != wet) continue;
            int bar_h = (int)lroundf(hours[i].rain * 0.26f);
            box_at((Box){cols + i * LARGE_COL + 4, bars_bottom - (bar_h < 3 ? 3 : bar_h), LARGE_COL - 8, (float)(bar_h < 3 ? 3 : bar_h), 5, 0}, NULL);
        }
        paint(RAIN_BLUE, wet ? 0.9f : 0.5f, NULL, 0, 0);
    }
    box_at((Box){cols, bars_bottom, n * LARGE_COL, 2, 0, 0}, NULL);
    paint(WHITE, 0.3f, NULL, 0, 0);
    for (int dry = 0; dry < 2; dry++) {
        for (int i = 0; i < n; i++) {
            if (hours[i].rain < 20 || (hours[i].rain < 30) != dry) continue;
            int bar_h = (int)lroundf(hours[i].rain * 0.26f);
            snprintf(text, sizeof text, "%d%%", hours[i].rain);
            float span_bottom = bars_bottom - (bar_h < 3 ? 3 : bar_h) - 3;
            text_at(CAST_TIME, text, cols + i * LARGE_COL + (LARGE_COL - text_w(CAST_TIME, text)) / 2, span_bottom - 18 + baseline_in(CAST_TIME, 18));
        }
        paint_text(RAIN_TEXT, dry ? 0.55f : 1);
    }
}

static void shade(void) {
    static const float at[] = {0, 0.25f, 0.45f, 0.65f, 0.82f, 1}, alpha[] = {0, 0.03f, 0.09f, 0.17f, 0.24f, 0.28f};
    for (int y = 0; y < band->h; y++) {
        float t = (y + 0.5f) / band->h, a = alpha[5];
        for (int i = 1; i < 6; i++)
            if (t <= at[i]) { a = alpha[i - 1] + (alpha[i] - alpha[i - 1]) * (t - at[i - 1]) / (at[i] - at[i - 1]); break; }
        Uint32 *row = (Uint32 *)((Uint8 *)band->pixels + (size_t)y * band->pitch), px = (Uint32)lroundf(a * 255) << 24;
        for (int x = 0; x < band->w; x++) row[x] = px;
    }
}

SDL_Surface *status_draw(int sw, int sh, const Weather *weather, time_t now) {
    if (fabsf(sh / 1080.0f - k) > 0.001f && !open_fonts(sh / 1080.0f)) return NULL;
    if (!fonts[CLOCK].main || !fonts[LINE].main) return NULL;
    int bh = STATUS_BAND_H * sh / 1080;
    band = SDL_CreateRGBSurfaceWithFormat(0, sw, bh, 32, SDL_PIXELFORMAT_ARGB8888);
    mask.a = band ? calloc((size_t)sw * bh, 1) : NULL;
    if (!mask.a) { if (band) SDL_FreeSurface(band); band = NULL; return NULL; }
    mask.w = sw; mask.h = bh;
    mask.x0 = sw; mask.y0 = bh; mask.x1 = 0; mask.y1 = 0;
    band_top = (float)(sh - bh);
    shade();

    float width = sw / k, left = 125, right = width - 125, bottom = 1080 - 72;
    int large = weather && settings.saver_forecast && settings.forecast_large;
    int show_line = weather && settings.show_weather && !large;
    WeatherHour hours[8];
    int n = weather && settings.saver_forecast ? hours_ahead(weather, now, large ? 7 : 6, hours) : 0;

    ClockLayout clock = clock_layout(now);
    char date[160], temp[16];
    date_text(localtime(&now), date, sizeof date);
    float date_w = text_w(LINE, date), date_line = 32 * 1.2f, weather_line = 32 * 1.25f;
    const char *cond = show_line && weather->label[0] ? tr(weather->label) : NULL;
    float weather_w = 0;
    if (show_line) {
        snprintf(temp, sizeof temp, "%d°", weather->temp);
        weather_w = 36 + 10 + text_w(LINE, temp) + (cond ? 12 + text_w(LINE, cond) : 0);
    }

    if (large) {
        LargeLayout l = large_layout(weather, hours, n);
        float row2 = 14 + date_line, row1 = clock.height, need = n ? l.height + 8 : 0;
        float extra = need > row1 + row2 ? (need - row1 - row2) / 2 : 0;
        row1 += extra;
        row2 += extra;
        float col1 = clock.w - 6 > 2 + date_w ? clock.w - 6 : 2 + date_w;
        draw_clock(&clock, left - 6 + (col1 + 6 - clock.w) / 2, bottom - row2);
        text_at(LINE, date, left + 2, bottom - date_line + baseline_in(LINE, date_line));
        paint_text(INK_DARK, 1);
        if (n) draw_large(weather, hours, n, &l, right, bottom - 8);
    } else {
        float row_w = date_w > weather_w ? date_w : weather_w;
        draw_clock(&clock, left - 6, bottom);
        float row_bottom = bottom - 12;
        text_at(LINE, date, right - date_w, row_bottom - date_line + baseline_in(LINE, date_line));
        float weather_top = row_bottom - date_line - 6 - weather_line;
        if (show_line) {
            float x = right - weather_w + 36 + 10;
            text_at(LINE, temp, x, weather_top + baseline_in(LINE, weather_line));
            if (cond) text_at(LINE, cond, x + text_w(LINE, temp) + 12, weather_top + baseline_in(LINE, weather_line));
        }
        paint_text(INK_DARK, 1);
        if (show_line) {
            icon_at(weather->kind, weather->day, 36, right - weather_w, weather_top + (weather_line - 36) / 2);
            paint(settings.dark ? INK_DARK : WHITE, 1, SHADOWS(STATUS_ICON_SHADOW), 0);
        }
        if (n) {
            float cast_w = small_width(n), free_w = (right - left) - (clock.w - 6) - (cast_w + 48) - row_w;
            draw_small(hours, n, left + (clock.w - 6) + free_w / 2 + 24, bottom - 12);
        }
    }
    free(mask.a);
    mask.a = NULL;
    SDL_Surface *out = band;
    band = NULL;
    return out;
}

static int sample_hours_ahead(const Weather *w, time_t now, int count, WeatherHour *out) {
    time_t there = now + w->offset;
    char hour[16];
    strftime(hour, sizeof hour, "%Y-%m-%dT%H", gmtime(&there));
    int n = 0;
    for (int i = 0; i < w->hour_count && n < count; i++)
        if (strncmp(w->hours[i].t, hour, 13) > 0) out[n++] = w->hours[i];
    return n;
}

static Weather sample_weather(time_t now) {
    static const int temps[] = {17, 18, 18, 16, 15, 14, 13, 12, 12, 11, 11, 10, 10, 9, 9, 9, 8, 9, 10, 12, 14, 15, 16, 17, 17, 16};
    static const int rains[] = {5, 10, 25, 55, 80, 60, 35, 20, 10, 0, 0, 0, 0, 0, 5, 5, 10, 10, 0, 0, 0, 0, 0, 0, 0, 0};
    static const WeatherKind kinds[] = {WK_PARTLY, WK_CLOUDY, WK_CLOUDY, WK_RAIN, WK_STORM, WK_RAIN, WK_CLOUDY, WK_PARTLY, WK_CLEAR, WK_CLEAR, WK_FOG, WK_FOG, WK_SNOW};
    Weather w = {.temp = 17, .kind = WK_PARTLY, .day = 1, .has_range = 1, .high = 19, .low = 8, .hour_count = 26};
    snprintf(w.label, sizeof w.label, "Partly cloudy");
    struct tm local = *localtime(&now);
    w.offset = (int)local.tm_gmtoff;
    w.day = local.tm_hour >= 7 && local.tm_hour < 17;
    time_t first = now - local.tm_min * 60 - local.tm_sec;
    for (int i = 0; i < 26; i++) {
        time_t at = first + i * 3600;
        struct tm t = *localtime(&at);
        WeatherHour *h = &w.hours[i];
        strftime(h->t, sizeof h->t, "%Y-%m-%dT%H:00", &t);
        h->temp = temps[i];
        h->rain = rains[i];
        h->kind = kinds[i % 13];
        h->day = t.tm_hour >= 7 && t.tm_hour < 17;
    }
    return w;
}

static void cover(SDL_Surface *photo, SDL_Surface *canvas) {
    float scale = fmaxf((float)canvas->w / photo->w, (float)canvas->h / photo->h);
    int w = (int)(canvas->w / scale), h = (int)(canvas->h / scale);
    SDL_Rect from = {(photo->w - w) / 2, (photo->h - h) / 2, w, h};
    SDL_BlitScaled(photo, &from, canvas, NULL);
}

int status_check(int argc, char **argv) {
    if (argc < 2) {
        fprintf(stderr, "usage: lumen-saver --status <app folder> <out.png> [--photo f.jpg] [--forecast off|small|large] [--line on|off]\n"
                        "  [--clock 12|24] [--ampm on|off] [--lang code] [--time 'YYYY-MM-DD HH:MM'] [--look light|dark] [--no-weather]\n");
        return 2;
    }
    Settings s;
    settings_load(argv[0], &s);
    const char *photo = NULL;
    int with_weather = 1;
    time_t now = time(NULL);
    for (int i = 2; i < argc; i++) {
        const char *o = argv[i];
        if (!strcmp(o, "--no-weather")) { with_weather = 0; continue; }
        if (i + 1 >= argc) { fprintf(stderr, "%s needs a value\n", o); return 2; }
        const char *v = argv[++i];
        if (!strcmp(o, "--photo")) photo = v;
        else if (!strcmp(o, "--forecast")) { s.saver_forecast = strcmp(v, "off") != 0; s.forecast_large = !strcmp(v, "large"); }
        else if (!strcmp(o, "--line")) s.show_weather = !strcmp(v, "on");
        else if (!strcmp(o, "--clock")) snprintf(s.clock_format, sizeof s.clock_format, "%sh", v);
        else if (!strcmp(o, "--ampm")) s.show_am_pm = !strcmp(v, "on");
        else if (!strcmp(o, "--lang")) snprintf(s.language, sizeof s.language, "%s", v);
        else if (!strcmp(o, "--look")) s.dark = strcmp(v, "light") != 0;
        else if (!strcmp(o, "--time")) {
            struct tm t = *localtime(&now);
            if (sscanf(v, "%d-%d-%d %d:%d", &t.tm_year, &t.tm_mon, &t.tm_mday, &t.tm_hour, &t.tm_min) == 5) { t.tm_year -= 1900; t.tm_mon -= 1; t.tm_sec = 0; t.tm_isdst = -1; now = mktime(&t); }
        } else { fprintf(stderr, "unknown option %s\n", o); return 2; }
    }
    if (!status_init(argv[0], &s)) { fprintf(stderr, "no font\n"); return 1; }
    printf("font %s, language %s, clock %s%s, forecast %s, line %s, %s\n", sf_clock ? "SF" : "LG Smart UI", s.language,
           settings_clock_24h(&s) ? "24h" : "12h", s.show_am_pm ? " AM/PM" : "", s.saver_forecast ? (s.forecast_large ? "large" : "small") : "off",
           s.show_weather ? "on" : "off", s.dark ? "dark" : "light");
    hours_ahead = sample_hours_ahead;
    Weather w = sample_weather(now);
    Uint64 t0 = SDL_GetPerformanceCounter();
    SDL_Surface *b = status_draw(1920, 1080, with_weather ? &w : NULL, now);
    Uint64 t1 = SDL_GetPerformanceCounter();
    if (b) SDL_FreeSurface(b);
    Uint64 t2 = SDL_GetPerformanceCounter();
    b = status_draw(1920, 1080, with_weather ? &w : NULL, now);
    Uint64 t3 = SDL_GetPerformanceCounter();
    double f = 1000.0 / SDL_GetPerformanceFrequency();
    printf("status_draw: %.1f ms first, %.1f ms again\n", (t1 - t0) * f, (t3 - t2) * f);
    if (!b) { fprintf(stderr, "draw failed\n"); return 1; }

    SDL_Surface *canvas = SDL_CreateRGBSurfaceWithFormat(0, 1920, 1080, 32, SDL_PIXELFORMAT_ARGB8888);
    SDL_FillRect(canvas, NULL, 0xFF808080);
    SDL_Surface *pic = photo ? IMG_Load(photo) : NULL;
    if (pic) { cover(pic, canvas); SDL_FreeSurface(pic); }
    else if (photo) fprintf(stderr, "photo: %s\n", IMG_GetError());
    for (int y = 0; y < b->h; y++) {
        Uint32 *dst = (Uint32 *)((Uint8 *)canvas->pixels + (size_t)(1080 - b->h + y) * canvas->pitch);
        const Uint32 *src = (const Uint32 *)((const Uint8 *)b->pixels + (size_t)y * b->pitch);
        for (int x = 0; x < 1920; x++) {
            Uint32 s4 = src[x], d = dst[x], keep = 255 - (s4 >> 24), out = 0xFF000000;
            for (int sh = 0; sh < 24; sh += 8) out |= (((s4 >> sh) & 255) + ((d >> sh) & 255) * keep / 255) << sh;
            dst[x] = out;
        }
    }
    int ok = IMG_SavePNG(canvas, argv[1]) == 0;
    if (!ok) fprintf(stderr, "save: %s\n", IMG_GetError());
    SDL_FreeSurface(canvas);
    SDL_FreeSurface(b);
    status_quit();
    return ok ? 0 : 1;
}
