#include "gfx.h"
#include "util.h"
#include <stdlib.h>
#include <string.h>

static const char *const VERTEX =
    "attribute vec2 a_pos;\n"
    "uniform vec4 u_quad;\n"
    "uniform float u_flip;\n"
    "varying vec2 v_px;\n"
    "varying vec2 v_uv;\n"
    "void main() {\n"
    "  v_px = u_quad.xy + a_pos * u_quad.zw;\n"
    "  v_uv = v_px / vec2(1920.0, 1080.0);\n"
    "  vec2 c = v_uv * 2.0 - 1.0;\n"
    "  gl_Position = vec4(c.x, c.y * u_flip, 0.0, 1.0);\n"
    "}\n";

static const char *const FRAGMENT_HEAD =
    "#ifdef GL_FRAGMENT_PRECISION_HIGH\n"
    "precision highp float;\n"
    "#else\n"
    "precision mediump float;\n"
    "#endif\n"
    "varying vec2 v_px;\n"
    "varying vec2 v_uv;\n"
    "float rr(vec2 p, vec2 half_size, float r) {\n"
    "  vec2 q = abs(p) - half_size + r;\n"
    "  return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;\n"
    "}\n"
    "vec2 rr_normal(vec2 p, vec2 half_size, float r) {\n"
    "  vec2 q = abs(p) - half_size + r;\n"
    "  vec2 n = (q.x > 0.0 && q.y > 0.0) ? normalize(q) : (q.x > q.y ? vec2(1.0, 0.0) : vec2(0.0, 1.0));\n"
    "  return n * vec2(p.x < 0.0 ? -1.0 : 1.0, p.y < 0.0 ? -1.0 : 1.0);\n"
    "}\n";

static int screen_w, screen_h;
static float flip = -1;
static GLuint quad_buffer;

void gfx_init(int w, int h) {
    screen_w = w;
    screen_h = h;
    static const GLfloat corners[] = {0, 0, 1, 0, 0, 1, 1, 1};
    glGenBuffers(1, &quad_buffer);
    glBindBuffer(GL_ARRAY_BUFFER, quad_buffer);
    glBufferData(GL_ARRAY_BUFFER, sizeof corners, corners, GL_STATIC_DRAW);
    glEnableVertexAttribArray(0);
    glVertexAttribPointer(0, 2, GL_FLOAT, GL_FALSE, 0, NULL);
    glDisable(GL_DEPTH_TEST);
    glClearColor(0, 0, 0, 1);
    glPixelStorei(GL_UNPACK_ALIGNMENT, 1);
    glPixelStorei(GL_PACK_ALIGNMENT, 1);
}

static GLuint compile(GLenum kind, const char *name, const char **parts, int count) {
    GLuint shader = glCreateShader(kind);
    glShaderSource(shader, count, parts, NULL);
    glCompileShader(shader);
    GLint ok = 0;
    glGetShaderiv(shader, GL_COMPILE_STATUS, &ok);
    if (!ok) {
        char why[1024] = "";
        glGetShaderInfoLog(shader, sizeof why, NULL, why);
        note("shader %s: %s", name, why);
    }
    return shader;
}

GLuint gfx_program(const char *name, const char *fragment) {
    const char *vertex_parts[] = {VERTEX};
    const char *fragment_parts[] = {FRAGMENT_HEAD, fragment};
    GLuint program = glCreateProgram();
    glAttachShader(program, compile(GL_VERTEX_SHADER, name, vertex_parts, 1));
    glAttachShader(program, compile(GL_FRAGMENT_SHADER, name, fragment_parts, 2));
    glBindAttribLocation(program, 0, "a_pos");
    glLinkProgram(program);
    GLint ok = 0;
    glGetProgramiv(program, GL_LINK_STATUS, &ok);
    if (!ok) {
        char why[1024] = "";
        glGetProgramInfoLog(program, sizeof why, NULL, why);
        note("program %s: %s", name, why);
    }
    return program;
}

GLint gfx_loc(GLuint program, const char *name) {
    static struct { GLuint program; const char *name; GLint at; } known[160];
    static int count;
    for (int i = 0; i < count; i++) if (known[i].program == program && known[i].name == name) return known[i].at;
    GLint at = glGetUniformLocation(program, name);
    if (count < (int)(sizeof known / sizeof *known)) {
        known[count].program = program;
        known[count].name = name;
        known[count++].at = at;
    }
    return at;
}

static GLuint new_texture(int w, int h, const void *pixels) {
    GLuint tex;
    glGenTextures(1, &tex);
    glBindTexture(GL_TEXTURE_2D, tex);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
    glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA, w, h, 0, GL_RGBA, GL_UNSIGNED_BYTE, pixels);
    return tex;
}

Tex gfx_upload(SDL_Surface *surface, int premultiply) {
    Tex t = {0, 0, 0};
    SDL_Surface *rgba = SDL_ConvertSurfaceFormat(surface, SDL_PIXELFORMAT_RGBA32, 0);
    if (!rgba) return t;
    Uint8 *tight = malloc((size_t)rgba->w * rgba->h * 4);
    if (tight) {
        for (int y = 0; y < rgba->h; y++) {
            Uint8 *row = tight + (size_t)y * rgba->w * 4;
            memcpy(row, (Uint8 *)rgba->pixels + (size_t)y * rgba->pitch, (size_t)rgba->w * 4);
            if (premultiply)
                for (int x = 0; x < rgba->w; x++) {
                    Uint8 *px = row + x * 4;
                    px[0] = (Uint8)(px[0] * px[3] / 255);
                    px[1] = (Uint8)(px[1] * px[3] / 255);
                    px[2] = (Uint8)(px[2] * px[3] / 255);
                }
        }
        t.tex = new_texture(rgba->w, rgba->h, tight);
        t.w = rgba->w;
        t.h = rgba->h;
        free(tight);
    }
    SDL_FreeSurface(rgba);
    return t;
}

void gfx_free(Tex *t) {
    if (t->tex) glDeleteTextures(1, &t->tex);
    t->tex = 0;
}

Target gfx_target(int w, int h) {
    Target target = {0, {new_texture(w, h, NULL), w, h}};
    glGenFramebuffers(1, &target.fbo);
    glBindFramebuffer(GL_FRAMEBUFFER, target.fbo);
    glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, target.t.tex, 0);
    if (glCheckFramebufferStatus(GL_FRAMEBUFFER) != GL_FRAMEBUFFER_COMPLETE) note("target %dx%d: not complete", w, h);
    return target;
}

void gfx_target_free(Target *target) {
    if (target->fbo) glDeleteFramebuffers(1, &target->fbo);
    target->fbo = 0;
    gfx_free(&target->t);
}

void gfx_into(const Target *target) {
    glBindFramebuffer(GL_FRAMEBUFFER, target ? target->fbo : 0);
    if (target) glViewport(0, 0, target->t.w, target->t.h);
    else glViewport(0, 0, screen_w, screen_h);
    flip = target ? 1 : -1;
}

void gfx_into_plate(const Target *target) {
    glBindFramebuffer(GL_FRAMEBUFFER, target->fbo);
    glViewport(0, 0, DESIGN_W, DESIGN_H);
    flip = 1;
}

void gfx_bind(GLuint program, int unit, const char *name, GLuint tex) {
    glUseProgram(program);
    glActiveTexture(GL_TEXTURE0 + (GLenum)unit);
    glBindTexture(GL_TEXTURE_2D, tex);
    glUniform1i(gfx_loc(program, name), unit);
}

void gfx_quad(GLuint program, float x, float y, float w, float h) {
    glUseProgram(program);
    glUniform4f(gfx_loc(program, "u_quad"), x, y, w, h);
    glUniform1f(gfx_loc(program, "u_flip"), flip);
    glDrawArrays(GL_TRIANGLE_STRIP, 0, 4);
}

SDL_Surface *gfx_read_screen(void) {
    SDL_Surface *shot = SDL_CreateRGBSurfaceWithFormat(0, screen_w, screen_h, 32, SDL_PIXELFORMAT_RGBA32);
    Uint8 *pixels = malloc((size_t)screen_w * screen_h * 4);
    if (!shot || !pixels) return NULL;
    glReadPixels(0, 0, screen_w, screen_h, GL_RGBA, GL_UNSIGNED_BYTE, pixels);
    for (int y = 0; y < screen_h; y++)
        memcpy((Uint8 *)shot->pixels + (size_t)y * shot->pitch, pixels + (size_t)(screen_h - 1 - y) * screen_w * 4, (size_t)screen_w * 4);
    free(pixels);
    return shot;
}
