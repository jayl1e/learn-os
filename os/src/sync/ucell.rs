use core::cell::{RefCell, RefMut};

pub struct UCell<T> {
    inner: RefCell<T>,
}

unsafe impl<T> Sync for UCell<T> {}

impl<T> UCell<T> {
    pub unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }
    pub fn exclusive_access(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}
