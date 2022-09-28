.data:
	message: db "Hello, World", 10

.text:
	global _start

_start:
	mov rax, 1
	mov rdi, 1
	mov rsi, message
	mov rdx, 13
	syscall

	mov rax, 1 ; sys_write
	mov rdi, 1 ; stdout
	push '3' ; data
	mov rsi, rsp ; data addr
	mov rdx, 1 ; data_len
	syscall
	pop rax

	mov rax, 1 ; sys_write
	mov rdi, 1 ; stdout
	push 10 ; data
	mov rsi, rsp ; data addr
	mov rdx, 1 ; data_len
	syscall
	pop rax

	mov rax, 60
	xor rdi, rdi
	syscall