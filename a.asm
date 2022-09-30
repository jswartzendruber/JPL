.data:
	message: db "Hello, World", 10

.text:
	global _start

_start:
	mov rsi, message
	mov rdx, 13
	call print_string

	mov rdi, '3'
	call print_char

	mov rdi, 10 ; newline
	call print_char

	mov rax, 60 ; sys_exit
	xor rdi, rdi ; return code
	syscall

; rdi - char c
print_char:
	push rdi
	mov rsi, rsp ; char*
	mov rdx, 1 ; len
	mov rax, 1 ; sys_write
	mov rdi, 1 ; stdout
	syscall
	pop rdi
	ret

; rsi - char* message
; rdx - int length
print_string:
	mov rax, 1 ; sys_write
	mov rdi, 1 ; stdout
	syscall
	ret