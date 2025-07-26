BITS 64

global main
extern printf
extern ExitProcess
section .data
    msg db "Hello from assembly!", 10, 0
    fmt db "Result: %d %d %d", 10, 0     ; format string with newline and null terminator

section .text
main:
    sub rsp, 40          ; shadow space required by Windows calling convention

    mov rcx, msg         ; format string is 2nd argument in RDX
    call printf
    mov rax, 10          ; load 10 into rax
    add rax, 12          ; add 12 => rax = 22
    mov rdx, rax         ; move result to RCX (1st argument to printf)
    mov r8, 69           ; move result to RCX (1st argument to printf)
    mov r9, 420          ; move result to RCX (1st argument to printf)
    mov rcx, fmt         ; format string is 2nd argument in RDX
    call printf

    add rsp, 40
    xor rcx, rcx         ; return 0
    call ExitProcess