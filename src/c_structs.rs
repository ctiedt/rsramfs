use crate::bindings::{current_time, inode};

pub trait InodeOperations {
    fn set_mctime_current(&mut self);
}

impl InodeOperations for inode {
    fn set_mctime_current(&mut self) {
        unsafe { self.i_mtime = current_time(self) };
        self.i_ctime = self.i_mtime;
    }
}
