#pragma once

void util_init(void);
void note(const char *fmt, ...) __attribute__((format(printf, 1, 2)));
long status_kb(const char *field);
