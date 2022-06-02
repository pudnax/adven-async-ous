fn main() {
    let t = 100;
    let t_ptr: *const usize = &t;
    let x = dereference(t_ptr);

    println!("{}", x);
}

fn dereference(ptr: *const usize) -> usize {
    let res: usize;
    unsafe {
        std::arch::asm!("mov {0}, [{1}]",
        out(reg) res,
        in(reg) ptr
        )
    };
    res
}
