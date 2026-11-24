#pragma once
#include <SDL.h>
#include "picker.h"

enum { PAIR_WIDTH = 956 };

SDL_Surface *compose_frame(const Picture *pic, int sw, int sh, int print, int night);
int compose_check(int argc, char **argv);
