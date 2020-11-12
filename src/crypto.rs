use crate::runtime::{Js, ThreadPoolTaskKind, RUNTIME};

pub struct Crypto;
impl Crypto {
    pub fn encrypt(n: usize, cb: impl Fn(Js) + 'static + Clone) {
        let work = move || {
            fn fibonacchi(n: usize) -> usize {
                match n {
                    0 => 0,
                    1 => 1,
                    _ => fibonacchi(n - 1) + fibonacchi(n - 2),
                }
            }

            let fib = fibonacchi(n);
            Js::Int(fib)
        };

        let rt = unsafe { &mut *RUNTIME };
        rt.register_event_threadpool(work, ThreadPoolTaskKind::Encrypt, cb);
    }
}
