#include "weather.h"
#include "http.h"
#include "util.h"
#include "../vendor/cJSON.h"
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

enum { MAX_AGE_S = 30 * 60, KEPT_HOURS = 26, HOUR_PREFIX = 13 };

static const char *const KIND_NAMES[] = {"clear", "partly", "cloudy", "fog", "rain", "snow", "storm"};

static int auto_place;
static double place_lat, place_lon;
static char kept_path[320];

static WeatherKind kind_of(int code) {
    if (code == 0) return WK_CLEAR;
    if (code <= 2) return WK_PARTLY;
    if (code == 45 || code == 48) return WK_FOG;
    if ((code >= 71 && code <= 77) || code == 85 || code == 86) return WK_SNOW;
    if (code >= 95) return WK_STORM;
    if ((code >= 51 && code <= 67) || (code >= 80 && code <= 82)) return WK_RAIN;
    return WK_CLOUDY;
}

static const char *describe(int code) {
    if (code == 0) return "Clear";
    if (code <= 2) return "Partly cloudy";
    if (code == 3) return "Cloudy";
    if (code == 45 || code == 48) return "Fog";
    if (code >= 51 && code <= 67) return "Rain";
    if (code >= 71 && code <= 77) return "Snow";
    if (code >= 80 && code <= 82) return "Showers";
    if (code >= 95) return "Storm";
    return "";
}

static int rounded(double v) { return (int)floor(v + 0.5); }

static void set_kept_path(const char *place) {
    char safe[64];
    size_t i = 0;
    for (; place[i] && i < sizeof safe - 1; i++) {
        char c = place[i];
        safe[i] = (c >= '0' && c <= '9') || (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z') ? c : '_';
    }
    safe[i] = 0;
    snprintf(kept_path, sizeof kept_path, "%s/weather-%s.json", state_dir(), safe);
}

static int city_known(const char *app_dir, const char *name) {
    char path[320];
    snprintf(path, sizeof path, "%s/assets/cities.tsv", app_dir);
    char *text = read_file(path, NULL);
    if (!text) return 0;
    size_t n = strlen(name);
    int found = 0;
    for (char *line = strtok(text, "\n"); line && !found; line = strtok(NULL, "\n"))
        if (!strncmp(line, name, n) && line[n] == '\t') found = sscanf(line + n + 1, "%lf\t%lf", &place_lat, &place_lon) == 2;
    free(text);
    return found;
}

int weather_init(const char *app_dir, const Settings *s) {
    if (!s->weather_city[0] || !(s->show_weather || s->saver_forecast)) return 0;
    auto_place = !strcmp(s->weather_city, "@auto");
    if (!auto_place && !city_known(app_dir, s->weather_city)) return 0;
    set_kept_path(auto_place ? "auto" : s->weather_city);
    return 1;
}

static cJSON *to_json(const Weather *w) {
    cJSON *o = cJSON_CreateObject();
    cJSON_AddNumberToObject(o, "temp", w->temp);
    cJSON_AddStringToObject(o, "label", w->label);
    cJSON_AddStringToObject(o, "kind", KIND_NAMES[w->kind]);
    cJSON_AddBoolToObject(o, "day", w->day);
    cJSON_AddNumberToObject(o, "offset", w->offset);
    if (w->has_range) {
        cJSON_AddNumberToObject(o, "high", w->high);
        cJSON_AddNumberToObject(o, "low", w->low);
    }
    cJSON *hours = cJSON_AddArrayToObject(o, "hours");
    for (int i = 0; i < w->hour_count; i++) {
        cJSON *h = cJSON_CreateObject();
        cJSON_AddStringToObject(h, "t", w->hours[i].t);
        cJSON_AddNumberToObject(h, "temp", w->hours[i].temp);
        cJSON_AddStringToObject(h, "kind", KIND_NAMES[w->hours[i].kind]);
        cJSON_AddBoolToObject(h, "day", w->hours[i].day);
        cJSON_AddNumberToObject(h, "rain", w->hours[i].rain);
        cJSON_AddItemToArray(hours, h);
    }
    return o;
}

static WeatherKind kind_named(const cJSON *v) {
    for (int k = 0; k < 7 && cJSON_IsString(v); k++)
        if (!strcmp(v->valuestring, KIND_NAMES[k])) return (WeatherKind)k;
    return WK_CLOUDY;
}

static int number(const cJSON *o, const char *key) {
    const cJSON *v = cJSON_GetObjectItemCaseSensitive(o, key);
    return cJSON_IsNumber(v) ? (int)v->valuedouble : 0;
}

static int from_json(const cJSON *o, Weather *w) {
    memset(w, 0, sizeof *w);
    const cJSON *temp = cJSON_GetObjectItemCaseSensitive(o, "temp");
    if (!cJSON_IsNumber(temp)) return 0;
    w->temp = (int)temp->valuedouble;
    const cJSON *label = cJSON_GetObjectItemCaseSensitive(o, "label");
    if (cJSON_IsString(label)) snprintf(w->label, sizeof w->label, "%s", label->valuestring);
    w->kind = kind_named(cJSON_GetObjectItemCaseSensitive(o, "kind"));
    w->day = cJSON_IsTrue(cJSON_GetObjectItemCaseSensitive(o, "day"));
    w->offset = number(o, "offset");
    w->has_range = cJSON_HasObjectItem(o, "high");
    w->high = number(o, "high");
    w->low = number(o, "low");
    const cJSON *h;
    cJSON_ArrayForEach(h, cJSON_GetObjectItemCaseSensitive(o, "hours")) {
        const cJSON *t = cJSON_GetObjectItemCaseSensitive(h, "t");
        if (w->hour_count >= KEPT_HOURS || !cJSON_IsString(t) || strlen(t->valuestring) >= sizeof w->hours[0].t) continue;
        WeatherHour *x = &w->hours[w->hour_count++];
        snprintf(x->t, sizeof x->t, "%s", t->valuestring);
        x->temp = number(h, "temp");
        x->kind = kind_named(cJSON_GetObjectItemCaseSensitive(h, "kind"));
        x->day = cJSON_IsTrue(cJSON_GetObjectItemCaseSensitive(h, "day"));
        x->rain = number(h, "rain");
    }
    return 1;
}

static int load_kept(Weather *out, double *taken) {
    char *text = read_file(kept_path, NULL);
    if (!text) return 0;
    cJSON *j = cJSON_Parse(text);
    free(text);
    const cJSON *t = cJSON_GetObjectItemCaseSensitive(j, "t");
    int ok = cJSON_IsNumber(t) && from_json(cJSON_GetObjectItemCaseSensitive(j, "w"), out);
    if (ok) *taken = t->valuedouble;
    cJSON_Delete(j);
    return ok;
}

int weather_peek(Weather *out) {
    double taken;
    return load_kept(out, &taken);
}

static void keep(const Weather *w) {
    cJSON *j = cJSON_CreateObject();
    cJSON_AddNumberToObject(j, "t", (double)time(NULL));
    cJSON_AddItemToObject(j, "w", to_json(w));
    char *text = cJSON_PrintUnformatted(j);
    if (text) write_file(kept_path, text, strlen(text));
    free(text);
    cJSON_Delete(j);
}

static int find_auto_place(void) {
    Buffer b;
    if (http_get("https://ipwho.is/", NULL, "application/json", &b)) return 0;
    cJSON *j = cJSON_Parse(b.data);
    free(b.data);
    const cJSON *lat = cJSON_GetObjectItemCaseSensitive(j, "latitude"), *lon = cJSON_GetObjectItemCaseSensitive(j, "longitude");
    int ok = cJSON_IsTrue(cJSON_GetObjectItemCaseSensitive(j, "success")) && cJSON_IsNumber(lat) && cJSON_IsNumber(lon);
    if (ok) {
        place_lat = lat->valuedouble;
        place_lon = lon->valuedouble;
    }
    cJSON_Delete(j);
    return ok;
}

static double item_at(const cJSON *arr, int i, int *present) {
    const cJSON *v = cJSON_GetArrayItem(arr, i);
    *present = cJSON_IsNumber(v);
    return *present ? v->valuedouble : 0;
}

static int parse_forecast(const cJSON *d, Weather *w) {
    const cJSON *c = cJSON_GetObjectItemCaseSensitive(d, "current");
    const cJSON *temp = cJSON_GetObjectItemCaseSensitive(c, "temperature_2m");
    if (!cJSON_IsNumber(temp)) return 0;
    memset(w, 0, sizeof *w);
    const cJSON *code_item = cJSON_GetObjectItemCaseSensitive(c, "weather_code");
    int code = cJSON_IsNumber(code_item) ? (int)code_item->valuedouble : 0;
    const cJSON *is_day = cJSON_GetObjectItemCaseSensitive(c, "is_day");
    w->temp = rounded(temp->valuedouble);
    snprintf(w->label, sizeof w->label, "%s", describe(code));
    w->kind = kind_of(code);
    w->day = !(cJSON_IsNumber(is_day) && is_day->valuedouble == 0);
    w->offset = number(d, "utc_offset_seconds");

    const cJSON *h = cJSON_GetObjectItemCaseSensitive(d, "hourly");
    const cJSON *times = cJSON_GetObjectItemCaseSensitive(h, "time"), *temps = cJSON_GetObjectItemCaseSensitive(h, "temperature_2m");
    if (cJSON_IsArray(times) && cJSON_IsArray(temps)) {
        const cJSON *codes = cJSON_GetObjectItemCaseSensitive(h, "weather_code"), *days = cJSON_GetObjectItemCaseSensitive(h, "is_day");
        const cJSON *rains = cJSON_GetObjectItemCaseSensitive(h, "precipitation_probability");
        const cJSON *now = cJSON_GetObjectItemCaseSensitive(c, "time");
        const char *from = cJSON_IsString(now) ? now->valuestring : "";
        int n = cJSON_GetArraySize(times);
        for (int i = 0; i < n && w->hour_count < KEPT_HOURS; i++) {
            const cJSON *t = cJSON_GetArrayItem(times, i);
            int has_temp, has_code, has_day, has_rain;
            double v = item_at(temps, i, &has_temp);
            if (!cJSON_IsString(t) || strlen(t->valuestring) >= sizeof w->hours[0].t || !has_temp || strncmp(t->valuestring, from, HOUR_PREFIX) < 0) continue;
            double code_v = item_at(codes, i, &has_code), day_v = item_at(days, i, &has_day), rain_v = item_at(rains, i, &has_rain);
            WeatherHour *x = &w->hours[w->hour_count++];
            snprintf(x->t, sizeof x->t, "%s", t->valuestring);
            x->temp = rounded(v);
            x->kind = kind_of(has_code ? (int)code_v : 0);
            x->day = !cJSON_IsArray(days) || day_v != 0;
            x->rain = has_rain ? (int)rain_v : 0;
        }
    }
    const cJSON *daily = cJSON_GetObjectItemCaseSensitive(d, "daily");
    const cJSON *hi = cJSON_GetObjectItemCaseSensitive(daily, "temperature_2m_max"), *lo = cJSON_GetObjectItemCaseSensitive(daily, "temperature_2m_min");
    const cJSON *hi0 = cJSON_GetArrayItem(hi, 0), *lo0 = cJSON_GetArrayItem(lo, 0);
    if (cJSON_IsNumber(hi0) && cJSON_IsNumber(lo0)) {
        w->has_range = 1;
        w->high = rounded(hi0->valuedouble);
        w->low = rounded(lo0->valuedouble);
    }
    return 1;
}

int weather_read(Weather *out) {
    Weather kept;
    double taken;
    if (load_kept(&kept, &taken) && (double)time(NULL) - taken < MAX_AGE_S) {
        *out = kept;
        return 1;
    }
    if (auto_place && !find_auto_place()) return 0;
    char url[512];
    snprintf(url, sizeof url, "https://api.open-meteo.com/v1/forecast?latitude=%.15g&longitude=%.15g&current=temperature_2m,weather_code,is_day&hourly=temperature_2m,weather_code,is_day,precipitation_probability&daily=temperature_2m_max,temperature_2m_min&forecast_days=2&timezone=auto", place_lat, place_lon);
    Buffer b;
    if (http_get(url, NULL, "application/json", &b)) return 0;
    cJSON *d = cJSON_Parse(b.data);
    free(b.data);
    int ok = d && parse_forecast(d, out);
    cJSON_Delete(d);
    if (ok) {
        keep(out);
        note("weather: asked open-meteo, %d, %d hours", out->temp, out->hour_count);
    } else {
        note("weather: no weather in the answer");
    }
    return ok;
}

int weather_hours_ahead(const Weather *w, time_t now, int count, WeatherHour *out) {
    time_t there = now + w->offset;
    char here[24];
    strftime(here, sizeof here, "%Y-%m-%dT%H", gmtime(&there));
    int n = 0;
    for (int i = 0; i < w->hour_count && n < count; i++)
        if (strncmp(w->hours[i].t, here, HOUR_PREFIX) > 0) out[n++] = w->hours[i];
    return n;
}

int weather_check(int argc, char **argv) {
    if (argc < 1) {
        fprintf(stderr, "usage: lumen-saver --weather <app folder> [city]\n");
        return 1;
    }
    Settings s;
    settings_load(argv[0], &s);
    if (argc > 1) snprintf(s.weather_city, sizeof s.weather_city, "%s", argv[1]);
    s.show_weather = 1;
    if (!weather_init(argv[0], &s)) {
        printf("weather not wanted or place unknown ('%s')\n", s.weather_city);
        return 1;
    }
    Weather w;
    if (!weather_read(&w)) {
        printf("no reading\n");
        return 1;
    }
    printf("%s: %d, %s (%s), day %d, high %d low %d, offset %d, %d hours\n", s.weather_city, w.temp, w.label, KIND_NAMES[w.kind], w.day, w.high, w.low, w.offset, w.hour_count);
    WeatherHour next[7];
    int n = weather_hours_ahead(&w, time(NULL), 7, next);
    for (int i = 0; i < n; i++) printf("  %s %d %s day %d rain %d\n", next[i].t, next[i].temp, KIND_NAMES[next[i].kind], next[i].day, next[i].rain);
    return 0;
}
