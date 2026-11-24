#include "http.h"
#include "util.h"
#include <curl/curl.h>
#include <stdlib.h>
#include <string.h>

enum { TIMEOUT_S = 15, MAX_BODY = 32 * 1024 * 1024 };

static volatile int *quitting;

void http_init(volatile int *abort_flag) {
    quitting = abort_flag;
    curl_global_init(CURL_GLOBAL_DEFAULT);
}

static size_t collect(void *data, size_t size, size_t n, void *user) {
    Buffer *b = user;
    size_t add = size * n;
    if (b->len + add > MAX_BODY) return 0;
    char *grown = realloc(b->data, b->len + add + 1);
    if (!grown) return 0;
    memcpy(grown + b->len, data, add);
    b->data = grown;
    b->len += add;
    b->data[b->len] = 0;
    return add;
}

static int stop_when_quitting(void *user, curl_off_t a, curl_off_t b, curl_off_t c, curl_off_t d) {
    (void)user; (void)a; (void)b; (void)c; (void)d;
    return quitting && *quitting;
}

static int request(const char *url, const char *key, const char *accept, const char *body, Buffer *out) {
    out->data = NULL;
    out->len = 0;
    CURL *c = curl_easy_init();
    if (!c) return -1;
    char key_header[256];
    snprintf(key_header, sizeof key_header, "x-api-key: %s", key);
    char accept_header[64];
    snprintf(accept_header, sizeof accept_header, "accept: %s", accept);
    struct curl_slist *headers = curl_slist_append(NULL, key_header);
    headers = curl_slist_append(headers, accept_header);
    if (body) headers = curl_slist_append(headers, "content-type: application/json");
    curl_easy_setopt(c, CURLOPT_URL, url);
    curl_easy_setopt(c, CURLOPT_HTTPHEADER, headers);
    curl_easy_setopt(c, CURLOPT_TIMEOUT, (long)TIMEOUT_S);
    curl_easy_setopt(c, CURLOPT_FAILONERROR, 1L);
    curl_easy_setopt(c, CURLOPT_NOSIGNAL, 1L);
    curl_easy_setopt(c, CURLOPT_WRITEFUNCTION, collect);
    curl_easy_setopt(c, CURLOPT_WRITEDATA, out);
    curl_easy_setopt(c, CURLOPT_NOPROGRESS, 0L);
    curl_easy_setopt(c, CURLOPT_XFERINFOFUNCTION, stop_when_quitting);
    if (body) curl_easy_setopt(c, CURLOPT_POSTFIELDS, body);
    CURLcode rc = curl_easy_perform(c);
    curl_slist_free_all(headers);
    curl_easy_cleanup(c);
    if (rc != CURLE_OK) {
        const char *path = strstr(url, "/api/");
        if (!(quitting && *quitting)) note("http: %s failed: %s", path ? path : "?", curl_easy_strerror(rc));
        free(out->data);
        out->data = NULL;
        out->len = 0;
        return -1;
    }
    return 0;
}

int http_get(const char *url, const char *key, const char *accept, Buffer *out) { return request(url, key, accept, NULL, out); }

int http_post_json(const char *url, const char *key, const char *body, Buffer *out) { return request(url, key, "application/json", body, out); }
