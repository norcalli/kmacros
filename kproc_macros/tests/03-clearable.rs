use kproc_macros::Clearable;

#[derive(Clearable)]
pub struct Foo {
    a: Vec<i32>,
    #[clearable(skip)]
    b: u32,
    #[clearable(default)]
    e: u32,
}

fn main() {
    use kmacros_shim::Clearable;
    let mut foo = Foo {
        a: vec![1, 2, 3],
        b: 3,
        e: 100,
    };
    foo.clear();
    assert_eq!(foo.e, 0);
    assert_eq!(foo.a, vec![]);
    foo.a.push(1);
    assert_eq!(foo.a, vec![1]);
    foo.a.clear();
    assert_eq!(foo.a, vec![]);
    assert_eq!(foo.b, 3);
}

