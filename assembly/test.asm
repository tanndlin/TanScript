section .data
    format db "%d", 0

section .text
    extern _printf
    global _main

_main:
    ; Set eax and ebx to 3 and 4 respectively
    mov eax, 3
    add eax, 4

    ; Print result to the terminal
    push eax
    push format
    call _printf
    add esp, 8