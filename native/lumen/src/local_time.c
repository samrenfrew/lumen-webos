#include <time.h>

void lumen_local_time(long long unix_seconds, int out[6]) {
    time_t at = (time_t)unix_seconds;
    struct tm t;
    localtime_r(&at, &t);
    out[0] = t.tm_hour; out[1] = t.tm_min; out[2] = t.tm_sec;
    out[3] = t.tm_wday; out[4] = t.tm_mday; out[5] = t.tm_mon;
}
