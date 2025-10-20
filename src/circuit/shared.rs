
use std::{hash::{Hash, Hasher}, ptr, sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard}};

#[derive(Debug)]
pub struct Shr<T: ?Sized> {
    inner: Arc<RwLock<T>>,
}

impl<T: ?Sized> Shr<T> {
    pub fn from_inner(inner: Arc<RwLock<T>>) -> Self {
        Self { inner }
    }
}

impl<T> Shr<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(value)),
        }
    }

    pub fn inner(self) -> Arc<RwLock<T>> {
        self.inner
    }
}

impl<T: ?Sized> Shr<T> {
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        self.inner.read().unwrap()
    }

    pub fn wrire(&self) -> RwLockWriteGuard<'_, T> {
        self.inner.write().unwrap()
    }
}

impl<T: ?Sized> Clone for Shr<T> {
    fn clone(&self) -> Self {
        Shr {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T: ?Sized> PartialEq for Shr<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl<T: ?Sized> Eq for Shr<T> {}

impl<T: ?Sized> Hash for Shr<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::hash(Arc::as_ptr(&self.inner), state);
    }
}