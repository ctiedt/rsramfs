use crate::bindings::{current_time, inode};

pub trait InodeOperations {
    fn set_mctime_current(self);
}

impl InodeOperations for inode {
    fn set_mctime_current(self) {
        unsafe { self.i_mtime = current_time(&mut self) };
        unsafe { self.i_ctime = self.i_mtime };
    }
}
