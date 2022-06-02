use std::{
    fmt::{self, Display},
    mem::transmute,
};

struct BoxedDisplay {
    data: *mut (),
    vtable: &'static DisplayVtable<()>,
}

struct DisplayVtable<T> {
    fmt: unsafe fn(*mut T, &mut fmt::Formatter<'_>) -> fmt::Result,
    drop: unsafe fn(*mut T),
}

impl<T: Display> DisplayVtable<T> {
    fn new() -> &'static Self {
        unsafe fn fmt<T: Display>(this: *mut T, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            (*this).fmt(f)
        }

        unsafe fn drop<T>(this: *mut T) {
            Box::from_raw(this);
        }

        &Self { fmt, drop }
    }
}

impl BoxedDisplay {
    fn new<T: Display + 'static>(t: T) -> Self {
        Self {
            data: Box::into_raw(Box::new(t)) as _,
            vtable: unsafe { transmute(DisplayVtable::<T>::new()) },
        }
    }
}

impl Display for BoxedDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe { (self.vtable.fmt)(self.data, f) }
    }
}

impl Drop for BoxedDisplay {
    fn drop(&mut self) {
        unsafe { (self.vtable.drop)(self.data) }
    }
}

fn get_char_of_int(give_char: bool) -> BoxedDisplay {
    if give_char {
        BoxedDisplay::new('C')
    } else {
        BoxedDisplay::new(64)
    }
}

fn show(v: impl Display) {
    println!("{v}")
}

fn main() {
    show(get_char_of_int(true));
    show(get_char_of_int(false));
}
