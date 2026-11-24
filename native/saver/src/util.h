#pragma once
#include <stddef.h>

void util_init(void);
void note(const char *fmt, ...) __attribute__((format(printf, 1, 2)));
const char *state_dir(void);
char *read_file(const char *path, size_t *len);
int write_file(const char *path, const void *data, size_t len);
long status_kb(const char *field);
