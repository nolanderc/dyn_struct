
use dyn_struct_derive::DynStruct;

#[test]
fn custom() {
    #[repr(C)]
    #[derive(Debug, DynStruct)]
    struct Foo {
        pub inner: u32,
        pub values: [u32],
    }

    let inner = 14;
    let values = &[1, 2, 3, 4];
    let foo = Foo::new(inner, values);
    assert_eq!(foo.inner, inner);
    assert_eq!(&foo.values, values);
}

#[test]
fn generic() {
    #[repr(C)]
    #[derive(Debug, DynStruct)]
    struct Foo<'a, T: Copy, U: Copy> {
        pub inner: T,
        pub text: &'a str,
        pub values: [U],
    }

    let inner = true;
    let text = "hello";
    let values = &[1, 2, 3, 4];
    let foo = Foo::new(inner, text, values);
    assert_eq!(foo.inner, inner);
    assert_eq!(foo.text, text);
    assert_eq!(&foo.values, values);
}

