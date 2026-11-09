use tanscript_emulator::{Options, run};

#[test]
fn hello_world() {
    let asm = "BITS 64
global main
extern printf
extern exit

SECTION .rdata
	data_0 db \"Hello %s, %d + %d = %d\", 10, 0
	data_1 db \"world\", 0

SECTION .text

add_two:
	push rbp
	mov rbp, rsp
	mov rax, rcx
	add rax, rdx
	pop rbp
	ret
main:
	push rbp
	mov rbp, rsp
	sub rsp, 48
	mov rcx, 40
	mov rdx, 2
	call add_two
	mov QWORD [rsp + 32], rax
	mov rcx, data_0
	mov rdx, data_1
	mov r8, 40
	mov r9, 2
	call printf
	mov rsp, rbp
	pop rbp
	sub rsp, 40
	xor rcx, rcx
	call exit
";
    let run = run(asm, Options::default());
    assert_eq!(run.stdout_lossy(), "Hello world, 40 + 2 = 42\n");
    assert_eq!(run.result.unwrap(), 0);
}
