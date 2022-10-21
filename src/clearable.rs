pub trait Clearable {
    fn clear(&mut self);

    fn cleared(&mut self) -> &mut Self {
        self.clear();
        self
    }

    fn cleared_if(&mut self, clear: bool) -> &mut Self {
        if clear {
            self.clear();
        }
        self
    }
}

#[cfg(not(feature = "no_std"))]
mod impls {
    use super::*;

    impl<T> Clearable for Vec<T> {
        fn clear(&mut self) {
            Vec::clear(self);
        }
    }

    use std::collections::HashMap;
    use std::collections::HashSet;

    impl<K, V> Clearable for HashMap<K, V> {
        fn clear(&mut self) {
            HashMap::clear(self);
        }
    }

    impl<K> Clearable for HashSet<K> {
        fn clear(&mut self) {
            HashSet::clear(self);
        }
    }

    impl<T> Clearable for Option<T> {
        fn clear(&mut self) {
            *self = None;
        }
    }
}
