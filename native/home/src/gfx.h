#pragma once
#include <SDL.h>
#include <GLES2/gl2.h>

enum { DESIGN_W = 1920, DESIGN_H = 1080 };

typedef struct { GLuint tex; int w, h; } Tex;
typedef struct { GLuint fbo; Tex t; } Target;

void gfx_init(int screen_w, int screen_h);
GLuint gfx_program(const char *name, const char *fragment);
GLint gfx_loc(GLuint program, const char *name);
Tex gfx_upload(SDL_Surface *surface, int premultiply);
void gfx_free(Tex *t);
Target gfx_target(int w, int h);
void gfx_target_free(Target *t);
void gfx_into(const Target *target);
void gfx_into_plate(const Target *target);
void gfx_bind(GLuint program, int unit, const char *name, GLuint tex);
void gfx_quad(GLuint program, float x, float y, float w, float h);
SDL_Surface *gfx_read_screen(void);
