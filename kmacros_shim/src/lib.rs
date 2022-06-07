pub struct HiddenVariable;
/// ```
/// use std::any::TypeId;
/// fn check<T: 'static>(_x: T) {
///   assert_eq!(TypeId::of::<kmacros_shim::HiddenVariable>(), TypeId::of::<T>());
/// }
/// let x = 1;
/// kmacros_shim::rename!(let a = x);
/// assert_eq!(a, 1);
/// check(x);
/// kmacros_shim::rename!(let b = a;);
/// assert_eq!(b, 1);
/// check(a);
/// let y = 2;
/// kmacros_shim::rename! {
///   let b = y;
///   let a = b;
/// }
/// assert_eq!(a, 2);
/// check(b);
/// check(y);
/// let x = 1;
/// let y = 2;
/// kmacros_shim::rename! {
///   let a = x;
///   let b = y;
/// }
/// assert_eq!(a, 1);
/// assert_eq!(b, 2);
/// check(x);
/// check(y);
/// kmacros_shim::rename!(let a = a;);
/// check(a);
/// ```
/// ```compile_fail
/// let x = 1;
/// kmacros_shim::rename!(let a = x);
/// assert_eq!(x, 1);
/// ```
#[macro_export]
macro_rules! rename {
    ($(let $p:pat_param = $from:ident);+$(;)?) => {
        $(
            let $p = $from;
            #[allow(unused_variables)]
            let $from = $crate::HiddenVariable;
        )+
    };
}

/// Unwrap the Option value or break.
#[macro_export]
macro_rules! or_continue {
    ( $wrapper:expr ) => {
        match $wrapper {
            Some(v) => v,
            None => continue,
        }
    };
}

/// ```
/// use std::any::TypeId;
/// fn check<T: 'static>(_x: T) {
///   assert_eq!(TypeId::of::<piex::macros::HiddenVariable>(), TypeId::of::<T>());
/// }
/// let x = 1;
/// piex::hide!(x);
/// check(x);
/// ```
#[macro_export]
macro_rules! hide {
    ($x:ident) => {
        #[allow(unused_variables)]
        let $x = $crate::macros::HiddenVariable;
    };
}

pub struct OptimizeCheckOutput<T, P> {
    pub function_name: &'static str,
    pub params: P,
    pub slow: T,
    pub fast: T,
}

impl<T, P> OptimizeCheckOutput<T, P> {
    pub fn assert_equal(self)
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
        assert_eq!(slow, fast, "For function {function_name:?} with params {params:?}");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
