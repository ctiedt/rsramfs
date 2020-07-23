use core::alloc::{GlobalAlloc, Layout};
use crate::bindings::kfree;

extern "C" {
    fn c_alloc(size: usize) -> *mut u8;
}

pub struct KernelAllocator;

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        c_alloc(layout.size())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        kfree(ptr as *mut _ as *mut cty::c_void);
    }
}

#[cfg(not(test))]
#[alloc_error_handler]
pub fn alloc_error_handler(_: Layout) -> ! {
    loop {}
}
