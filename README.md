Derived glob macro with the same name as the structure

# Examples

```rust
use fields_glob::fields_glob;

#[derive(fields_glob, Debug, Default)]
struct Foo {
    x: i32,
    y: i32,
    z: i32,
}

let y = 2;
let foo = Foo! { x: 1, z: 3, * };
assert_eq!(foo.x, 1);
assert_eq!(foo.y, 2);
assert_eq!(foo.z, 3);
```

```rust
# use fields_glob::fields_glob;
#
# #[derive(fields_glob, Debug, Default)]
# struct Foo {
#     x: i32,
#     y: i32,
#     z: i32,
# }

let foo = Foo { x: 1, y: 2, z: 3 };
let Foo! { x: n, * } = foo; // pattern destructure
assert_eq!(n, 1);
assert_eq!(y, 2);
assert_eq!(z, 3);
```
