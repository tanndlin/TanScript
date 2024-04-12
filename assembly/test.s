	.file	"test.c"
	.text
	.def	___main;	.scl	2;	.type	32;	.endef
	.section .rdata,"dr"
LC0:
	.ascii "%d\0"
	.text
	.globl	_main
	.def	_main;	.scl	2;	.type	32;	.endef
_main:
LFB17:
	.cfi_startproc
	pushl	%ebp
	.cfi_def_cfa_offset 8
	.cfi_offset 5, -8
	movl	%esp, %ebp
	.cfi_def_cfa_register 5
	andl	$-16, %esp
	subl	$96, %esp
	call	___main
	movl	$69, 92(%esp)
	movl	$420, 88(%esp)
	movl	$1, 84(%esp)
	movl	$2, 80(%esp)
	movl	$3, 76(%esp)
	movl	$4, 72(%esp)
	movl	$5, 68(%esp)
	movl	$6, 64(%esp)
	movl	$7, 60(%esp)
	movl	$8, 56(%esp)
	movl	$9, 52(%esp)
	movl	$10, 48(%esp)
	movl	$11, 44(%esp)
	movl	$12, 40(%esp)
	movl	$13, 36(%esp)
	movl	$14, 32(%esp)
	movl	92(%esp), %edx
	movl	88(%esp), %eax
	addl	%eax, %edx
	movl	84(%esp), %eax
	addl	%eax, %edx
	movl	80(%esp), %eax
	addl	%eax, %edx
	movl	76(%esp), %eax
	addl	%eax, %edx
	movl	72(%esp), %eax
	addl	%eax, %edx
	movl	68(%esp), %eax
	addl	%eax, %edx
	movl	64(%esp), %eax
	addl	%eax, %edx
	movl	60(%esp), %eax
	addl	%eax, %edx
	movl	56(%esp), %eax
	addl	%eax, %edx
	movl	52(%esp), %eax
	addl	%eax, %edx
	movl	48(%esp), %eax
	addl	%eax, %edx
	movl	44(%esp), %eax
	addl	%eax, %edx
	movl	40(%esp), %eax
	addl	%eax, %edx
	movl	36(%esp), %eax
	addl	%eax, %edx
	movl	32(%esp), %eax
	addl	%edx, %eax
	movl	%eax, 28(%esp)
	movl	28(%esp), %eax
	movl	%eax, 4(%esp)
	movl	$LC0, (%esp)
	call	_printf
	movl	$0, %eax
	leave
	.cfi_restore 5
	.cfi_def_cfa 4, 4
	ret
	.cfi_endproc
LFE17:
	.ident	"GCC: (MinGW.org GCC-8.2.0-3) 8.2.0"
	.def	_printf;	.scl	2;	.type	32;	.endef
