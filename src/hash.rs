use std::ffi::{c_char, CStr};
use std::marker::PhantomData;

/// Defines a hash function which should have good anti-collision
/// properties
pub trait Hash<T: ?Sized> {
    fn hash(val: &T) -> usize;
}

/// The default hash struct implemented for basic types
pub struct DefaultHash<T: ?Sized> {
    _ignore_type: PhantomData<T>,
}

/// Default implementations
/// TODO: Make these more loosely typed

impl Hash<u8> for DefaultHash<u8> {
    fn hash(val: &u8) -> usize {
        *val as usize
    }
}

impl Hash<i8> for DefaultHash<i8> {
    fn hash(val: &i8) -> usize {
        *val as usize
    }
}

impl Hash<u16> for DefaultHash<u16> {
    fn hash(val: &u16) -> usize {
        *val as usize
    }
}

impl Hash<i16> for DefaultHash<i16> {
    fn hash(val: &i16) -> usize {
        *val as usize
    }
}

impl Hash<u32> for DefaultHash<u32> {
    fn hash(val: &u32) -> usize {
        *val as usize
    }
}

impl Hash<i32> for DefaultHash<i32> {
    fn hash(val: &i32) -> usize {
        *val as usize
    }
}

impl Hash<u64> for DefaultHash<u64> {
    fn hash(val: &u64) -> usize {
        *val as usize
    }
}

impl Hash<i64> for DefaultHash<i64> {
    fn hash(val: &i64) -> usize {
        *val as usize
    }
}

impl Hash<usize> for DefaultHash<usize> {
    fn hash(val: &usize) -> usize {
        *val
    }
}

impl Hash<isize> for DefaultHash<isize> {
    fn hash(val: &isize) -> usize {
        *val as usize
    }
}

impl Hash<f32> for DefaultHash<f32> {
    fn hash(val: &f32) -> usize {
        *val as usize
    }
}

impl Hash<f64> for DefaultHash<f64> {
    fn hash(val: &f64) -> usize {
        *val as usize
    }
}

impl Hash<bool> for DefaultHash<bool> {
    fn hash(val: &bool) -> usize {
        *val as usize
    }
}

/// The FNV1 hash function
///
/// # Arguments
///
/// `str`: The string to hash
fn fnv1<S: AsRef<str>>(str: S) -> usize {
    let mut res: u32 = 2166136261;
    str.as_ref()
        .bytes()
        .for_each(|c| res = (res.wrapping_mul(16777619)) ^ (c as u32));
    res as usize
}

impl Hash<str> for DefaultHash<str> {
    fn hash(val: &str) -> usize {
        fnv1(val)
    }
}

impl Hash<*const c_char> for DefaultHash<*const c_char> {
    fn hash(val: &*const c_char) -> usize {
        DefaultHash::<str>::hash(unsafe { CStr::from_ptr(*val) }.to_string_lossy().as_ref())
    }
}

#[cfg(test)]
mod test {
    use super::DefaultHash;
    use super::Hash;
    use crate::allocator::DefaultAllocator;
    use std::ffi::{c_char, CString};

    #[test]
    fn test_str() {
        assert_eq!(DefaultHash::hash(""), 2166136261);
        assert_eq!(DefaultHash::hash("Test"), 556965705);
        assert_eq!(
            DefaultHash::hash(&(CString::new("Test").unwrap().into_raw() as *const c_char)),
            556965705
        );
        assert_eq!(
            DefaultHash::hash("The big brown fox jumped over the lazy dog"),
            3003320415
        );
        assert_eq!(
            DefaultHash::hash(&crate::string::String::<DefaultAllocator>::from("Test")),
            556965705
        );
    }
}
