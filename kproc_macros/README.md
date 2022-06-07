See the `tests` for more examples.

# `kproc_macros::explain`

Creates a function with the suffix `_explain` that additionally takes a callback
of `name: &str, expr: Option<&str>, value: &dyn Display` for all of the
statements on the top-level of the function which are simple `let` bindings.

The `expr` is only none if `#[no_expr]` is used or for the return value.

For the return value of the function, the callback is called with `name == ""`
and `expr == None`.

Used for explaining the steps in a math function. A callback is used to permit
use in GUI rendering as well.

```rust
use kproc_macros::explain;

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

# `kproc_macros::optimized`

Expects a function with two blocks: the slow block, and then the fast block.

(Technically the second block can just be an expression, but the slow block
*has* to be a block).

This will generate a `foo_slow` and `foo_check` function with the same
parameters, although `_check` returns a `kmacros_shim::OptimizeCheckOutput` which contains
an `.assert_equal()` method which pretty prints the result along with the
results from the previous calls, the parameters, and the function name.

This requires the parameters are able to be reused between two calls, so they're
either `Copy` or a `&mut` reference which isn't borrowed from in the result.

```rust
use kproc_macros::optimized;

#[optimized]
fn foo(a: u32, b: f64) -> u32 {
    {
        let mut r = 0.;
        for _ in 0..a {
            r += b;
        }
        r as u32
    }
    {
        // a * b as u32
        (a as f64 * b) as u32
    }
}

fn main() {
    foo_check(17, 3.1).assert_equal();
    assert_eq!(foo_slow(17, 3.1), foo(17, 3.1));
}
```

```rust
use kproc_macros::optimized;

#[optimized]
fn foo(a: u32, b: f64) -> u32 {
    {
        let mut r = 0.;
        for _ in 0..a {
            r += b;
        }
        r as u32
    }
    (a as f64 * b) as u32
}

fn main() {
    foo_check(17, 3.1).assert_equal();
    assert_eq!(foo_slow(17, 3.1), foo(17, 3.1));
}
```
