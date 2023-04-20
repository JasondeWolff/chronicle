pub use std::sync::Arc;
pub use std::sync::{Mutex, MutexGuard};

pub struct ArcMutex<T> {
    value: Arc<Mutex<T>>
}

impl<T> ArcMutex<T> {
    pub fn new(value: T) -> Self {
        ArcMutex {
            value: Arc::new(Mutex::new(value))
        }
    }

    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.value)
    }

    pub fn as_ref(&self) -> MutexGuard<'_, T> {
        self.value.as_ref().lock().unwrap()
    }

    pub fn as_mut(&self) -> MutexGuard<'_, T> {
        self.value.as_ref().lock().unwrap()
    }

    pub fn clone(&self) -> ArcMutex<T> {
        ArcMutex {
            value: self.value.clone()
        }
    }
}