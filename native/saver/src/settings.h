#pragma once

typedef struct {
    char language[16];
    char tv_locale[16];
    char clock_format[8];
    int tv_clock_24h;
    int show_am_pm;
    char wallpaper_dim[8];
    int reduce_motion;
    int dark;
    int show_weather;
    int saver_forecast;
    int forecast_large;
    char weather_city[64];
    int print;
    char brightness[8];
} Settings;

void settings_load(const char *app_dir, Settings *out);
int settings_clock_24h(const Settings *s);
int settings_night(const Settings *s);
