#include "util.h"
#include <SDL.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <time.h>
#include <unistd.h>

static char dir[64];
static char log_path[96];
static SDL_mutex *log_lock;

void util_init(void) {
    snprintf(dir, sizeof dir, "/tmp/lumen-saver-%u", (unsigned)getuid());
    mkdir(dir, 0755);
    chmod(dir, 0755);
    snprintf(log_path, sizeof log_path, "%s/log", dir);
    log_lock = SDL_CreateMutex();
}

const char *state_dir(void) { return dir; }

void note(const char *fmt, ...) {
    if (log_lock) SDL_LockMutex(log_lock);
    FILE *f = fopen(log_path, "a");
    if (f) {
        char stamp[32];
        time_t now = time(NULL);
        strftime(stamp, sizeof stamp, "%Y-%m-%dT%H:%M:%SZ", gmtime(&now));
        fprintf(f, "%s nsaver ", stamp);
        va_list ap;
        va_start(ap, fmt);
        vfprintf(f, fmt, ap);
        va_end(ap);
        fputc('\n', f);
        fclose(f);
    }
    if (log_lock) SDL_UnlockMutex(log_lock);
}

char *read_file(const char *path, size_t *len) {
    FILE *f = fopen(path, "rb");
    if (!f) return NULL;
    fseek(f, 0, SEEK_END);
    long n = ftell(f);
    fseek(f, 0, SEEK_SET);
    char *buf = n >= 0 ? malloc((size_t)n + 1) : NULL;
    if (buf && fread(buf, 1, (size_t)n, f) != (size_t)n) { free(buf); buf = NULL; }
    fclose(f);
    if (!buf) return NULL;
    buf[n] = 0;
    if (len) *len = (size_t)n;
    return buf;
}

int write_file(const char *path, const void *data, size_t len) {
    char tmp[256];
    snprintf(tmp, sizeof tmp, "%s.tmp", path);
    FILE *f = fopen(tmp, "wb");
    if (!f) return -1;
    int ok = fwrite(data, 1, len, f) == len;
    ok = fclose(f) == 0 && ok;
    if (!ok || rename(tmp, path) != 0) { unlink(tmp); return -1; }
    return 0;
}

long status_kb(const char *field) {
    FILE *f = fopen("/proc/self/status", "r");
    if (!f) return -1;
    char line[128];
    long kb = -1;
    size_t n = strlen(field);
    while (fgets(line, sizeof line, f))
        if (!strncmp(line, field, n)) { kb = strtol(line + n + 1, NULL, 10); break; }
    fclose(f);
    return kb;
}
