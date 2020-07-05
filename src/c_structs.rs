use crate::bindings::super_block;
use crate::bindings::{current_time, inode};

#[derive(Copy, Clone)]
pub struct Inode {
    ptr: *mut inode,
}

pub trait InodeOperations {
    fn from_ptr(inode: *mut inode) -> Self;
    fn get_ptr(self) -> *mut inode;
    fn get_sb(self) -> *mut super_block;
    fn set_mctime_current(self);
}

impl InodeOperations for Inode {
    fn from_ptr(inode: *mut inode) -> Self {
        Self { ptr: inode }
    }

    fn get_ptr(self) -> *mut inode {
        self.ptr
    }

    fn get_sb(self) -> *mut super_block {
        unsafe { (*self.ptr).i_sb }
    }

    fn set_mctime_current(self) {
        unsafe { (*self.ptr).i_mtime = current_time(self.ptr) };
        unsafe { (*self.ptr).i_ctime = (*self.ptr).i_mtime };
    }
}
