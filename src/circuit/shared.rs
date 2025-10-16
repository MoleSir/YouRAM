
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub struct Shr<T: ?Sized> {
    inner: Arc<RwLock<T>>,
}

impl<T> Shr<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(value)),
        }
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
