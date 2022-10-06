#[derive(kmacros::Clearable)]
pub struct Foo {
    a: Vec<i32>,
    #[clearable(skip)]
    b: u32,
    #[clearable(default)]
    e: u32,
    #[clearable(expr = "{} = 3")]
    f: u32,
    #[clearable(expr = "{} += 1")]
    g: u32,
    #[clearable(expr = "{}.push(123)")]
    h: Vec<i32>,
    #[clearable(raw_expr = "self.i = format!(\"{}\", 123)")]
    i: String,
}

fn main() {
    use kmacros::Clearable;
    let mut foo = Foo {
        a: vec![1, 2, 3],
        b: 3,
        e: 100,
        f: 1,
        g: 0,
        h: vec![],
        i: Default::default(),
    };
    foo.clear();
    assert_eq!(foo.a, vec![]);
    assert_eq!(foo.e, 0);
    assert_eq!(foo.f, 3);
    assert_eq!(foo.g, 1);
    assert_eq!(foo.h, vec![123]);
    assert_eq!(foo.i, "123");
    foo.clear();
    assert_eq!(foo.a, vec![]);
    assert_eq!(foo.e, 0);
    assert_eq!(foo.f, 3);
    assert_eq!(foo.g, 2);
    assert_eq!(foo.h, vec![123, 123]);
    assert_eq!(foo.i, "123");
    foo.a.push(1);
    assert_eq!(foo.a, vec![1]);
    foo.a.clear();
    assert_eq!(foo.a, vec![]);
    assert_eq!(foo.b, 3);
}

