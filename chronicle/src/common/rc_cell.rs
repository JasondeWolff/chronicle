pub use std::rc::Rc;
pub use std::cell::{RefCell, Ref, RefMut};

pub struct RcCell<T> {
    value: Rc<RefCell<T>>
}

impl<T> RcCell<T> {
    pub fn new(value: T) -> Self {
        RcCell {
            value: Rc::new(RefCell::new(value))
        }
    }

    pub fn strong_count(&self) -> usize {
        Rc::strong_count(&self.value)
    }

    pub fn as_ref(&self) -> Ref<'_, T> {
        self.value.as_ref().borrow()
    }

    pub fn as_mut(&self) -> RefMut<'_, T> {
        self.value.as_ref().borrow_mut()
    }

    pub fn as_ptr(&self) -> *const T {
        RefCell::as_ptr(&self.value)
    }

    pub fn clone(&self) -> RcCell<T> {
        RcCell {
            value: self.value.clone()
        }
    }
}