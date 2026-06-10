/// KairosOS Example WASM Plugin — Metrics Collector
/// Compile: clang --sysroot=... -O2 -nostdlib -Wl,--export=run -Wl,--export=alloc -o metrics_collector.wasm metrics_collector.c

#include "../../sdk/kairos_plugin.h"

static int json_escape(char *out, int max, const char *in) {
    int i = 0, j = 0;
    while (in[i] && j < max - 6) {
        if (in[i] == '"' || in[i] == '\\') { out[j++] = '\\'; }
        out[j++] = in[i++];
    }
    out[j] = 0;
    return j;
}

WASM_EXPORT int run(int input_len, int _unused) {
    char buf[4096];
    char method_buf[128];
    char value_buf[64];

    // Read input args
    int actual_len = kairos_read_args((int)buf, 4096);
    if (actual_len <= 0) {
        buf[0] = 0;
    }
    buf[actual_len < 4096 ? actual_len : 4095] = 0;

    // Parse method from input JSON (simplified: skip to "method":"...")
    char *method = 0;
    const char *p = buf;
    while (*p) {
        if (p[0] == '"' && p[1] == 'm' && p[2] == 'e' && p[3] == 't' && p[4] == 'h' && p[5] == 'o' && p[6] == 'd') {
            p += 8;
            while (*p && *p != '"') p++;
            if (*p == '"') p++;
            method = p;
            break;
        }
        p++;
    }

    // Build response based on method
    if (method && method[0] == 'r' && method[1] == 'u' && method[2] == 'n') {
        return respond(buf, 4096, "{\"output\":\"metrics_collector: running\"}");
    }
    else if (method && method[0] == 'h' && method[1] == 'a' && method[2] == 'n' && method[3] == 'd' && method[4] == 'l' && method[5] == 'e' && method[6] == '_' && method[7] == 'e' && method[8] == 'v' && method[9] == 'e' && method[10] == 'n' && method[11] == 't') {
        return respond(buf, 4096, "{\"output\":\"metrics_collector: event handled\"}");
    }
    else {
        return respond(buf, 4096, "{\"error\":\"unknown method\"}");
    }
}
