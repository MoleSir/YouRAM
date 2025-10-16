use std::{borrow::Borrow, collections::HashMap, hash::{Hash, Hasher}, sync::{Arc, LazyLock, RwLock}};

#[derive(Debug, Clone)]
pub struct ShrString(Arc<String>);

#[derive(Default)]
pub struct StringPool {
    map: HashMap<String, Arc<String>>,
}

static POOL: LazyLock<RwLock<StringPool>> = LazyLock::new(|| {
    RwLock::new(StringPool::default())
});

impl ShrString {
    pub fn new<'a, S: Into<&'a str>>(s: S) -> Self {
        Self::new_str(s)
    }

    pub fn new_str<'a, S: Into<&'a str>>(s: S) -> Self {
        let s = s.into();
        let map = &mut POOL.write().unwrap().map;

        if let Some(existing) = map.get(s) {
            return ShrString(existing.clone());
        }

        let arc = Arc::new(s.to_owned());
        map.insert(arc.to_string(), arc.clone());
        Self(arc)
    }

    pub fn new_string<S: Into<String>>(s: S) -> Self {
        let s = s.into();
        let map = &mut POOL.write().unwrap().map;

        if let Some(existing) = map.get(&s) {
            return ShrString(existing.clone());
        }

        let arc = Arc::new(s);
        map.insert(arc.to_string(), arc.clone());
        Self(arc)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ShrString {
    fn default() -> Self {
        Self::new("")
    }
}

impl std::ops::Deref for ShrString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for ShrString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl PartialEq for ShrString {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for ShrString {}

impl Hash for ShrString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl PartialEq<&str> for ShrString {
    fn eq(&self, other: &&str) -> bool {
        self.0.as_str() == *other
    }
}
impl PartialEq<ShrString> for &str {
    fn eq(&self, other: &ShrString) -> bool {
        *self == other.0.as_str()
    }
}

impl PartialEq<String> for ShrString {
    fn eq(&self, other: &String) -> bool {
        self.0.as_str() == other.as_str()
    }
}
impl PartialEq<ShrString> for String {
    fn eq(&self, other: &ShrString) -> bool {
        self.as_str() == other.0.as_str()
    }
}

impl From<&str> for ShrString {
    fn from(s: &str) -> Self {
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