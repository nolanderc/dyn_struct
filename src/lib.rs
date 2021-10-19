#[cfg(feature = "derive")]
pub use dyn_struct_derive::DynStruct;

#[repr(C)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DynStruct<T, D> {
    pub single: T,
    pub many: [D],
}

impl<T, D> DynStruct<T, D> {
    pub fn new(single: T, many: &[D]) -> Box<Self>
    where
        T: Copy,
        D: Copy,
    {
        use std::mem::{align_of, size_of};

        let total_size = size_of::<T>() + size_of::<D>() * many.len();

        if total_size == 0 {
            // Create a fat pointer to a slice of `many.len()` elements, then cast the slice into a
            // fat pointer to `Self`. This essentially creates the fat pointer to `Self` of
            // `many.len()` we need.
            let slice: Box<[()]> = Box::from(slice_with_len(many.len()));
            let ptr = Box::into_raw(slice) as *mut [()] as *mut Self;
            unsafe { Box::from_raw(ptr) }
        } else {
            let align = usize::max(align_of::<T>(), align_of::<D>());
            let layout = std::alloc::Layout::from_size_align(total_size, align).unwrap();

            unsafe {
                let raw = std::alloc::alloc(layout);
                if raw.is_null() {
                    std::alloc::handle_alloc_error(layout)
                }

                Self::single_ptr(raw).copy_from_nonoverlapping(&single as *const T, 1);
                Self::many_ptr(raw).copy_from_nonoverlapping(many.as_ptr(), many.len());

                let slice = std::slice::from_raw_parts_mut(raw as *mut (), many.len());
                let ptr = slice as *mut [()] as *mut Self;
                Box::from_raw(ptr)
            }
        }
    }

    fn single_ptr(raw: *mut u8) -> *mut T {
        raw as *mut T
    }

    fn many_ptr(raw: *mut u8) -> *mut D {
        unsafe {
            let naive = raw.add(std::mem::size_of::<T>());
            let align = std::mem::align_of::<D>();
            let ptr = naive.add(naive.align_offset(align));
            ptr as *mut D
        }
    }
}

impl<T> DynStruct<T, T> {
    /// Get a `DynStruct` as a view over a slice (this does not allocate).
    pub fn from_slice(values: &[T]) -> &Self {
        assert!(
            !values.is_empty(),
            "attempted to create `{}` without `single` value (`values.is_empty()`)",
            std::any::type_name::<Self>()
        );
        let slice = &values[..values.len() - 1];
        unsafe { &*(slice as *const [T] as *const Self) }
    }
}

fn slice_with_len(len: usize) -> &'static [()] {
    static ARBITRARY: [(); usize::MAX] = [(); usize::MAX];
    &ARBITRARY[..len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixed_types() {
        let mixed = DynStruct::new((true, 32u64), &[1, 2, 3, 4]);
        assert_eq!(mixed.single, (true, 32u64));
        assert_eq!(&mixed.many, &[1, 2, 3, 4]);
    }

    #[test]
    fn zero_sized_types() {
        let zero = DynStruct::new((), &[(), ()]);
        assert_eq!(zero.single, ());
        assert_eq!(&zero.many, &[(), ()]);
    }

    #[test]
    fn from_slice() {
        let same = DynStruct::<u32, u32>::from_slice(&[1, 2, 3]);
        assert_eq!(same.single, 1);
        assert_eq!(&same.many, &[2, 3]);
    }
}
