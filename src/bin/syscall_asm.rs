#![feature(asm)]

fn main() {
    let message = "Hello from the Interrupt World!\n".to_string();
    syscall(message);
}

fn syscall(message: String) {
    let msg_ptr = message.as_ptr();
    let len = message.len();

    unsafe {
        asm!(
            "syscall",
            in("rax") 1,
            in("rdi") 1,
            in("rsi") msg_ptr,
            in("rdx") len,
        )
    }
}
