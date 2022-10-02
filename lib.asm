.text:
	global print_char
	global print_int
	global print_string

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

; edi - i32 i
print_int:
	mov r12, 0
	mov eax, edi
print_int1:
	mov edx, 0
	mov ecx, 10
	div ecx ; eax /= ecx

	push rdx
	inc r12

	cmp eax, 0
	jg print_int1 ; push chars on stack until int is 0

print_int2:
	pop rdi
	add edi, 0x30 ; convert to char
	call print_char
	dec r12

	cmp r12, 0
	jg print_int2 ; print all chars on stack
	ret

; rsi - char* message
; rdx - int length
print_string:
	mov rax, 1 ; sys_write
	mov rdi, 1 ; stdout
	syscall
	ret