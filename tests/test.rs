use fields_glob::fields_glob;

#[derive(fields_glob, Debug, Default)]
struct Foo {
    x: i32,
    y: i32,
    z: i32,
}

#[test]
fn it_works() {
    let foo = Foo::default();
    let Foo! { x: n, * } = foo;

    assert_eq!(n, 0);
    assert_eq!(y, 0);
    assert_eq!(z, 0);
}

#[test]
fn all_ref() {
    let foo = Foo::default();
    let Foo! { x: n, ref * } = foo;

    assert_eq!(n, 0);
    assert_eq!(y, &0);
    assert_eq!(z, &0);
}

#[test]
fn all_ref_mut() {
    let mut foo = Foo::default();
    let Foo! { x: n, ref mut * } = foo;

    assert_eq!(n, 0);
    assert_eq!(y, &mut 0);
    assert_eq!(z, &mut 0);
}

#[test]
fn build() {
    let foo = Foo::default();
    let Foo! { x: n, * } = foo;
    let _foo = Foo! { *, x: n+1 };
}
