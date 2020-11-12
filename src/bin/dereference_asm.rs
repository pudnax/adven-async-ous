#![feature(asm)]
fn main() {
    let t = 100;
    let t_ptr: *const usize = &t;
    let x = dereference(t_ptr);

    println!("{}", x);
}

fn dereference(ptr: *const usize) -> usize {
    let res: usize;
    unsafe {
        asm!("mov {1}, {0}",
        out(reg) res,
        in(reg) ptr
        )
    };
    res
}
