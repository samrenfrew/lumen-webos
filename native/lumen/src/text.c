#include <ft2build.h>
#include FT_FREETYPE_H
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

typedef struct hb_buffer_t hb_buffer_t;
typedef struct hb_font_t hb_font_t;
typedef struct { uint32_t codepoint, mask, cluster, var1, var2; } hb_glyph_info_t;
typedef struct { int32_t x_advance, y_advance, x_offset, y_offset; uint32_t var; } hb_glyph_position_t;
hb_buffer_t *hb_buffer_create(void);
void hb_buffer_destroy(hb_buffer_t *buffer);
void hb_buffer_add_utf8(hb_buffer_t *buffer, const char *text, int text_length, unsigned int item_offset, int item_length);
void hb_buffer_guess_segment_properties(hb_buffer_t *buffer);
void hb_shape(hb_font_t *font, hb_buffer_t *buffer, const void *features, unsigned int num_features);
hb_glyph_info_t *hb_buffer_get_glyph_infos(hb_buffer_t *buffer, unsigned int *length);
hb_glyph_position_t *hb_buffer_get_glyph_positions(hb_buffer_t *buffer, unsigned int *length);
hb_font_t *hb_ft_font_create_referenced(FT_Face face);
const char *hb_version_string(void);
typedef struct { uint32_t tag, value; unsigned int start, end; } hb_feature_t;
int hb_feature_from_string(const char *str, int len, hb_feature_t *feature);

enum { FACES = 16 };
typedef struct { char path[256]; int px; FT_Face face; hb_font_t *font; } Face;

static FT_Library library;
static Face faces[FACES];
static int face_count;

static Face *face_for(const char *path, int px) {
    for (int i = 0; i < face_count; i++) if (faces[i].px == px && !strcmp(faces[i].path, path)) return &faces[i];
    if (face_count == FACES || (!library && FT_Init_FreeType(&library))) return NULL;
    Face *f = &faces[face_count];
    if (FT_New_Face(library, path, 0, &f->face)) return NULL;
    FT_Set_Pixel_Sizes(f->face, 0, (FT_UInt)px);
    f->font = hb_ft_font_create_referenced(f->face);
    f->px = px;
    strncpy(f->path, path, sizeof f->path - 1);
    face_count++;
    return f;
}

static int run_end(const char *text, int from, int *other_script) {
    int i = from, first = -1;
    while (text[i]) {
        unsigned char c = (unsigned char)text[i];
        int len = c < 0x80 ? 1 : c < 0xE0 ? 2 : c < 0xF0 ? 3 : 4;
        int neutral = c == ' ' || (c < 0x80 && !((c | 32) >= 'a' && (c | 32) <= 'z'));
        int other = len == 3 && c == 0xE0 && ((unsigned char)text[i + 1] == 0xA4 || (unsigned char)text[i + 1] == 0xA5);
        if (!neutral) {
            if (first < 0) first = other;
            else if (other != first) break;
        }
        i += len;
    }
    *other_script = first > 0;
    return i;
}

typedef struct { Face *face; hb_buffer_t *buffer; } Run;

const char *lumen_text_shaper(void) { return hb_version_string(); }

int lumen_text_line(const char *font_path, const char *other_script_font_path, int px, float letter_spacing, const char *feature, const char *utf8, unsigned char **rgba, int *w, int *h) {
    enum { RUNS = 16 };
    Run runs[RUNS];
    hb_feature_t wanted;
    unsigned int wanted_count = feature && feature[0] && hb_feature_from_string(feature, -1, &wanted) ? 1 : 0;
    int count = 0;
    float width = 0;
    for (int at = 0; utf8[at] && count < RUNS;) {
        int other, end = run_end(utf8, at, &other);
        Face *face = face_for(other ? other_script_font_path : font_path, px);
        if (!face) return 0;
        hb_buffer_t *buffer = hb_buffer_create();
        hb_buffer_add_utf8(buffer, utf8, -1, (unsigned)at, end - at);
        hb_buffer_guess_segment_properties(buffer);
        hb_shape(face->font, buffer, wanted_count ? &wanted : NULL, wanted_count);
        unsigned int n;
        hb_glyph_position_t *pos = hb_buffer_get_glyph_positions(buffer, &n);
        for (unsigned int i = 0; i < n; i++) width += (float)pos[i].x_advance / 64 + letter_spacing;
        runs[count].face = face;
        runs[count++].buffer = buffer;
        at = end;
    }
    if (!count) return 0;
    int ascent = px, descent = px / 2 + 2, pad = 2;
    *w = (int)width + 2 * pad + 2;
    *h = ascent + descent;
    unsigned char *out = calloc((size_t)*w * *h, 4);
    if (!out) return 0;
    float pen = (float)pad;
    for (int r = 0; r < count; r++) {
        unsigned int n;
        hb_glyph_info_t *info = hb_buffer_get_glyph_infos(runs[r].buffer, &n);
        hb_glyph_position_t *pos = hb_buffer_get_glyph_positions(runs[r].buffer, &n);
        FT_Face face = runs[r].face->face;
        for (unsigned int i = 0; i < n; i++) {
            if (!FT_Load_Glyph(face, info[i].codepoint, FT_LOAD_RENDER | FT_LOAD_NO_HINTING)) {
                FT_Bitmap *bitmap = &face->glyph->bitmap;
                int left = (int)(pen + (float)pos[i].x_offset / 64 + 0.5f) + face->glyph->bitmap_left;
                int top = ascent - (int)((float)pos[i].y_offset / 64) - face->glyph->bitmap_top;
                for (unsigned int y = 0; y < bitmap->rows; y++)
                    for (unsigned int x = 0; x < bitmap->width; x++) {
                        int ox = left + (int)x, oy = top + (int)y;
                        if (ox < 0 || oy < 0 || ox >= *w || oy >= *h) continue;
                        unsigned char cover = bitmap->buffer[y * (unsigned int)bitmap->pitch + x], *px4 = out + ((size_t)oy * *w + ox) * 4;
                        if (cover > px4[3]) memset(px4, cover, 4);
                    }
            }
            pen += (float)pos[i].x_advance / 64 + letter_spacing;
        }
        hb_buffer_destroy(runs[r].buffer);
    }
    *rgba = out;
    return 1;
}

void lumen_text_free(unsigned char *rgba) { free(rgba); }
