use crate::runtime::{Js, ThreadPoolTaskKind, RUNTIME};
use std::io::Read;
use std::{fs, thread};

pub struct Fs {}
impl Fs {
    pub fn read(path: &'static str, cb: impl Fn(Js) + 'static) {
        let work = move || {
            thread::sleep(std::time::Duration::from_secs(1));
            let mut buffer = String::new();
            fs::File::open(&path)
                .unwrap()
                .read_to_string(&mut buffer)
                .unwrap();
            Js::String(buffer)
        };
        let rt = unsafe { &mut *RUNTIME };
        rt.register_event_threadpool(work, ThreadPoolTaskKind::FileRead, cb);
    }
}
