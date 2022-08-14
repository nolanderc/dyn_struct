use dyn_struct::DynStruct;

#[test]
fn custom() {
    #[repr(C)]
    #[derive(Debug, DynStruct)]
    struct Foo {
        pub inner: u32,
        pub values: [u32],
    }

    let foo = Foo::new(14, vec![1, 2, 3, 4]);
    assert_eq!(foo.inner, 14);
    assert_eq!(&foo.values, [1, 2, 3, 4]);
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

    let foo = Foo::new(true, "hello", [1, 2, 3, 4]);
    assert_eq!(foo.inner, true);
    assert_eq!(foo.text, "hello");
    assert_eq!(&foo.values, [1, 2, 3, 4]);
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

    let foo: Box<MyDynamicType> = MyDynamicType::new(true, 123, 4..8);
    assert_eq!(foo.awesome, true);
    assert_eq!(foo.number, 123);
    assert_eq!(&foo.dynamic, &[4, 5, 6, 7]);
}

#[test]
fn non_copy_with_drop() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

    struct Droppable;
    impl Drop for Droppable {
        fn drop(&mut self) {
            eprintln!("drop");
            DROP_COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[repr(C)]
    #[derive(DynStruct)]
    struct MyDynamicType {
        pub boxed: Droppable,
        pub dynamic: [u32],
    }

    let foo: Box<MyDynamicType> = MyDynamicType::new(Droppable, 1..8);
    assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 0, "creating DynStruct should not result in drop");

    assert_eq!(&foo.dynamic, &[1, 2, 3, 4, 5, 6, 7]);
    drop(foo);

    assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 1, "dropping DynStruct should result in drop");
}
