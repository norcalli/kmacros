pub mod clearable;
pub use clearable::*;

pub use kmacros_shim::{self, *};
#[cfg(feature = "proc")]
pub use kproc_macros::*;

pub struct OptimizeCheckOutput<T, P> {
    pub function_name: &'static str,
    pub params: P,
    pub slow: T,
    pub fast: T,
}

impl<T, P> OptimizeCheckOutput<T, P> {
    pub fn assert_equal(self) -> T
    where
        T: PartialEq + std::fmt::Debug,
        P: std::fmt::Debug,
    {
        let Self {
            function_name,
            params,
            slow,
            fast,
        } = self;
        assert_eq!(
            slow, fast,
            "For function {function_name:?} with params {params:?}"
        );
        fast
    }
}
