#pragma once
#include <SDL.h>
#include <time.h>
#include "settings.h"
#include "weather.h"

enum { STATUS_BAND_H = 520 };

int status_init(const char *app_dir, const Settings *s);
SDL_Surface *status_draw(int sw, int sh, const Weather *weather, time_t now);
void status_quit(void);
int status_check(int argc, char **argv);
