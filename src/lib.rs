use std::borrow::Borrow;
use std::hash::Hash;

#[derive(Debug)]
pub struct Str {
    data: *const u8,
    size: usize,
}

impl Str {
    #[inline(always)]
    fn layout(size: usize) -> std::alloc::Layout {
        match std::alloc::Layout::array::<u8>(size) {
            Ok(value) => value,
            Err(err) => {
                panic!("Failed to create Str layout for size {}: {}", size, err)
            }
        }
    }

    #[inline]
    pub fn new(s: &str) -> Self {
        let size = s.len();
        unsafe {
            let data = std::alloc::alloc(Str::layout(size));
            std::ptr::copy(s.as_ptr(), data, size);
            Self { data, size }
        }
    }
}

impl Drop for Str {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            let data = self.data as *mut u8;
            std::alloc::dealloc(data, Str::layout(self.size));
        }
    }
}

impl Clone for Str {
    #[inline]
    fn clone(&self) -> Self {
        unsafe {
            let size = self.size;
            let data = std::alloc::alloc(Str::layout(size));
            std::ptr::copy(self.data, data, size);
            Self { data, size }
        }
    }
}

impl AsRef<str> for Str {
    fn as_ref(&self) -> &str {
        self.borrow()
    }
}

impl Borrow<str> for Str {
    fn borrow(&self) -> &str {
        unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                self.data, self.size,
            ))
        }
    }
}

impl PartialEq<Str> for Str {
    #[inline]
    fn eq(&self, other: &Str) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Eq for Str {}

impl Hash for Str {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    #[derive(Debug, Clone, Hash, PartialEq, Eq)]
    struct SchemaKey {
        subject: Str,
        version: i32,
    }

    #[derive(Debug, Clone, Hash, PartialEq, Eq)]
    struct SchemaKeyRef<'a> {
        subject: &'a str,
        version: i32,
    }

    impl<'a> Borrow<SchemaKeyRef<'a>> for SchemaKey {
        fn borrow(&self) -> &SchemaKeyRef<'a> {
            unsafe { &*(self as *const SchemaKey as *const SchemaKeyRef) }
        }
    }

    #[test]
    fn test_size_equals_to_str() {
        assert_eq!(std::mem::size_of::<Str>(), std::mem::size_of::<&str>());
    }

    #[test]
    fn test_new_and_as_ref() {
        let s = Str::new("hello world");
        assert_eq!(s.as_ref(), "hello world");
        assert_eq!(s.size, 11);
    }

    #[test]
    fn test_empty_string() {
        let s = Str::new("");
        assert_eq!(s.as_ref(), "");
        assert_eq!(s.size, 0);
    }

    #[test]
    fn test_clone() {
        let s1 = Str::new("test string");
        let s2 = s1.clone();

        assert_eq!(s1.as_ref(), s2.as_ref());
        assert_eq!(s1.size, s2.size);
        assert_ne!(s1.data, s2.data);
    }

    #[test]
    fn test_equality() {
        let s1 = Str::new("hello");
        let s2 = Str::new("hello");
        let s3 = Str::new("world");

        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
    }

    #[test]
    fn test_hash() {
        let mut set = HashSet::new();

        let s1 = Str::new("hello");
        let s2 = Str::new("hello");
        let s3 = Str::new("world");

        set.insert(s1);
        assert!(!set.insert(s2));
        assert!(set.insert(s3));
        assert_eq!(set.len(), 2);
        assert!(set.contains("hello"));
        assert!(set.contains("world"));
    }

    #[test]
    fn test_unicode() {
        let s = Str::new("Hello, 世界!");
        assert_eq!(s.as_ref(), "Hello, 世界!");
    }

    #[test]
    fn test_drop() {
        {
            let _s = Str::new("test drop");
        }
    }

    #[test]
    fn test_large_string() {
        let size = 1000000;
        let large = "a".repeat(size);
        let s = Str::new(&large);
        assert_eq!(s.as_ref(), large);
        assert_eq!(s.size, size);
    }

    #[test]
    fn test_multiple_clones() {
        let s1 = Str::new("test");
        let s2 = s1.clone();
        let s3 = s2.clone();
        let s4 = s3.clone();

        assert_eq!(s1.as_ref(), s4.as_ref());
        assert_ne!(s1.data, s2.data);
        assert_ne!(s2.data, s3.data);
        assert_ne!(s3.data, s4.data);
    }

    #[test]
    fn test_complex_hashmap_key() {
        let mut cache: HashMap<SchemaKey, String> = HashMap::new();
        cache.insert(
            SchemaKey {
                subject: Str::new("User"),
                version: 1,
            },
            "User:1".to_string(),
        );
        cache.insert(
            SchemaKey {
                subject: Str::new("User"),
                version: 2,
            },
            "User:2".to_string(),
        );

        assert_eq!(
            cache.get(&SchemaKeyRef {
                subject: "User",
                version: 1
            }),
            Some(&"User:1".to_string()),
        );
        assert_eq!(
            cache.get(&SchemaKeyRef {
                subject: "User",
                version: 2
            }),
            Some(&"User:2".to_string()),
        );
    }
}
