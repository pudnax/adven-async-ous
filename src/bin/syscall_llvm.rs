#![feature(llvm_asm)]

fn main() {
    let message = "Hello from the Interrupt World!\n".to_string();
    syscall(message);
}

fn syscall(message: String) {
    let msg_ptr = message.as_ptr();
    let len = message.len();

    unsafe {
        llvm_asm!("
            mov     $$1, %rax
            mov     $$1, %rdi
            mov     $0, %rsi
            mov     $1, %rdx
            syscall
        "
        :
        : "r"(msg_ptr), "r"(len)
        : "rax", "rdi", "rsi", "rdx"
        )
    }
}
