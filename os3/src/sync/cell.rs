use core::cell::{RefCell, RefMut};

pub struct SyncRefCell<T>(RefCell<T>);

unsafe impl<T> Sync for SyncRefCell<T> {}

impl<T> SyncRefCell<T> {
    pub unsafe fn new(value: T) -> Self {
        Self(RefCell::new(value))
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.0.borrow_mut()
    }
}
