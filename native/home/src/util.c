#include "util.h"
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <time.h>
#include <unistd.h>

static char log_path[96];

void util_init(void) {
    char dir[64];
    snprintf(dir, sizeof dir, "/tmp/lumen-ntest-%u", (unsigned)getuid());
    mkdir(dir, 0755);
    chmod(dir, 0755);
    snprintf(log_path, sizeof log_path, "%s/log", dir);
}

void note(const char *fmt, ...) {
    char stamp[32], line[1200];
    time_t now = time(NULL);
    strftime(stamp, sizeof stamp, "%Y-%m-%dT%H:%M:%SZ", gmtime(&now));
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(line, sizeof line, fmt, ap);
    va_end(ap);
    FILE *f = fopen(log_path, "a");
    if (f) {
        fprintf(f, "%s ntest %s\n", stamp, line);
        fclose(f);
    }
    if (isatty(2)) fprintf(stderr, "ntest %s\n", line);
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
