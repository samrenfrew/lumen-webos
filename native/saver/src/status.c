#include "status.h"

int status_init(const char *app_dir, const Settings *s) { (void)app_dir; (void)s; return 0; }
SDL_Surface *status_draw(int sw, int sh, const Weather *weather, time_t now) { (void)sw; (void)sh; (void)weather; (void)now; return NULL; }
void status_quit(void) {}
int status_check(int argc, char **argv) { (void)argc; (void)argv; return 1; }
