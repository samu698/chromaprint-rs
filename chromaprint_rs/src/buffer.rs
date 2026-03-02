use std::slice;

use chromaprint_sys as sys;

#[repr(transparent)]
pub(crate) struct AllocSlot<T>(*mut T);

impl<T> AllocSlot<T> {
    pub(crate) fn new() -> Self {
        Self(std::ptr::null_mut())
    }

    pub(crate) fn as_ptr(&mut self) -> *mut *mut T {
        &mut self.0
    }

}

impl<T: Copy> AllocSlot<T> {
    pub(crate) unsafe fn into_box(self, len: usize) -> Option<Box<[T]>> {
        let ptr = self.0;
        if ptr.is_null() { return None; }
        unsafe {
            let slice = slice::from_raw_parts(ptr, len);
            Some(slice.into())
        }
    }
}

impl<T> Drop for AllocSlot<T> {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                sys::chromaprint_dealloc(self.0.cast());
            }
        }
    }
}
