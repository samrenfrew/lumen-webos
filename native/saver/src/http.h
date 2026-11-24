#pragma once
#include <stddef.h>

typedef struct { char *data; size_t len; } Buffer;

void http_init(volatile int *abort_flag);
int http_get(const char *url, const char *key, const char *accept, Buffer *out);
int http_post_json(const char *url, const char *key, const char *body, Buffer *out);
