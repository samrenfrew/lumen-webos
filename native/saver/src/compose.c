#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <SDL_image.h>
#include "compose.h"
#include "util.h"

enum { BLUR_BAND_AT_1080 = 520, NIGHT_KEEP = 141, ONE = 65536, TAP_ONE = 256 };

typedef struct {
    int *start, *count, *weight;
    int stride, lo, hi;
} Axis;

static void axis_free(Axis *a) {
    free(a->start);
    free(a->count);
    free(a->weight);
}

static int axis_build(Axis *a, int source, int scaled, int offset, int c0, int c1) {
    int n = c1 - c0;
    double s = (double)source / scaled;
    a->stride = s > 1 ? (int)ceil(s) + 2 : 2;
    a->start = malloc(n * sizeof(int));
    a->count = malloc(n * sizeof(int));
    a->weight = calloc((size_t)n * a->stride, sizeof(int));
    if (!a->start || !a->count || !a->weight) return 0;
    double *exact = malloc(a->stride * sizeof(double));
    if (!exact) return 0;
    a->lo = source;
    a->hi = -1;
    for (int i = 0; i < n; i++) {
        int *w = a->weight + i * a->stride;
        double pos = c0 + i - offset;
        int first, taps;
        if (s > 1) {
            double from = pos * s, to = from + s;
            if (to > source) to = source;
            first = (int)floor(from);
            int last = (int)ceil(to) - 1;
            if (last >= source) last = source - 1;
            taps = last - first + 1;
            for (int k = 0; k < taps; k++) {
                double lo = first + k > from ? first + k : from, hi = first + k + 1 < to ? first + k + 1 : to;
                exact[k] = (hi - lo) / (to - from);
            }
        } else {
            double centre = (pos + 0.5) * s - 0.5;
            int base = (int)floor(centre);
            double f = centre - base;
            if (base < 0) { first = 0; taps = 1; exact[0] = 1; }
            else if (base >= source - 1) { first = source - 1; taps = 1; exact[0] = 1; }
            else { first = base; taps = 2; exact[0] = 1 - f; exact[1] = f; }
        }
        int sum = 0, biggest = 0;
        for (int k = 0; k < taps; k++) {
            w[k] = (int)(exact[k] * TAP_ONE + 0.5);
            sum += w[k];
            if (w[k] > w[biggest]) biggest = k;
        }
        w[biggest] += TAP_ONE - sum;
        a->start[i] = first;
        a->count[i] = taps;
        if (first < a->lo) a->lo = first;
        if (first + taps - 1 > a->hi) a->hi = first + taps - 1;
    }
    free(exact);
    return 1;
}

static const uint32_t *argb_row(const SDL_Surface *src, int y, uint32_t *buf) {
    const uint8_t *line = (const uint8_t *)src->pixels + (size_t)y * src->pitch;
    if (src->format->format == SDL_PIXELFORMAT_ARGB8888) return (const uint32_t *)line;
    if (src->format->format == SDL_PIXELFORMAT_RGB24) {
        for (int x = 0; x < src->w; x++) buf[x] = 0xFF000000u | (uint32_t)line[x * 3] << 16 | (uint32_t)line[x * 3 + 1] << 8 | line[x * 3 + 2];
    } else {
        SDL_ConvertPixels(src->w, 1, src->format->format, line, src->pitch, SDL_PIXELFORMAT_ARGB8888, buf, src->w * 4);
    }
    return buf;
}

static void resample_into(SDL_Surface *dst, const SDL_Surface *src, int dw, int dh, int dx, int dy, SDL_Rect box) {
    int c0 = dx > box.x ? dx : box.x, c1 = dx + dw < box.x + box.w ? dx + dw : box.x + box.w;
    int r0 = dy > box.y ? dy : box.y, r1 = dy + dh < box.y + box.h ? dy + dh : box.y + box.h;
    if (c1 <= c0 || r1 <= r0) return;
    int cw = c1 - c0, ch = r1 - r0;
    Axis ax = {0}, ay = {0};
    uint32_t *mid = NULL, *converted = malloc((size_t)src->w * 4);
    if (!axis_build(&ax, src->w, dw, dx, c0, c1) || !axis_build(&ay, src->h, dh, dy, r0, r1)) goto done;
    int rows = ay.hi - ay.lo + 1;
    mid = malloc((size_t)rows * cw * sizeof(uint32_t));
    if (!mid || !converted) goto done;
    for (int r = 0; r < rows; r++) {
        const uint32_t *in = argb_row(src, ay.lo + r, converted);
        uint32_t *out = mid + (size_t)r * cw;
        for (int x = 0; x < cw; x++) {
            const uint32_t *p = in + ax.start[x];
            const int *w = ax.weight + x * ax.stride;
            uint32_t rb = 0, g = 0;
            for (int k = 0; k < ax.count[x]; k++) {
                rb += (p[k] & 0x00FF00FFu) * w[k];
                g += (p[k] >> 8 & 255) * w[k];
            }
            out[x] = ((rb + 0x00800080u) >> 8 & 0x00FF00FFu) | ((g + 128) >> 8) << 8;
        }
    }
    for (int y = 0; y < ch; y++) {
        uint32_t *out = (uint32_t *)((uint8_t *)dst->pixels + (size_t)(r0 + y) * dst->pitch) + c0;
        const int *w = ay.weight + y * ay.stride;
        const uint32_t *first = mid + (size_t)(ay.start[y] - ay.lo) * cw;
        int taps = ay.count[y];
        for (int x = 0; x < cw; x++) {
            uint32_t rb = 0, g = 0;
            for (int k = 0; k < taps; k++) {
                uint32_t v = first[(size_t)k * cw + x];
                rb += (v & 0x00FF00FFu) * w[k];
                g += (v >> 8 & 255) * w[k];
            }
            out[x] = 0xFF000000u | ((rb + 0x00800080u) >> 8 & 0x00FF00FFu) | ((g + 128) >> 8) << 8;
        }
    }
done:
    free(converted);
    free(mid);
    axis_free(&ax);
    axis_free(&ay);
}

static uint32_t box_mean(uint32_t rb, uint32_t g, uint32_t recip) {
    return (((rb >> 16) * recip + 32768) >> 16) << 16 | ((g * recip + 32768) >> 16) << 8 | (((rb & 0xFFFF) * recip + 32768) >> 16);
}

static void box_blur_row(const uint32_t *in, uint32_t *out, int n, int r) {
    uint32_t recip = (ONE + r) / (2 * r + 1), rb = 0, g = 0;
    for (int k = -r; k <= r; k++) {
        uint32_t v = in[k < 0 ? 0 : k >= n ? n - 1 : k];
        rb += v & 0x00FF00FFu;
        g += v >> 8 & 255;
    }
    for (int i = 0; i < n; i++) {
        out[i] = box_mean(rb, g, recip);
        uint32_t add = in[i + r + 1 >= n ? n - 1 : i + r + 1], sub = in[i - r < 0 ? 0 : i - r];
        rb = rb + (add & 0x00FF00FFu) - (sub & 0x00FF00FFu);
        g = g + (add >> 8 & 255) - (sub >> 8 & 255);
    }
}

static void box_blur_columns(const uint32_t *in, uint32_t *out, int w, int h, int r, uint32_t *sums) {
    uint32_t recip = (ONE + r) / (2 * r + 1);
    memset(sums, 0, (size_t)w * 2 * sizeof(uint32_t));
    for (int k = -r; k <= r; k++) {
        const uint32_t *row = in + (size_t)(k < 0 ? 0 : k >= h ? h - 1 : k) * w;
        for (int x = 0; x < w; x++) {
            sums[x * 2] += row[x] & 0x00FF00FFu;
            sums[x * 2 + 1] += row[x] >> 8 & 255;
        }
    }
    for (int y = 0; y < h; y++) {
        uint32_t *dst = out + (size_t)y * w;
        const uint32_t *add = in + (size_t)(y + r + 1 >= h ? h - 1 : y + r + 1) * w;
        const uint32_t *sub = in + (size_t)(y - r < 0 ? 0 : y - r) * w;
        for (int x = 0; x < w; x++) {
            uint32_t *s = sums + x * 2;
            dst[x] = box_mean(s[0], s[1], recip);
            s[0] = s[0] + (add[x] & 0x00FF00FFu) - (sub[x] & 0x00FF00FFu);
            s[1] = s[1] + (add[x] >> 8 & 255) - (sub[x] >> 8 & 255);
        }
    }
}

static double blur_mask(double t) {
    if (t < 0.4) return t / 0.4 * 0.6;
    if (t < 0.7) return 0.6 + (t - 0.4) / 0.3 * 0.4;
    return 1;
}

static int blur_bottom(SDL_Surface *frame) {
    static const int radius = 3;
    int w = frame->w, band = BLUR_BAND_AT_1080 * frame->h / 1080, y0 = frame->h - band;
    size_t bytes = (size_t)w * band * sizeof(uint32_t);
    uint32_t *a = calloc(1, bytes), *b = malloc(bytes), *sums = malloc((size_t)w * 2 * sizeof(uint32_t));
    int ok = a && b && sums;
    if (ok) {
        for (int y = 0; y < band; y++) {
            uint32_t *row = a + (size_t)y * w, *spare = b + (size_t)y * w;
            memcpy(row, (uint8_t *)frame->pixels + (size_t)(y0 + y) * frame->pitch, (size_t)w * 4);
            box_blur_row(row, spare, w, radius);
            box_blur_row(spare, row, w, radius);
        }
        box_blur_columns(a, b, w, band, radius, sums);
        box_blur_columns(b, a, w, band, radius, sums);
        for (int y = 0; y < band; y++) {
            int m = (int)(blur_mask((y + 0.5) / band) * 256 + 0.5);
            uint32_t *sharp = (uint32_t *)((uint8_t *)frame->pixels + (size_t)(y0 + y) * frame->pitch);
            const uint32_t *soft = a + (size_t)y * w;
            for (int x = 0; x < w; x++) {
                int sr = sharp[x] >> 16 & 255, sg = sharp[x] >> 8 & 255, sb = sharp[x] & 255;
                int r = sr + (((int)(soft[x] >> 16 & 255) - sr) * m >> 8);
                int g = sg + (((int)(soft[x] >> 8 & 255) - sg) * m >> 8);
                int bl = sb + (((int)(soft[x] & 255) - sb) * m >> 8);
                sharp[x] = 0xFF000000u | (uint32_t)r << 16 | (uint32_t)g << 8 | (uint32_t)bl;
            }
        }
    }
    free(a);
    free(b);
    free(sums);
    return ok;
}

static void dim_for_night(SDL_Surface *frame) {
    for (int y = 0; y < frame->h; y++) {
        uint32_t *row = (uint32_t *)((uint8_t *)frame->pixels + (size_t)y * frame->pitch);
        for (int x = 0; x < frame->w; x++) {
            uint32_t v = row[x];
            row[x] = 0xFF000000u | ((v >> 16 & 255) * NIGHT_KEEP >> 8) << 16 | ((v >> 8 & 255) * NIGHT_KEEP >> 8) << 8 | ((v & 255) * NIGHT_KEEP >> 8);
        }
    }
}

static void place_one(SDL_Surface *frame, SDL_Surface *img, SDL_Rect box, int fit_width, float px, float py) {
    float across = (float)box.w / img->w, down = (float)box.h / img->h;
    float scale = fit_width ? across : across > down ? across : down;
    int dw = (int)(img->w * scale + 0.5f), dh = (int)(img->h * scale + 0.5f);
    resample_into(frame, img, dw, dh, box.x + (int)((box.w - dw) * px), box.y + (int)((box.h - dh) * py), box);
}

SDL_Surface *compose_frame(const Picture *pic, int sw, int sh, int night) {
    if (!pic || pic->count < 1 || pic->count > 2 || !pic->img[0] || (pic->count == 2 && !pic->img[1]) || sw <= 0 || sh <= 0) return NULL;
    SDL_Surface *frame = SDL_CreateRGBSurfaceWithFormat(0, sw, sh, 32, SDL_PIXELFORMAT_ARGB8888);
    if (!frame) return NULL;
    SDL_FillRect(frame, NULL, 0xFF000000u);
    if (pic->count == 1) {
        place_one(frame, pic->img[0], (SDL_Rect){0, 0, sw, sh}, 0, pic->x[0], pic->y[0]);
    } else {
        int w = PAIR_WIDTH * sw / 1920;
        place_one(frame, pic->img[0], (SDL_Rect){0, 0, w, sh}, 1, 0, pic->y[0]);
        place_one(frame, pic->img[1], (SDL_Rect){sw - w, 0, w, sh}, 1, 0, pic->y[1]);
    }
    if (!blur_bottom(frame)) {
        SDL_FreeSurface(frame);
        return NULL;
    }
    if (night) dim_for_night(frame);
    return frame;
}

int compose_check(int argc, char **argv) {
    int extra = argc - 2;
    if (extra != 1 && extra != 3 && extra != 4) {
        fprintf(stderr, "usage: --compose <out.png> <night 0|1> <photo.jpg> [x y] | <photo.jpg> <y1> <photo2.jpg> <y2>\n");
        return 2;
    }
    Picture pic = {.count = extra == 4 ? 2 : 1};
    char **in = argv + 2;
    pic.img[0] = IMG_Load(in[0]);
    pic.x[0] = extra == 3 ? (float)atof(in[1]) : 0.5f;
    pic.y[0] = extra == 3 ? (float)atof(in[2]) : extra == 4 ? (float)atof(in[1]) : 0.5f;
    if (extra == 4) {
        pic.img[1] = IMG_Load(in[2]);
        pic.y[1] = (float)atof(in[3]);
    }
    if (!pic.img[0] || (extra == 4 && !pic.img[1])) {
        fprintf(stderr, "cannot load: %s\n", IMG_GetError());
        picture_free(&pic);
        return 1;
    }
    Uint32 t0 = SDL_GetTicks();
    SDL_Surface *frame = compose_frame(&pic, 1920, 1080, atoi(argv[1]));
    Uint32 ms = SDL_GetTicks() - t0;
    int rc = 1;
    if (frame) {
        printf("composed in %u ms, peak %ld kB\n", ms, status_kb("VmHWM"));
        rc = IMG_SavePNG(frame, argv[0]) == 0 ? 0 : 1;
        SDL_FreeSurface(frame);
    } else {
        fprintf(stderr, "compose failed\n");
    }
    picture_free(&pic);
    return rc;
}
