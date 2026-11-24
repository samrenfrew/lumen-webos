#pragma once
#include <SDL.h>

typedef struct {
    int count;
    char id[80];
    SDL_Surface *img[2];
    float x[2], y[2];
} Picture;

typedef struct {
    int immich;
    char sources[512];
    int recent_days;
    int seconds;
} SaverConfig;

void picker_init(const char *app_dir, SaverConfig *config);
int picker_next(Picture *out);
void picture_free(Picture *pic);
