trait Querializer {}

trait Generic {
    fn generic_fn<Q: Querializer>(&self, querializer: Q);
}

impl<'a, T: ?Sized> Querializer for &'a T where T: Querializer {}

impl<'a, T: ?Sized> Generic for Box<T>
where
    T: Generic,
{
    fn generic_fn<Q: Querializer>(&self, querializer: Q) {
        (**self).generic_fn(querializer)
    }
}

trait ErasedGeneric {
    fn erased_fn(&self, querializer: &dyn Querializer);
}

impl Generic for dyn ErasedGeneric {
    fn generic_fn<Q: Querializer>(&self, querializer: Q) {
        self.erased_fn(&querializer)
    }
}

impl<T> ErasedGeneric for T
where
    T: Generic,
{
    fn erased_fn(&self, querializer: &dyn Querializer) {
        self.generic_fn(querializer)
    }
}

fn main() {
    struct T;
    impl Querializer for T {}

    struct S;
    impl Generic for S {
        fn generic_fn<Q: Querializer>(&self, _querializer: Q) {
            println!("quering the real S");
        }
    }

    let trait_object: Box<dyn ErasedGeneric> = Box::new(S);

    trait_object.generic_fn(T);
}
