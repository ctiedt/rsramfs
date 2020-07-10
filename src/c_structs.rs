use crate::bindings::{current_time, inode, super_block, super_operations};

pub const DEFAULT_SUPER_OPS: super_operations = super_operations {
    statfs: None,
    drop_inode: None,
    show_options: None,
    alloc_inode: None,
    destroy_inode: None,
    dirty_inode: None,
    write_inode: None,
    evict_inode: None,
    put_super: None,
    sync_fs: None,
    freeze_super: None,
    freeze_fs: None,
    thaw_super: None,
    unfreeze_fs: None,
    remount_fs: None,
    umount_begin: None,
    show_devname: None,
    show_path: None,
    show_stats: None,
    quota_read: None,
    quota_write: None,
    get_dquots: None,
    bdev_try_to_free_page: None,
    nr_cached_objects: None,
    free_cached_objects: None,
};

#[derive(Copy, Clone)]
pub struct Inode {
    ptr: *mut inode,
}

impl Inode {
    pub fn from_ptr(inode: *mut inode) -> Option<Self> {
        if inode == core::ptr::null_mut() {
            None
        } else {
            Some(Self { ptr: inode })
        }
    }

    pub fn from_ptr_unchecked(inode: *mut inode) -> Self {
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

    pub fn null() -> Self {
        Self {
            ptr: core::ptr::null_mut(),
        }
    }  
}

#[derive(Clone, Copy)]
pub struct SuperBlock {
    pub ptr: *mut super_block,
}

impl SuperBlock {
    pub fn from_ptr(sb: *mut super_block) -> Self {
        Self { ptr: sb }
    }

    pub fn set_fs_info(&self, fsi: &mut crate::RamfsFsInfo) {
        unsafe { (*self.ptr).s_fs_info = fsi as *mut _ as *mut cty::c_void };
    }

    pub fn set_fields(
        &self,
        maxbytes: cty::c_longlong,
        blocksize_bits: cty::c_uchar,
        magic: cty::c_ulonglong,
        op: *const super_operations,
        time_gran: cty::c_uint,
    ) {
        unsafe { (*self.ptr).s_maxbytes = maxbytes };
        unsafe { (*self.ptr).s_blocksize_bits = blocksize_bits };
        unsafe { (*self.ptr).s_magic = magic };
        unsafe { (*self.ptr).s_op = op };
        unsafe { (*self.ptr).s_time_gran = time_gran };
    }
}
