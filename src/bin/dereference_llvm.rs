#![feature(llvm_asm)]
fn main() {
    let t = 100;
    let t_ptr: *const usize = &t;
    let x = dereference(t_ptr);

    println!("{}", x);
}

fn dereference(ptr: *const usize) -> usize {
    let res: usize;
    unsafe { llvm_asm!("mov ($1), $0":"=r"(res): "r"(ptr)) };
    res
}
