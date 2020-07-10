use crate::bindings::super_block;
use crate::bindings::{current_time, inode};
use core::ptr;

#[derive(Copy, Clone)]
pub struct Inode {
    ptr: *mut inode,
}

impl Inode{
    pub fn from_ptr(inode: *mut inode) -> Self {
        Self { ptr: inode }
    }

    pub fn get_ptr(self) -> *mut inode {
        self.ptr
    }

    pub fn get_sb(self) -> *mut super_block {
        unsafe { (*self.ptr).i_sb }
    }

    pub fn set_mctime_current(self) {
        unsafe { (*self.ptr).i_mtime = current_time(self.ptr) };
        unsafe { (*self.ptr).i_ctime = (*self.ptr).i_mtime };
    }

    pub fn inode_null(self) -> *mut inode{
        ptr::null_mut()        
    }
}
