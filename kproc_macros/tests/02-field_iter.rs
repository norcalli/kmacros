use kproc_macros::FieldIter;
use std::any::Any;
use std::fmt::Debug;

#[derive(Debug, FieldIter)]
/// outer
#[allow(dead_code)]
#[field_iter(debug_iter = "dyn std::fmt::Debug")]
struct Foo<T> {
    x: bool,
    b: String,
    #[field_iter(skip(debug_iter))]
    t: T,
}

trait AnyDebug: Any + Debug {
    fn as_any(&self) -> &dyn Any;
}

impl<T> AnyDebug for T
where
    T: Any + Debug,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, FieldIter)]
/// outer
#[allow(dead_code)]
#[field_iter(debug_iter_mut = "dyn std::fmt::Debug", any_iter = "dyn Any")]
#[field_iter(any_debug_iter = "dyn AnyDebug")]
#[field_iter(bound(debug_iter_mut = "T: Debug"))]
#[field_iter(bound(any_iter = "T: 'static"))]
#[field_iter(bound(any_debug_iter = "T: 'static + AnyDebug"))]
struct Bar<T> {
    x: bool,
    b: String,
    t: T,
}

fn main() {
    Foo {
        x: true,
        b: format!("Test"),
        t: 64u64,
    }
    .debug_iter(|name, value: &dyn Debug| {
        eprintln!("{name} = {value:?}");
        assert_eq!(
            format!("{value:?}"),
            match name {
                "x" => "true",
                "b" => r#""Test""#,
                "t" => "64",
                _ => unreachable!(),
            }
        );
        None::<()>
    });

    Bar {
        x: true,
        b: format!("Test"),
        t: 64u64,
    }
    .debug_iter_mut(|name, value: &mut dyn Debug| {
        eprintln!("{name} = {value:?}");
        assert_eq!(
            format!("{value:?}"),
            match name {
                "x" => "true",
                "b" => r#""Test""#,
                "t" => "64",
                _ => unreachable!(),
            }
        );
        None::<()>
    });

    Bar {
        x: true,
        b: format!("Test"),
        t: 64u64,
    }
    .any_iter(|name, value| {
        dbg!((name, value.is::<bool>()));
        None::<()>
    });

    Bar {
        x: true,
        b: format!("Test"),
        t: 64u64,
    }
    .any_debug_iter(|name, value| {
        dbg!((name, value.as_any().is::<bool>(), value));
        None::<()>
    });
}
