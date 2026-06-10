#ifndef KAIROS_PLUGIN_H
#define KAIROS_PLUGIN_H

#define WASM_EXPORT __attribute__((visibility("default")))

extern int kairos_read_args(char *buf, int max_len);

static inline int respond(char *buf, int max_len, const char *json) {
    int i = 0;
    while (json[i] && i < max_len - 1) {
        buf[i] = json[i];
        i++;
    }
    buf[i] = 0;
    return i;
}

#endif
