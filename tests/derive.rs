
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

#[test]
fn readme() {
    #[repr(C)]
    #[derive(DynStruct)]
    struct MyDynamicType {
        pub awesome: bool,
        pub number: u32,
        pub dynamic: [u32],
    }

    let foo: Box<MyDynamicType> = MyDynamicType::new(true, 123, &[4, 5, 6, 7]);
    assert_eq!(foo.awesome, true);
    assert_eq!(foo.number, 123);
    assert_eq!(&foo.dynamic, &[4, 5, 6, 7]);
}
