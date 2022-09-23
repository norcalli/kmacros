use kproc_macros::FieldIter;
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

#[derive(Debug, FieldIter)]
/// outer
#[allow(dead_code)]
#[field_iter(debug_iter_mut = "dyn std::fmt::Debug")]
#[field_iter(bound(debug_iter_mut = "T: Debug"))]
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
}
