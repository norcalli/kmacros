```rust
use kexplain::explain;

#[explain]
fn foo(a: u32, b: f64) -> u32 {
    let _x = a * b as u32;
    #[no_expr]
    let x = a * b as u32;
    #[skip]
    let _y = a * b as u32;
    x * 3
}

struct Foo;

impl Foo {
    #[explain]
    fn bar(&self, a: u32, b: f64) -> u32 {
        let _x = a * b as u32;
        #[no_expr]
        let x = a * b as u32;
        #[skip]
        let _y = a * b as u32;
        x * 3
    }
}

fn main() {
    assert_eq!(6, foo(1, 2.));
    assert_eq!(6, foo_explain(1, 2., |name, expr, value| {
        println!("{name} {expr:?} {value}");
    }));
    assert_eq!(6, Foo.bar(1, 2.));
    assert_eq!(6, Foo.bar_explain(1, 2., |name, expr, value| {
        println!("{name} {expr:?} {value}");
    }));
}
```

Example stdout:
```text
STDOUT:
┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈
a None 1
b None 2
_x Some("a * b as u32") 2
x None 2
 None 6
a None 1
b None 2
_x Some("a * b as u32") 2
x None 2
 None 6
┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈
```
