#include "weather.h"

int weather_init(const char *app_dir, const Settings *s) { (void)app_dir; (void)s; return 0; }
int weather_peek(Weather *out) { (void)out; return 0; }
int weather_read(Weather *out) { (void)out; return 0; }
int weather_hours_ahead(const Weather *w, time_t now, int count, WeatherHour *out) { (void)w; (void)now; (void)count; (void)out; return 0; }
int weather_check(int argc, char **argv) { (void)argc; (void)argv; return 1; }
