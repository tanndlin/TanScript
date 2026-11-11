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

fn compile_and_run(source: &str, options: Options) -> tanscript_emulator::Run {
    let mut sources = tanscript::source_map::SourceMap::default();
    let file = sources.add(std::path::PathBuf::from("test.tan"), source.to_string());
    let asm =
        tanscript::compile(file, &mut sources, None).unwrap_or_else(|_| panic!("compile failed"));
    run(&asm, options)
}

#[test]
fn time_and_rand() {
    let source = r#"
        srand(time(0));
        printf("%d %d\n", rand(), rand());
    "#;
    let run = compile_and_run(
        source,
        Options {
            time: 1,
            ..Options::default()
        },
    );
    // MSVC's rand() after srand(1).
    assert_eq!(run.stdout_lossy(), "41 18467\n");
    assert_eq!(run.result.unwrap(), 0);
}

#[test]
fn realloc_keeps_contents() {
    let source = r#"
        let a: *u32 = malloc(8);
        a[0] = 7;
        a[1] = 9;
        a = realloc(a, 4096);
        a[1000] = 1;
        printf("%d %d\n", a[0], a[1]);
    "#;
    let run = compile_and_run(source, Options::default());
    assert_eq!(run.stdout_lossy(), "7 9\n");
    assert_eq!(run.result.unwrap(), 0);
}
