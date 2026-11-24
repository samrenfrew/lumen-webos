#pragma once
#include <time.h>
#include "settings.h"

typedef enum { WK_CLEAR, WK_PARTLY, WK_CLOUDY, WK_FOG, WK_RAIN, WK_SNOW, WK_STORM } WeatherKind;

typedef struct {
    char t[17];
    int temp;
    WeatherKind kind;
    int day;
    int rain;
} WeatherHour;

typedef struct {
    int temp;
    char label[16];
    WeatherKind kind;
    int day;
    int has_range, high, low;
    int hour_count;
    WeatherHour hours[26];
    int offset;
} Weather;

int weather_init(const char *app_dir, const Settings *s);
int weather_peek(Weather *out);
int weather_read(Weather *out);
int weather_hours_ahead(const Weather *w, time_t now, int count, WeatherHour *out);
int weather_check(int argc, char **argv);
