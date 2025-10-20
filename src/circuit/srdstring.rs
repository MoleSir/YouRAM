use std::{borrow::Borrow, collections::HashMap, hash::{Hash, Hasher}, sync::{Arc, LazyLock, RwLock}};

#[derive(Debug, Clone)]
pub enum ShrString {
    Static(&'static str),
    Dynamic(Arc<String>),
}

#[derive(Default)]
pub struct StringPool {
    map: HashMap<String, Arc<String>>,
}

static POOL: LazyLock<RwLock<StringPool>> = LazyLock::new(|| {
    RwLock::new(StringPool::default())
});

impl ShrString {
    pub fn new_str(s: &'static str) -> Self {
        Self::Static(s)
    }

    pub fn new_string<S: Into<String>>(s: S) -> Self {
        let s = s.into();
        let map = &mut POOL.write().unwrap().map;

        if let Some(existing) = map.get(&s) {
            return ShrString::Dynamic(existing.clone());
        }

        let arc = Arc::new(s);
        map.insert(arc.to_string(), arc.clone());
        Self::Dynamic(arc)
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Static(s) => s,
            Self::Dynamic(s) => s.as_str()
        }
    }
}

impl Default for ShrString {
    fn default() -> Self {
        Self::new_str("")
    }
}

impl std::ops::Deref for ShrString {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl std::fmt::Display for ShrString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl PartialEq for ShrString {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for ShrString {}

impl Hash for ShrString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl PartialEq<&str> for ShrString {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<ShrString> for &str {
    fn eq(&self, other: &ShrString) -> bool {
        *self == other.as_str()
    }
}

impl PartialEq<String> for ShrString {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}
impl PartialEq<ShrString> for String {
    fn eq(&self, other: &ShrString) -> bool {
        self.as_str() == other.as_str()
    }
}

impl From<&'static str> for ShrString {
    fn from(s: &'static str) -> Self {
        ShrString::new_str(s)
    }
}

impl From<String> for ShrString {
    fn from(s: String) -> Self {
        ShrString::new_string(s)
    }
}

#[macro_export]
macro_rules! format_shr {
    ($($arg:tt)*) => {{
        $crate::circuit::ShrString::new_string(format!($($arg)*))
    }};
}

impl Borrow<str> for ShrString {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_str() {
        let a = ShrString::new_str("hello");
        let b = ShrString::from("hello");
        assert_eq!(a, b);
        assert_eq!(a.as_str(), "hello");
        assert!(matches!(a, ShrString::Static(_)));
    }

    #[test]
    fn test_dynamic_string_pooling() {
        let s1 = ShrString::new_string("abc");
        let s2 = ShrString::new_string("abc");

        if let (ShrString::Dynamic(a1), ShrString::Dynamic(a2)) = (&s1, &s2) {
            assert!(Arc::ptr_eq(a1, a2), "Expected pooled Arc to be shared");
        } else {
            panic!("Expected dynamic ShrString");
        }

        let s3 = ShrString::new_string("abcd");
        if let (ShrString::Dynamic(a1), ShrString::Dynamic(a3)) = (&s1, &s3) {
            assert!(!Arc::ptr_eq(a1, a3));
        }
    }

    #[test]
    fn test_display_and_deref() {
        let s = ShrString::new_string("xyz");
        assert_eq!(s.to_string(), "xyz");
        assert_eq!(&*s, "xyz");
    }

    #[test]
    fn test_partial_eq_variants() {
        let s = ShrString::new_string("test");
        assert_eq!(s, "test");
        assert_eq!("test", s);
        assert_eq!(s, "test".to_string());
        assert_eq!("test".to_string(), s);
    }

    #[test]
    fn test_hash_and_eq() {
        use std::collections::HashSet;
        let s1 = ShrString::new_string("foo");
        let s2 = ShrString::new_string("foo");
        let s3 = ShrString::new_string("bar");

        let mut set = HashSet::new();
        set.insert(s1.clone());
        assert!(set.contains(&s2));
        assert!(!set.contains(&s3));
    }

    #[test]
    fn test_pool_is_global() {
        let before = POOL.read().unwrap().map.len();
        let _ = ShrString::new_string("pooled_test");
        let after = POOL.read().unwrap().map.len();
        assert!(after >= before, "Pool size should not decrease");
    }

    #[test]
    fn test_borrow_trait() {
        use std::collections::HashMap;
        let mut map: HashMap<ShrString, i32> = HashMap::new();
        map.insert("key1".into(), 42);
        assert_eq!(map.get("key1"), Some(&42));
    }
}

