use std::{collections::HashMap, marker::PhantomData, mem::ManuallyDrop};

#[repr(transparent)]
#[derive(Debug)]
struct Aliased<T, U: DropBehavior> {
    real: Box<T>,
    _marker: PhantomData<U>,
}

impl<T, U> Drop for Aliased<T, U>
where
    U: DropBehavior,
{
    fn drop(&mut self) {
        if U::should_drop() {
            let _ = self.real;
        }
    }
}

trait DropBehavior {
    fn should_drop() -> bool;
}

#[derive(Debug)]
struct NoDrop;
#[derive(Debug)]
struct Dodrop;

impl DropBehavior for Dodrop {
    fn should_drop() -> bool {
        true
    }
}

impl DropBehavior for NoDrop {
    fn should_drop() -> bool {
        false
    }
}

fn main() {
    let inner = Box::into_raw(Box::new(5));

    let left: ManuallyDrop<Aliased<_, Dodrop>> = ManuallyDrop::new(Aliased {
        real: unsafe { Box::from_raw(inner) },
        _marker: PhantomData,
    });
    let right: ManuallyDrop<Aliased<_, NoDrop>> = ManuallyDrop::new(Aliased {
        real: unsafe { Box::from_raw(inner) },
        _marker: PhantomData,
    });

    let mut left_hashmap = HashMap::new();
    left_hashmap.insert(0, left);
    left_hashmap.retain(|_, v| *v.real != 0);

    let mut right_hashmap = HashMap::new();
    right_hashmap.insert(0, right);

    let right_hashmap: &mut HashMap<i32, Aliased<usize, Dodrop>> =
        unsafe { &mut *((&mut right_hashmap) as *mut _ as *mut _) };
    right_hashmap.retain(|_k, v| *v.real != 0);

    dbg!(&right_hashmap);
}
