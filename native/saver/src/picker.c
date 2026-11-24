#include "picker.h"
#include "http.h"
#include "util.h"
#include "../vendor/cJSON.h"
#include <SDL_image.h>
#include <ctype.h>
#include <dirent.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

#define LUMEN_PHOTO_DIR "/media/developer/apps/usr/palm/applications/com.samrenfrew.lumen/wallpapers"

enum { SHOWN_MAX = 40, FRAMES_MAX = 300, STACKS_EVERY_S = 6 * 3600, MAX_FILES = 64, PICKS_MAX = 128, KEY_MAX = 64, LOCAL_MAX = 64, POOL_MAX = PICKS_MAX + LOCAL_MAX };
static const double PAIR_BOX = (960.0 - 4) / 1080;
static const double SCREEN_BOX = 16.0 / 9;

static struct {
    double part, share;
} const TIERS[] = {{1, 0.5}, {1.0 / 3, 0.25}, {1.0 / 12, 0.25}};

static char app_dir[256];
static SaverConfig *cfg;
static char server_url[256];
static char server_key[128];
static char last_id[80];
static char files[MAX_FILES][300];
static int file_count;
static char picks[PICKS_MAX][KEY_MAX];
static int pick_count;
static char *catalog;

static void state_path(char *out, size_t n, const char *name) { snprintf(out, n, "%s/%s", state_dir(), name); }

static cJSON *read_json(const char *path) {
    char *text = read_file(path, NULL);
    if (!text) return NULL;
    cJSON *j = cJSON_Parse(text);
    free(text);
    return j;
}

static void write_json(const char *path, const cJSON *j) {
    char *text = cJSON_PrintUnformatted(j);
    if (text) { write_file(path, text, strlen(text)); free(text); }
}

static const char *str(const cJSON *obj, const char *key) {
    const cJSON *v = cJSON_GetObjectItemCaseSensitive(obj, key);
    return cJSON_IsString(v) ? v->valuestring : NULL;
}

static double num(const cJSON *obj, const char *key) {
    const cJSON *v = cJSON_GetObjectItemCaseSensitive(obj, key);
    return cJSON_IsNumber(v) ? v->valuedouble : 0;
}

static int valid_server(const char *url, const char *key) {
    if (strncmp(url, "http://", 7) && strncmp(url, "https://", 8)) return 0;
    for (const char *p = url; *p; p++)
        if (*p <= ' ' || *p == '"' || *p == '\'' || *p == '\\') return 0;
    if (!*key) return 0;
    for (const char *p = key; *p; p++)
        if (!((*p >= 'a' && *p <= 'z') || (*p >= 'A' && *p <= 'Z') || (*p >= '0' && *p <= '9') || *p == '-' || *p == '_')) return 0;
    return 1;
}

static int valid_id(const char *id) {
    size_t n = strlen(id);
    if (n != 36) return 0;
    for (const char *p = id; *p; p++)
        if (!((*p >= '0' && *p <= '9') || (*p >= 'a' && *p <= 'f') || *p == '-')) return 0;
    return 1;
}

static void list_files(void) {
    DIR *dir = opendir(LUMEN_PHOTO_DIR);
    if (!dir) return;
    struct dirent *e;
    while ((e = readdir(dir)) && file_count < MAX_FILES) {
        size_t n = strlen(e->d_name);
        if (n > 4 && !strcmp(e->d_name + n - 4, ".jpg") && !strstr(e->d_name, "-bg."))
            snprintf(files[file_count++], sizeof files[0], "%s/%s", LUMEN_PHOTO_DIR, e->d_name);
    }
    closedir(dir);
}

void picker_init(const char *dir, SaverConfig *config) {
    snprintf(app_dir, sizeof app_dir, "%s", dir);
    cfg = config;
    cfg->immich = 0;
    snprintf(cfg->sources, sizeof cfg->sources, "immich:recent");
    cfg->recent_days = 180;
    cfg->seconds = 30;

    char path[320];
    snprintf(path, sizeof path, "%s/config/lumen-config.json", app_dir);
    cJSON *file = read_json(path);
    const cJSON *c = cJSON_GetObjectItemCaseSensitive(file, "config");
    if (c) {
        const char *show = str(c, "saverShow");
        cfg->immich = show && !strcmp(show, "immich");
        const char *sources = str(c, "ambient");
        if (sources) snprintf(cfg->sources, sizeof cfg->sources, "%s", sources);
        if (num(c, "immichRecentDays") > 0) cfg->recent_days = (int)num(c, "immichRecentDays");
        const cJSON *wanted;
        cJSON_ArrayForEach(wanted, cJSON_GetObjectItemCaseSensitive(c, "wallpaperPicks"))
            if (cJSON_IsString(wanted) && strlen(wanted->valuestring) < KEY_MAX && pick_count < PICKS_MAX)
                snprintf(picks[pick_count++], KEY_MAX, "%s", wanted->valuestring);
        if (num(c, "ambientSeconds") >= 5) cfg->seconds = (int)num(c, "ambientSeconds");
    } else {
        note("config: no %s", path);
    }
    cJSON_Delete(file);

    snprintf(path, sizeof path, "%s/config/immich.json", app_dir);
    cJSON *srv = read_json(path);
    const char *url = str(srv, "url"), *key = str(srv, "key");
    if (url && key && valid_server(url, key)) {
        snprintf(server_url, sizeof server_url, "%s", url);
        size_t n = strlen(server_url);
        while (n && server_url[n - 1] == '/') server_url[--n] = 0;
        snprintf(server_key, sizeof server_key, "%s", key);
    }
    cJSON_Delete(srv);
    snprintf(path, sizeof path, "%s/assets/wallpapers.tsv", app_dir);
    catalog = read_file(path, NULL);
    list_files();
    note("config: show %s, sources %s, recent %d days, %d s a photo, server %s, %d picks, %d developer pictures",
         cfg->immich ? "immich" : "pictures", cfg->sources, cfg->recent_days, cfg->seconds, *server_url ? "set" : "none", pick_count, file_count);
}

/* ---- Immich requests ---- */

static cJSON *api_get(const char *path) {
    char url[512];
    snprintf(url, sizeof url, "%s%s", server_url, path);
    Buffer b;
    if (http_get(url, server_key, "application/json", &b)) return NULL;
    cJSON *j = cJSON_Parse(b.data);
    free(b.data);
    return j;
}

static cJSON *api_post(const char *path, const cJSON *body) {
    char url[512];
    snprintf(url, sizeof url, "%s%s", server_url, path);
    char *text = cJSON_PrintUnformatted(body);
    Buffer b;
    int rc = http_post_json(url, server_key, text, &b);
    free(text);
    if (rc) return NULL;
    cJSON *j = cJSON_Parse(b.data);
    free(b.data);
    return j;
}

static void since(int days, char *out, size_t n) {
    time_t t = time(NULL) - (time_t)days * 86400;
    strftime(out, n, "%Y-%m-%dT%H:%M:%S.000Z", gmtime(&t));
}

static cJSON *ask(int size, const char *taken_after, int favorite, const char *album) {
    cJSON *body = cJSON_CreateObject();
    cJSON_AddStringToObject(body, "type", "IMAGE");
    cJSON_AddBoolToObject(body, "withExif", 1);
    cJSON_AddBoolToObject(body, "withPeople", 1);
    cJSON_AddNumberToObject(body, "size", size);
    if (taken_after) cJSON_AddStringToObject(body, "takenAfter", taken_after);
    if (favorite) cJSON_AddBoolToObject(body, "isFavorite", 1);
    if (album) {
        cJSON *ids = cJSON_AddArrayToObject(body, "albumIds");
        cJSON_AddItemToArray(ids, cJSON_CreateString(album));
    }
    cJSON *list = api_post("/api/search/random", body);
    cJSON_Delete(body);
    if (!cJSON_IsArray(list)) { cJSON_Delete(list); return cJSON_CreateArray(); }
    return list;
}

static int in_array(const cJSON *arr, const char *s) {
    const cJSON *v;
    cJSON_ArrayForEach(v, arr) if (cJSON_IsString(v) && !strcmp(v->valuestring, s)) return 1;
    return 0;
}

static cJSON *tiered(int days, int size, int favorite, int all) {
    cJSON *out = cJSON_CreateArray();
    cJSON *seen = cJSON_CreateObject();
    for (size_t t = 0; t < sizeof TIERS / sizeof TIERS[0]; t++) {
        char after[40];
        since((int)lround(days * TIERS[t].part), after, sizeof after);
        cJSON *list = ask((int)lround(size * TIERS[t].share), TIERS[t].part == 1 && all ? NULL : after, favorite, NULL);
        cJSON *a;
        while ((a = cJSON_DetachItemFromArray(list, 0))) {
            const char *id = str(a, "id");
            if (id && !cJSON_HasObjectItem(seen, id)) {
                cJSON_AddTrueToObject(seen, id);
                cJSON_AddItemToArray(out, a);
            } else {
                cJSON_Delete(a);
            }
        }
        cJSON_Delete(list);
    }
    cJSON_Delete(seen);
    return out;
}

/* ---- shape, faces and placing ---- */

static double ratio(const cJSON *a) {
    const cJSON *exif = cJSON_GetObjectItemCaseSensitive(a, "exifInfo");
    double w = num(a, "width"), h = num(a, "height");
    int from_exif = !w;
    if (!w) w = num(exif, "exifImageWidth");
    if (!h) h = num(exif, "exifImageHeight");
    const char *o = str(exif, "orientation");
    if (from_exif && o && (!strcmp(o, "6") || !strcmp(o, "8") || !strcmp(o, "5") || !strcmp(o, "7"))) { double t = w; w = h; h = t; }
    return w > 0 && h > 0 ? w / h : 0;
}

static int has_people(const cJSON *a) {
    const cJSON *p = cJSON_GetObjectItemCaseSensitive(a, "people");
    return cJSON_IsArray(p) && cJSON_GetArraySize(p) > 0;
}

static cJSON *frames;

static void frames_load(void) {
    if (frames) return;
    char path[128];
    state_path(path, sizeof path, "frames.json");
    frames = read_json(path);
    if (!cJSON_IsObject(frames)) { cJSON_Delete(frames); frames = cJSON_CreateObject(); }
}

static void frames_save(void) {
    while (cJSON_GetArraySize(frames) > FRAMES_MAX) cJSON_DeleteItemFromArray(frames, 0);
    char path[128];
    state_path(path, sizeof path, "frames.json");
    write_json(path, frames);
}

static void frame_set(const char *id, double r, double x, double y) {
    cJSON_DeleteItemFromObjectCaseSensitive(frames, id);
    double v[3] = {round(r * 1000) / 1000, x < 0 ? x : round(x * 1000) / 1000, y < 0 ? y : round(y * 1000) / 1000};
    cJSON_AddItemToObject(frames, id, cJSON_CreateDoubleArray(v, 3));
}

static int frame_get(const char *id, double f[3]) {
    const cJSON *e = cJSON_GetObjectItemCaseSensitive(frames, id);
    if (!cJSON_IsArray(e) || cJSON_GetArraySize(e) != 3) return 0;
    for (int i = 0; i < 3; i++) f[i] = cJSON_GetArrayItem(e, i)->valuedouble;
    return 1;
}

static void save_frames(const cJSON *list) {
    frames_load();
    const cJSON *a;
    cJSON_ArrayForEach(a, list) {
        const char *id = str(a, "id");
        if (!id) continue;
        double known[3];
        if (frame_get(id, known) && known[1] >= 0) frame_set(id, known[0], known[1], known[2]);
        else frame_set(id, ratio(a), has_people(a) ? -2 : -1, -1);
    }
    frames_save();
}

static int face_centre(const cJSON *faces, double *cx, double *cy) {
    double x1 = 1, y1 = 1, x2 = 0, y2 = 0;
    const cJSON *f;
    cJSON_ArrayForEach(f, faces) {
        double w = num(f, "imageWidth"), h = num(f, "imageHeight");
        if (!w || !h || !cJSON_IsNumber(cJSON_GetObjectItemCaseSensitive(f, "boundingBoxX1"))) continue;
        x1 = fmin(x1, num(f, "boundingBoxX1") / w);
        y1 = fmin(y1, num(f, "boundingBoxY1") / h);
        x2 = fmax(x2, num(f, "boundingBoxX2") / w);
        y2 = fmax(y2, num(f, "boundingBoxY2") / h);
    }
    if (!(x2 > x1 && y2 > y1)) return 0;
    *cx = (x1 + x2) / 2;
    *cy = (y1 + y2) / 2;
    return 1;
}

static void read_faces(char ids[2][40], int count) {
    frames_load();
    int changed = 0;
    for (int i = 0; i < count; i++) {
        double f[3];
        if (!frame_get(ids[i], f) || f[1] != -2) continue;
        char path[96];
        snprintf(path, sizeof path, "/api/faces?id=%s", ids[i]);
        cJSON *list = api_get(path);
        double cx = -1, cy = -1;
        if (!cJSON_IsArray(list) || !face_centre(list, &cx, &cy)) cx = cy = -1;
        cJSON_Delete(list);
        frame_set(ids[i], f[0], cx, cy);
        changed = 1;
    }
    if (changed) frames_save();
}

static double clamp01(double n) { return n < 0 ? 0 : n > 1 ? 1 : n; }

static void place(const char *id, double box, float *px, float *py) {
    double f[3];
    if (!frame_get(id, f) || !f[0]) { *px = 0.5f; *py = 0.4f; return; }
    double k = fmax(box / f[0], 1);
    double w = f[0] * k, h = k;
    *px = (float)(f[1] >= 0 && w > box + 0.001 ? clamp01((box * 0.5 - f[1] * w) / (box - w)) : 0.5);
    *py = (float)(h <= 1.001 ? 0.5 : f[2] >= 0 ? clamp01((0.42 - f[2] * h) / (1 - h)) : 0.3);
}

/* ---- candidates ---- */

static const char *day_of(const cJSON *a, char out[11]) {
    const char *d = str(a, "localDateTime");
    if (!d || !*d) d = str(a, "fileCreatedAt");
    if (!d || strlen(d) < 10) return NULL;
    memcpy(out, d, 10);
    out[10] = 0;
    return out;
}

static int camera(const cJSON *a) {
    const char *make = str(cJSON_GetObjectItemCaseSensitive(a, "exifInfo"), "make");
    return make && *make;
}

static void shuffle(cJSON *arr) {
    int n = cJSON_GetArraySize(arr);
    for (int i = n - 1; i > 0; i--) {
        int j = rand() % (i + 1);
        if (i == j) continue;
        cJSON *a = cJSON_GetArrayItem(arr, i), *b = cJSON_GetArrayItem(arr, j);
        char *t = a->valuestring;
        a->valuestring = b->valuestring;
        b->valuestring = t;
    }
}

static cJSON *arrange(const cJSON *list, cJSON *days) {
    save_frames(list);
    cJSON *out = cJSON_CreateArray();
    const cJSON *tall[256];
    int tall_n = 0;
    const cJSON *a;
    cJSON_ArrayForEach(a, list) {
        const char *id = str(a, "id");
        if (!id || !valid_id(id)) continue;
        char d[11];
        if (day_of(a, d)) { cJSON_DeleteItemFromObjectCaseSensitive(days, id); cJSON_AddStringToObject(days, id, d); }
        double r = ratio(a);
        if (!r || r >= 1.2) cJSON_AddItemToArray(out, cJSON_CreateString(id));
        else if (r <= 1 / 1.2 && tall_n < 256) tall[tall_n++] = a;
    }
    int used[256] = {0};
    for (int i = 0; i < tall_n; i++) {
        if (used[i]) continue;
        used[i] = 1;
        char d[11], e[11];
        const char *da = day_of(tall[i], d);
        for (int j = i + 1; j < tall_n; j++) {
            if (used[j]) continue;
            const char *db = day_of(tall[j], e);
            if (!da || !db || strcmp(da, db)) {
                used[j] = 1;
                char pair[80];
                snprintf(pair, sizeof pair, "%s+%s", str(tall[i], "id"), str(tall[j], "id"));
                cJSON_AddItemToArray(out, cJSON_CreateString(pair));
                break;
            }
        }
    }
    shuffle(out);
    return out;
}

static cJSON *stacked_away(void) {
    char path[128];
    state_path(path, sizeof path, "stacks.json");
    cJSON *saved = read_json(path);
    cJSON *set = cJSON_CreateObject();
    if (saved && time(NULL) - (time_t)num(saved, "at") < STACKS_EVERY_S) {
        const cJSON *v;
        cJSON_ArrayForEach(v, cJSON_GetObjectItemCaseSensitive(saved, "ids")) if (cJSON_IsString(v)) cJSON_AddTrueToObject(set, v->valuestring);
        cJSON_Delete(saved);
        return set;
    }
    cJSON_Delete(saved);
    cJSON *stacks = api_get("/api/stacks");
    cJSON *ids = cJSON_CreateArray();
    const cJSON *s;
    cJSON_ArrayForEach(s, stacks) {
        const char *cover = str(s, "primaryAssetId");
        const cJSON *a;
        cJSON_ArrayForEach(a, cJSON_GetObjectItemCaseSensitive(s, "assets")) {
            const char *id = str(a, "id");
            if (id && (!cover || strcmp(id, cover))) { cJSON_AddItemToArray(ids, cJSON_CreateString(id)); cJSON_AddTrueToObject(set, id); }
        }
    }
    cJSON_Delete(stacks);
    cJSON *rec = cJSON_CreateObject();
    cJSON_AddNumberToObject(rec, "at", (double)time(NULL));
    cJSON_AddItemToObject(rec, "ids", ids);
    write_json(path, rec);
    cJSON_Delete(rec);
    return set;
}

static cJSON *keep(const cJSON *list, int broad, const cJSON *away, cJSON *days) {
    cJSON *kept = cJSON_CreateArray();
    const cJSON *a;
    cJSON_ArrayForEach(a, list) {
        const char *type = str(a, "type"), *id = str(a, "id");
        if (!id || (type && strcmp(type, "IMAGE")) || (broad && !camera(a)) || cJSON_HasObjectItem(away, id)) continue;
        cJSON_AddItemToArray(kept, cJSON_Duplicate(a, 1));
    }
    cJSON *out = arrange(kept, days);
    cJSON_Delete(kept);
    return out;
}

static void local_date(char out[11]) {
    time_t t = time(NULL);
    strftime(out, 11, "%Y-%m-%d", localtime(&t));
}

static cJSON *search_in(const char *sel, const cJSON *away, cJSON *days);

static cJSON *recent_batch(int size, const cJSON *away, cJSON *days) {
    cJSON *list = tiered(cfg->recent_days, size, 0, 0);
    cJSON *out = keep(list, 1, away, days);
    cJSON_Delete(list);
    return out;
}

static cJSON *search_in(const char *sel, const cJSON *away, cJSON *days) {
    if (strchr(sel, '|')) {
        char copy[512];
        snprintf(copy, sizeof copy, "%s", sel);
        char *parts[16];
        int n = 0;
        for (char *p = strtok(copy, "|"); p && n < 16; p = strtok(NULL, "|")) parts[n++] = p;
        return search_in(parts[rand() % n], away, days);
    }
    if (!strcmp(sel, "immich:memories")) {
        char path[64], d[11];
        local_date(d);
        snprintf(path, sizeof path, "/api/memories?for=%s", d);
        cJSON *memories = api_get(path);
        cJSON *all = cJSON_CreateArray();
        const cJSON *m;
        cJSON_ArrayForEach(m, memories) {
            const cJSON *a;
            cJSON_ArrayForEach(a, cJSON_GetObjectItemCaseSensitive(m, "assets")) cJSON_AddItemToArray(all, cJSON_Duplicate(a, 1));
        }
        cJSON_Delete(memories);
        cJSON *out = keep(all, 0, away, days);
        cJSON_Delete(all);
        if (cJSON_GetArraySize(out) > 0) return out;
        cJSON_Delete(out);
        return recent_batch(40, away, days);
    }
    if (!strcmp(sel, "immich:favorites")) {
        cJSON *list = tiered(365, 40, 1, 1);
        cJSON *out = keep(list, 0, away, days);
        cJSON_Delete(list);
        return out;
    }
    if (!strcmp(sel, "immich:recent")) return recent_batch(40, away, days);
    if (!strcmp(sel, "immich:recent-people")) {
        cJSON *list = tiered(cfg->recent_days, 120, 0, 0);
        cJSON *people = cJSON_CreateArray();
        const cJSON *a;
        cJSON_ArrayForEach(a, list) if (has_people(a)) cJSON_AddItemToArray(people, cJSON_Duplicate(a, 1));
        cJSON_Delete(list);
        cJSON *out = keep(people, 1, away, days);
        cJSON_Delete(people);
        if (cJSON_GetArraySize(out) > 0) return out;
        cJSON_Delete(out);
        return search_in("immich:recent", away, days);
    }
    if (!strncmp(sel, "immich:album:", 13) && valid_id(sel + 13)) {
        cJSON *list = ask(40, NULL, 0, sel + 13);
        cJSON *out = keep(list, 0, away, days);
        cJSON_Delete(list);
        return out;
    }
    cJSON *list = ask(40, NULL, 0, NULL);
    cJSON *out = keep(list, 1, away, days);
    cJSON_Delete(list);
    return out;
}

/* ---- the pool: what is left from the last search, and what was shown ---- */

static int split_ids(const char *pic, char ids[2][40]) {
    const char *plus = strchr(pic, '+');
    if (!plus) { snprintf(ids[0], 40, "%s", pic); return 1; }
    snprintf(ids[0], 40, "%.*s", (int)(plus - pic), pic);
    snprintf(ids[1], 40, "%s", plus + 1);
    return 2;
}

static int fresh(const char *pic, const cJSON *shown, char avoided[2][40], int avoided_n) {
    char ids[2][40];
    int n = split_ids(pic, ids);
    for (int i = 0; i < n; i++) {
        if (in_array(shown, ids[i])) return 0;
        for (int j = 0; j < avoided_n; j++) if (!strcmp(ids[i], avoided[j])) return 0;
    }
    return 1;
}

static int clashes(const char *pic, const cJSON *days, const cJSON *last) {
    char ids[2][40];
    int n = split_ids(pic, ids);
    for (int i = 0; i < n; i++) {
        const char *d = str(days, ids[i]);
        if (d && in_array(last, d)) return 1;
    }
    return 0;
}

static int take(cJSON *pool, int strict, char avoided[2][40], int avoided_n, char *out, size_t out_n) {
    cJSON *ids = cJSON_GetObjectItemCaseSensitive(pool, "ids");
    const cJSON *shown = cJSON_GetObjectItemCaseSensitive(pool, "shown");
    for (int i = cJSON_GetArraySize(ids) - 1; i >= 0; i--)
        if (!fresh(cJSON_GetArrayItem(ids, i)->valuestring, shown, avoided, avoided_n)) cJSON_DeleteItemFromArray(ids, i);
    const cJSON *days = cJSON_GetObjectItemCaseSensitive(pool, "days");
    const cJSON *last = cJSON_GetObjectItemCaseSensitive(pool, "last");
    int n = cJSON_GetArraySize(ids);
    for (int i = 0; i < n; i++) {
        const char *pic = cJSON_GetArrayItem(ids, i)->valuestring;
        if (strict && clashes(pic, days, last)) continue;
        snprintf(out, out_n, "%s", pic);
        cJSON_DeleteItemFromArray(ids, i);
        return 1;
    }
    return 0;
}

static void ensure(cJSON *obj, const char *key, int array) {
    const cJSON *v = cJSON_GetObjectItemCaseSensitive(obj, key);
    if (array ? cJSON_IsArray(v) : cJSON_IsObject(v)) return;
    cJSON_DeleteItemFromObjectCaseSensitive(obj, key);
    cJSON_AddItemToObject(obj, key, array ? cJSON_CreateArray() : cJSON_CreateObject());
}

static void finish(cJSON *pool, const char *pic) {
    cJSON *days = cJSON_GetObjectItemCaseSensitive(pool, "days");
    if (pic) {
        char ids[2][40];
        int n = split_ids(pic, ids);
        cJSON *shown = cJSON_CreateArray();
        cJSON *last = cJSON_CreateArray();
        for (int i = 0; i < n; i++) {
            cJSON_AddItemToArray(shown, cJSON_CreateString(ids[i]));
            const char *d = str(days, ids[i]);
            if (d) cJSON_AddItemToArray(last, cJSON_CreateString(d));
        }
        const cJSON *v;
        cJSON_ArrayForEach(v, cJSON_GetObjectItemCaseSensitive(pool, "shown"))
            if (cJSON_GetArraySize(shown) < SHOWN_MAX && cJSON_IsString(v)) cJSON_AddItemToArray(shown, cJSON_CreateString(v->valuestring));
        cJSON_ReplaceItemInObjectCaseSensitive(pool, "shown", shown);
        cJSON_ReplaceItemInObjectCaseSensitive(pool, "last", last);
    }
    cJSON *kept = cJSON_CreateObject();
    const cJSON *v;
    cJSON_ArrayForEach(v, cJSON_GetObjectItemCaseSensitive(pool, "ids")) {
        char ids[2][40];
        int n = split_ids(v->valuestring, ids);
        for (int i = 0; i < n; i++) { const char *d = str(days, ids[i]); if (d && !cJSON_HasObjectItem(kept, ids[i])) cJSON_AddStringToObject(kept, ids[i], d); }
    }
    if (pic) {
        char ids[2][40];
        int n = split_ids(pic, ids);
        for (int i = 0; i < n; i++) { const char *d = str(days, ids[i]); if (d && !cJSON_HasObjectItem(kept, ids[i])) cJSON_AddStringToObject(kept, ids[i], d); }
    }
    cJSON_ReplaceItemInObjectCaseSensitive(pool, "days", kept);
    char path[128];
    state_path(path, sizeof path, "pool.json");
    write_json(path, pool);
}

static int next_asset(const char *sel, const char *avoid, char *out, size_t out_n) {
    char tag[600];
    if (strstr(sel, "immich:recent")) snprintf(tag, sizeof tag, "%s@%d", sel, cfg->recent_days);
    else snprintf(tag, sizeof tag, "%s", sel);
    char path[128];
    state_path(path, sizeof path, "pool.json");
    cJSON *pool = read_json(path);
    const char *saved_tag = str(pool, "sel");
    if (!cJSON_IsObject(pool) || !saved_tag || strcmp(saved_tag, tag)) {
        cJSON_Delete(pool);
        pool = cJSON_CreateObject();
        cJSON_AddStringToObject(pool, "sel", tag);
    }
    ensure(pool, "ids", 1);
    ensure(pool, "shown", 1);
    ensure(pool, "last", 1);
    ensure(pool, "days", 0);
    char avoided[2][40];
    int avoided_n = *avoid ? split_ids(avoid, avoided) : 0;

    int found = take(pool, 1, avoided, avoided_n, out, out_n);
    if (!found) {
        cJSON *away = stacked_away();
        cJSON *batch_days = cJSON_CreateObject();
        cJSON *batch = search_in(sel, away, batch_days);
        cJSON_Delete(away);
        cJSON *days = cJSON_GetObjectItemCaseSensitive(pool, "days");
        const cJSON *d;
        cJSON_ArrayForEach(d, batch_days) {
            cJSON_DeleteItemFromObjectCaseSensitive(days, d->string);
            cJSON_AddStringToObject(days, d->string, d->valuestring);
        }
        cJSON_Delete(batch_days);
        cJSON *ids = cJSON_GetObjectItemCaseSensitive(pool, "ids");
        cJSON *merged = cJSON_Duplicate(batch, 1);
        const cJSON *v;
        cJSON_ArrayForEach(v, ids) if (!in_array(batch, v->valuestring)) cJSON_AddItemToArray(merged, cJSON_CreateString(v->valuestring));
        cJSON_ReplaceItemInObjectCaseSensitive(pool, "ids", merged);
        found = take(pool, 1, avoided, avoided_n, out, out_n) || take(pool, 0, avoided, avoided_n, out, out_n);
        if (!found && cJSON_GetArraySize(batch) > 0) {
            cJSON_ReplaceItemInObjectCaseSensitive(pool, "shown", cJSON_CreateArray());
            cJSON_ReplaceItemInObjectCaseSensitive(pool, "ids", cJSON_Duplicate(batch, 1));
            found = take(pool, 1, avoided, avoided_n, out, out_n) || take(pool, 0, avoided, avoided_n, out, out_n);
        }
        note("search: %d pictures from %s", cJSON_GetArraySize(batch), sel);
        cJSON_Delete(batch);
    }
    finish(pool, found ? out : NULL);
    cJSON_Delete(pool);
    return found;
}

/* ---- the picture itself ---- */

static SDL_Surface *decode(const void *data, size_t len) {
    SDL_RWops *rw = SDL_RWFromConstMem(data, (int)len);
    return rw ? IMG_Load_RW(rw, 1) : NULL;
}

static void ready_path(char *out, size_t n, int i) {
    if (i < 0) state_path(out, n, "ready.json");
    else { char name[24]; snprintf(name, sizeof name, "ready-%d.jpg", i); state_path(out, n, name); }
}

static void keep_ready(const Picture *pic, Buffer jpg[2]) {
    char described[128];
    ready_path(described, sizeof described, -1);
    unlink(described);
    for (int i = 0; i < pic->count; i++) { char p[128]; ready_path(p, sizeof p, i); write_file(p, jpg[i].data, jpg[i].len); }
    cJSON *meta = cJSON_CreateObject();
    cJSON_AddStringToObject(meta, "id", pic->id);
    cJSON_AddNumberToObject(meta, "count", pic->count);
    float xy[4] = {pic->x[0], pic->y[0], pic->x[1], pic->y[1]};
    cJSON_AddItemToObject(meta, "xy", cJSON_CreateFloatArray(xy, 4));
    char p[128];
    ready_path(p, sizeof p, -1);
    write_json(p, meta);
    cJSON_Delete(meta);
}

static int take_ready(Picture *out) {
    char p[128];
    ready_path(p, sizeof p, -1);
    cJSON *meta = read_json(p);
    const char *id = str(meta, "id");
    int count = (int)num(meta, "count");
    const cJSON *xy = cJSON_GetObjectItemCaseSensitive(meta, "xy");
    int ok = id && (count == 1 || count == 2) && cJSON_GetArraySize(xy) == 4;
    if (ok) {
        memset(out, 0, sizeof *out);
        snprintf(out->id, sizeof out->id, "%s", id);
        out->count = count;
        for (int i = 0; i < 2; i++) {
            out->x[i] = (float)cJSON_GetArrayItem(xy, i * 2)->valuedouble;
            out->y[i] = (float)cJSON_GetArrayItem(xy, i * 2 + 1)->valuedouble;
        }
        for (int i = 0; i < count && ok; i++) {
            char jp[128];
            ready_path(jp, sizeof jp, i);
            out->img[i] = IMG_Load(jp);
            ok = out->img[i] != NULL;
        }
        if (!ok) picture_free(out);
    }
    cJSON_Delete(meta);
    return ok;
}

static int from_immich(Picture *out) {
    char pic[80];
    if (!next_asset(cfg->sources, last_id, pic, sizeof pic)) return 0;
    char ids[2][40];
    int n = split_ids(pic, ids);
    read_faces(ids, n);
    memset(out, 0, sizeof *out);
    snprintf(out->id, sizeof out->id, "%s", pic);
    out->count = n;
    Buffer jpg[2] = {{0}};
    for (int i = 0; i < n; i++) {
        if (n == 2) { out->x[i] = (float)i; place(ids[i], PAIR_BOX, &(float){0}, &out->y[i]); }
        else place(ids[i], SCREEN_BOX, &out->x[i], &out->y[i]);
        char url[512];
        snprintf(url, sizeof url, "%s/api/assets/%s/thumbnail?size=preview", server_url, ids[i]);
        if (http_get(url, server_key, "image/*", &jpg[i]) || !(out->img[i] = decode(jpg[i].data, jpg[i].len))) {
            for (int j = 0; j <= i; j++) free(jpg[j].data);
            picture_free(out);
            return 0;
        }
    }
    keep_ready(out, jpg);
    for (int i = 0; i < n; i++) free(jpg[i].data);
    return 1;
}

static int from_files(Picture *out) {
    if (!file_count) return 0;
    int i = rand() % file_count;
    if (file_count > 1 && !strcmp(files[i], last_id)) i = (i + 1) % file_count;
    memset(out, 0, sizeof *out);
    snprintf(out->id, sizeof out->id, "%.79s", files[i]);
    out->count = 1;
    out->x[0] = 0.5f;
    out->y[0] = 0.5f;
    out->img[0] = IMG_Load(files[i]);
    return out->img[0] != NULL;
}

static int catalog_url(const char *id, char *url, size_t n) {
    size_t id_len = strlen(id);
    for (const char *line = catalog; line && *line;) {
        const char *end = strchr(line, '\n');
        size_t len = end ? (size_t)(end - line) : strlen(line);
        if (len > id_len + 1 && !strncmp(line, id, id_len) && line[id_len] == '\t') {
            snprintf(url, n, "%.*s", (int)(len - id_len - 1), line + id_len + 1);
            return 1;
        }
        line += len + (end ? 1 : 0);
    }
    return 0;
}

static int local_number(const char *key) {
    if (strncmp(key, "local:", 6) || !key[6] || strlen(key) > 8) return -1;
    for (const char *p = key + 6; *p; p++) if (!isdigit((unsigned char)*p)) return -1;
    return atoi(key + 6);
}

static void local_path(char *out, size_t n, int number) { snprintf(out, n, "%s/photos/default-%d.jpg", app_dir, number); }

static int is_pick(const char *key) {
    int number = local_number(key);
    char path[320];
    if (number >= 0) { local_path(path, sizeof path, number); return access(path, R_OK) == 0; }
    return !strncmp(key, "cloud:", 6) && catalog_url(key + 6, path, sizeof path);
}

static int pool_of(char pool[][KEY_MAX], int locals_only) {
    int n = 0;
    for (int i = 0; i < pick_count && !locals_only; i++)
        if (is_pick(picks[i])) memcpy(pool[n++], picks[i], KEY_MAX);
    int none_picked = !n;
    for (int i = 0; i < LOCAL_MAX && none_picked; i++) {
        snprintf(pool[n], KEY_MAX, "local:%d", i);
        if (is_pick(pool[n])) n++;
    }
    return n;
}

static const char *other_than_last(char pool[][KEY_MAX], int n) {
    int fit[POOL_MAX], count = 0;
    for (int i = 0; i < n; i++) if (strcmp(pool[i], last_id)) fit[count++] = i;
    if (count) return pool[fit[rand() % count]];
    return n ? pool[0] : NULL;
}

static void resized_url(const char *url, char *out, size_t n) {
    const char *bare = strstr(url, "://");
    bare = bare ? bare + 3 : url;
    size_t o = (size_t)snprintf(out, n, "https://images.weserv.nl/?url=");
    for (const char *p = bare; *p && o + 4 < n; p++) {
        if (isalnum((unsigned char)*p) || strchr("-_.!~*'()", *p)) out[o++] = *p;
        else o += (size_t)snprintf(out + o, n - o, "%%%02X", (unsigned char)*p);
    }
    snprintf(out + o, n - o, "&w=1920&h=1080&fit=cover&q=82&output=jpg");
}

static SDL_Surface *load_pick(const char *key) {
    char path[320];
    int number = local_number(key);
    if (number >= 0) { local_path(path, sizeof path, number); return IMG_Load(path); }
    state_path(path, sizeof path, "cloud-");
    size_t o = strlen(path);
    snprintf(path + o, sizeof path - o, "%s.jpg", key + 6);
    SDL_Surface *kept = IMG_Load(path);
    if (kept) return kept;
    char url[400], resized[900];
    if (!catalog_url(key + 6, url, sizeof url)) return NULL;
    resized_url(url, resized, sizeof resized);
    Buffer jpg;
    if (http_get(resized, NULL, "image/*", &jpg)) return NULL;
    SDL_Surface *img = decode(jpg.data, jpg.len);
    if (img) write_file(path, jpg.data, jpg.len);
    free(jpg.data);
    return img;
}

static int from_picks(Picture *out) {
    char pool[POOL_MAX][KEY_MAX];
    const char *key = other_than_last(pool, pool_of(pool, 0));
    if (!key) return 0;
    SDL_Surface *img = load_pick(key);
    if (!img && strncmp(key, "local:", 6)) {
        note("picture: %s did not load, a local one instead", key);
        key = other_than_last(pool, pool_of(pool, 1));
        img = key ? load_pick(key) : NULL;
    }
    if (!img) return 0;
    memset(out, 0, sizeof *out);
    snprintf(out->id, sizeof out->id, "%s", key);
    out->count = 1;
    out->x[0] = 0.5f;
    out->y[0] = 0.5f;
    out->img[0] = img;
    return 1;
}

int picker_next(Picture *out) {
    Uint32 t0 = SDL_GetTicks();
    int ok = 0;
    const char *how = "";
    static int opened;
    int opening = !opened;
    opened = 1;
    if (opening && cfg->immich && *server_url && (ok = take_ready(out))) how = "kept from last time";
    else if (cfg->immich && *server_url && (ok = from_immich(out))) how = "from Immich";
    else if ((ok = from_picks(out))) how = "from the wallpaper pictures";
    else if ((ok = from_files(out))) how = "from the developer folder";
    if (ok) {
        snprintf(last_id, sizeof last_id, "%s", out->id);
        note("picture: %s %s, %d photo%s, %u ms", out->id, how, out->count, out->count > 1 ? "s" : "", SDL_GetTicks() - t0);
    }
    return ok;
}

void picture_free(Picture *pic) {
    for (int i = 0; i < 2; i++) if (pic->img[i]) { SDL_FreeSurface(pic->img[i]); pic->img[i] = NULL; }
    pic->count = 0;
}
