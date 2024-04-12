section .data
    format db "%d", 10, 0

section .text
    global _main
    extern _printf
        
 _main:
    ; Add the numbers 5 and 3
    push 3
    push 5
    call add
    add esp, 8

    ; Print the result
    push eax
    push format
    call _printf
    add esp, 8
    ret