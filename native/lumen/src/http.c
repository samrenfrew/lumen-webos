#include <curl/curl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

enum { TIMEOUT_S = 20, MAX_BODY = 32 * 1024 * 1024 };
typedef struct { unsigned char *data; size_t len; } Body;

static size_t collect(void *data, size_t size, size_t n, void *user) {
    Body *b = user;
    size_t add = size * n;
    if (b->len + add > MAX_BODY) return 0;
    unsigned char *grown = realloc(b->data, b->len + add);
    if (!grown) return 0;
    memcpy(grown + b->len, data, add);
    b->data = grown;
    b->len += add;
    return add;
}

long lumen_http_request(const char *url, const char *key, const char *post_json, unsigned char **data, size_t *len) {
    static int ready;
    if (!ready) { curl_global_init(CURL_GLOBAL_DEFAULT); ready = 1; }
    Body body = {NULL, 0};
    CURL *c = curl_easy_init();
    if (!c) return -1;
    struct curl_slist *headers = NULL;
    if (key) {
        char key_header[256];
        snprintf(key_header, sizeof key_header, "x-api-key: %s", key);
        headers = curl_slist_append(headers, key_header);
    }
    if (post_json) {
        headers = curl_slist_append(headers, "content-type: application/json");
        curl_easy_setopt(c, CURLOPT_POSTFIELDS, post_json);
    }
    if (headers) curl_easy_setopt(c, CURLOPT_HTTPHEADER, headers);
    curl_easy_setopt(c, CURLOPT_URL, url);
    curl_easy_setopt(c, CURLOPT_FOLLOWLOCATION, key ? 0L : 1L);
    curl_easy_setopt(c, CURLOPT_TIMEOUT, (long)TIMEOUT_S);
    curl_easy_setopt(c, CURLOPT_NOSIGNAL, 1L);
    curl_easy_setopt(c, CURLOPT_USERAGENT, "Lumen");
    curl_easy_setopt(c, CURLOPT_WRITEFUNCTION, collect);
    curl_easy_setopt(c, CURLOPT_WRITEDATA, &body);
    long status = -1;
    if (curl_easy_perform(c) == CURLE_OK) curl_easy_getinfo(c, CURLINFO_RESPONSE_CODE, &status);
    curl_easy_cleanup(c);
    curl_slist_free_all(headers);
    if (status != 200) { free(body.data); body.data = NULL; body.len = 0; }
    *data = body.data;
    *len = body.len;
    return status;
}

long lumen_http_get(const char *url, unsigned char **data, size_t *len) { return lumen_http_request(url, NULL, NULL, data, len); }

void lumen_http_free(unsigned char *data) { free(data); }
