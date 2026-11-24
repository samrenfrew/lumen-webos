#include <glib.h>
#include <luna-service2/lunaservice.h>
#include <stdlib.h>
#include <string.h>

typedef void (*lumen_luna_listener)(const char *reply, void *user);
typedef struct { lumen_luna_listener call; void *user; int once; } Listener;

static GMainContext *context;
static LSHandle *handle;

static char *failure(LSError *error) {
    char *text = g_strdup_printf("%d %s", error->error_code, error->message ? error->message : "");
    LSErrorFree(error);
    return text;
}

char *lumen_luna_open(const char *name) {
    LSError error;
    LSErrorInit(&error);
    if (!context) context = g_main_context_new();
    if (!LSRegister(name, &handle, &error)) { handle = NULL; return failure(&error); }
    if (!LSGmainContextAttach(handle, context, &error)) return failure(&error);
    return NULL;
}

static bool heard(LSHandle *sh, LSMessage *message, void *user) {
    (void)sh;
    Listener *listener = user;
    listener->call(LSMessageGetPayload(message), listener->user);
    if (listener->once) g_free(listener);
    return true;
}

char *lumen_luna_send(const char *uri, const char *payload, int subscribe, int timeout_ms, lumen_luna_listener call, void *user) {
    if (!handle) return g_strdup("0 the bus is not open");
    LSError error;
    LSErrorInit(&error);
    Listener *listener = g_new(Listener, 1);
    listener->call = call;
    listener->user = user;
    listener->once = !subscribe;
    LSMessageToken token = 0;
    bool sent = subscribe ? LSCall(handle, uri, payload, heard, listener, &token, &error) : LSCallOneReply(handle, uri, payload, heard, listener, &token, &error);
    if (!sent) { g_free(listener); return failure(&error); }
    if (!subscribe && timeout_ms > 0) LSCallSetTimeout(handle, token, timeout_ms, NULL);
    return NULL;
}

void lumen_luna_wait(void) { if (context) g_main_context_iteration(context, TRUE); }
void lumen_luna_wake(void) { if (context) g_main_context_wakeup(context); }
void lumen_luna_free(char *text) { g_free(text); }
