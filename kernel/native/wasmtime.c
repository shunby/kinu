#include <stdint.h>
#include <stdbool.h>

// We don't have threads, so no need to store the pointer in TLS
static unsigned char *WASMTIME_TLS_PTR = 0;

typedef struct {
    uint64_t rbx;
    uint64_t rsp;
    uint64_t rbp;
    uint64_t r12;
    uint64_t r13;
    uint64_t r14;
    uint64_t r15;
    uint64_t rip;
} jmpbuf_t;

extern int platform_setjmp(jmpbuf_t *jmp_buf);
extern void platform_longjmp(jmpbuf_t *jmp_buf, int32_t val);

void wasmtime_tls_set(unsigned char *val) {
    WASMTIME_TLS_PTR = val;
}

unsigned char *wasmtime_tls_get() {
    return WASMTIME_TLS_PTR;
}

bool wasmtime_setjmp(const uint8_t **jmp_buf_out,
                     bool (*callback)(uint8_t *, uint8_t *), uint8_t *payload,
                     uint8_t *callee) {
    jmpbuf_t jmpbuf;
    if (platform_setjmp(&jmpbuf) != 0) 
        return false;
    *jmp_buf_out = (uint8_t *)&jmpbuf; 
    return callback(payload, callee);
}

void wasmtime_longjmp(const uint8_t *jmp_buf) {
    platform_longjmp((jmpbuf_t *)jmp_buf, 42);
}
