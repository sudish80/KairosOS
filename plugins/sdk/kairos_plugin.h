/// KairosOS WASM Plugin SDK — C header for building plugins
/// Compile:  /opt/wasi-sdk/bin/clang --sysroot=/opt/wasi-sdk/share/wasi-sysroot -O2 -nostdlib -Wl,--export=run -Wl,--export=alloc -o plugin.wasm plugin.c
///
/// Memory layout:
///   - Export "memory" (single page, growable)
///   - Export "alloc"  (fn(size: i32) -> ptr: i32) — allocate WASM memory
///   - Export "run"    (fn(input_len: i32, _: i32) -> output_len: i32) — main entry
///   - Import "env.log"     (fn(ptr: i32, len: i32)) — log a string
///   - Import "env.read_args" (fn(ptr: i32, len: i32) -> i32) — read input args

#ifndef KAIROS_PLUGIN_SDK_H
#define KAIROS_PLUGIN_SDK_H

#define WASM_EXPORT __attribute__((visibility("default")))
#define WASM_IMPORT __attribute__((import_module("env"), import_name("log")))
#define WASM_IMPORT_ARGS __attribute__((import_module("env"), import_name("read_args")))

/// Log a message through the KairosOS host
void WASM_IMPORT kairos_log(int ptr, int len);

/// Read input arguments into buffer
int WASM_IMPORT_ARGS kairos_read_args(int ptr, int len);

/// Convenience: log a string
static void log_str(const char *msg) {
    int len = 0;
    while (msg[len]) len++;
    kairos_log((int)msg, len);
}

/// Allocate WASM memory (provided by the module)
extern unsigned char __heap_base;
WASM_EXPORT int alloc(int size) {
    static int heap_ptr = 0;
    int ptr = heap_ptr;
    heap_ptr += size;
    return ptr;
}

/// Helper: write JSON response and return its length
static int respond(char *buf, int max_len, const char *json) {
    int i = 0;
    while (json[i] && i < max_len - 1) { buf[i] = json[i]; i++; }
    buf[i] = 0;
    return i;
}

#endif /* KAIROS_PLUGIN_SDK_H */
