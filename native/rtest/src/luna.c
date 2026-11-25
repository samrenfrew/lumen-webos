#include <glib.h>
#include <luna-service2/lunaservice.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef void (*lumen_luna_listener)(const char *reply, void *user);
typedef struct { lumen_luna_listener call; void *user; } Listener;

static GMainContext *context;
static LSHandle *handle;

static char *failure(LSError *error) {
    char *text = g_strdup_printf("!%d %s", error->error_code, error->message ? error->message : "");
    LSErrorFree(error);
    return text;
}

char *lumen_luna_open(const char *name) {
    LSError error;
    LSErrorInit(&error);
    if (!context) context = g_main_context_new();
    if (handle) { LSUnregister(handle, NULL); handle = NULL; }
    if (!LSRegister(name, &handle, &error)) { handle = NULL; return failure(&error); }
    if (!LSGmainContextAttach(handle, context, &error)) return failure(&error);
    return NULL;
}

static bool one_reply(LSHandle *sh, LSMessage *message, void *slot) {
    (void)sh;
    *(char **)slot = g_strdup(LSMessageGetPayload(message));
    return true;
}

char *lumen_luna_call(const char *uri, const char *payload, int timeout_ms) {
    if (!handle) return g_strdup("!not open");
    LSError error;
    LSErrorInit(&error);
    char *reply = NULL;
    LSMessageToken token = 0;
    if (!LSCallOneReply(handle, uri, payload, one_reply, &reply, &token, &error)) return failure(&error);
    gint64 deadline = g_get_monotonic_time() + (gint64)timeout_ms * 1000;
    while (!reply && g_get_monotonic_time() < deadline)
        if (!g_main_context_iteration(context, FALSE)) g_usleep(1000);
    if (!reply) {
        LSCallCancel(handle, token, NULL);
        return g_strdup("!timeout");
    }
    return reply;
}

static bool each_reply(LSHandle *sh, LSMessage *message, void *user) {
    (void)sh;
    Listener *listener = user;
    listener->call(LSMessageGetPayload(message), listener->user);
    return true;
}

char *lumen_luna_subscribe(const char *uri, const char *payload, lumen_luna_listener call, void *user) {
    if (!handle) return g_strdup("!not open");
    LSError error;
    LSErrorInit(&error);
    Listener *listener = g_new(Listener, 1);
    listener->call = call;
    listener->user = user;
    if (!LSCall(handle, uri, payload, each_reply, listener, NULL, &error)) return failure(&error);
    return NULL;
}

void lumen_luna_pump(void) {
    if (context) while (g_main_context_iteration(context, FALSE)) {}
}

void lumen_luna_free(char *text) { g_free(text); }
