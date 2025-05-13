.intel_syntax noprefix
.text
.globl platform_setjmp
platform_setjmp:
    mov [rdi], rbx
    lea rax, [rsp+8]
    mov [rdi+8], rax
    mov [rdi+16], rbp
    mov [rdi+24], r12
    mov [rdi+32], r13
    mov [rdi+40], r14
    mov [rdi+48], r15
    mov rax, [rsp]
    mov [rdi+56], rax
    xor rax, rax
    ret

.globl platform_longjmp
platform_longjmp:
    mov rbx, [rdi]
    mov rsp, [rdi+8]
    mov rbp, [rdi+16]
    mov r12, [rdi+24]
    mov r13, [rdi+32]
    mov r14, [rdi+40]
    mov r15, [rdi+48]
    mov rdx, [rdi+56]
    mov rax, rsi
    test eax, eax
    jnz jnz_wasmtime_longjmp
    inc eax
jnz_wasmtime_longjmp:
    jmp rdx
