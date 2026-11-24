#include "settings.h"
#include "util.h"
#include "../vendor/cJSON.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

static cJSON *read_json(const char *dir, const char *name) {
    char path[320];
    snprintf(path, sizeof path, "%s/config/%s", dir, name);
    char *text = read_file(path, NULL);
    if (!text) return NULL;
    cJSON *j = cJSON_Parse(text);
    free(text);
    return j;
}

static void copy_string(const cJSON *obj, const char *key, char *out, size_t n) {
    const cJSON *v = cJSON_GetObjectItemCaseSensitive(obj, key);
    if (cJSON_IsString(v) && strlen(v->valuestring) < n) snprintf(out, n, "%s", v->valuestring);
}

static int flag(const cJSON *obj, const char *key, int fallback) {
    const cJSON *v = cJSON_GetObjectItemCaseSensitive(obj, key);
    return cJSON_IsBool(v) ? cJSON_IsTrue(v) : fallback;
}

static int local_hour(void) {
    time_t now = time(NULL);
    return localtime(&now)->tm_hour;
}

void settings_load(const char *app_dir, Settings *s) {
    memset(s, 0, sizeof *s);
    snprintf(s->language, sizeof s->language, "auto");
    snprintf(s->tv_locale, sizeof s->tv_locale, "en-GB");
    snprintf(s->clock_format, sizeof s->clock_format, "auto");
    snprintf(s->wallpaper_dim, sizeof s->wallpaper_dim, "off");
    snprintf(s->brightness, sizeof s->brightness, "dim");
    s->tv_clock_24h = 1;
    s->show_am_pm = 1;
    s->show_weather = 1;
    char dark_mode[8] = "on", size[8] = "small";

    cJSON *file = read_json(app_dir, "lumen-config.json");
    const cJSON *c = cJSON_GetObjectItemCaseSensitive(file, "config");
    if (c) {
        copy_string(c, "language", s->language, sizeof s->language);
        copy_string(c, "clockFormat", s->clock_format, sizeof s->clock_format);
        copy_string(c, "wallpaperDim", s->wallpaper_dim, sizeof s->wallpaper_dim);
        copy_string(c, "ambientBrightness", s->brightness, sizeof s->brightness);
        copy_string(c, "weatherCity", s->weather_city, sizeof s->weather_city);
        copy_string(c, "darkMode", dark_mode, sizeof dark_mode);
        copy_string(c, "saverForecastSize", size, sizeof size);
        s->show_am_pm = flag(c, "showAmPm", 1);
        s->show_weather = flag(c, "showWeather", 1);
        s->reduce_motion = flag(c, "reduceMotion", 0);
        s->saver_forecast = flag(c, "saverForecast", 0);
    }
    cJSON_Delete(file);
    s->forecast_large = s->saver_forecast && !strcmp(size, "large");
    int h = local_hour();
    s->dark = !strcmp(dark_mode, "off") ? 0 : !strcmp(dark_mode, "auto") ? (h >= 19 || h < 6) : 1;

    cJSON *tv = read_json(app_dir, "tv.json");
    if (tv) {
        copy_string(tv, "locale", s->tv_locale, sizeof s->tv_locale);
        char clock[8] = "24h";
        copy_string(tv, "clock", clock, sizeof clock);
        s->tv_clock_24h = strcmp(clock, "12h") != 0;
    }
    cJSON_Delete(tv);
    note("settings: language %s (tv %s), clock %s, dim %s, brightness %s, weather %s in '%s', forecast %s, reduce motion %d",
         s->language, s->tv_locale, settings_clock_24h(s) ? "24h" : "12h", s->wallpaper_dim, s->brightness, s->show_weather ? "line" : "no line",
         s->weather_city, s->saver_forecast ? (s->forecast_large ? "large" : "small") : "off", s->reduce_motion);
}

int settings_clock_24h(const Settings *s) {
    if (!strcmp(s->clock_format, "24h")) return 1;
    if (!strcmp(s->clock_format, "12h")) return 0;
    return s->tv_clock_24h;
}

int settings_night(const Settings *s) {
    if (!strcmp(s->wallpaper_dim, "always")) return 1;
    int h = local_hour();
    return !strcmp(s->wallpaper_dim, "night") && (h >= 19 || h < 6);
}
