use crate::bindings::{
    address_space, address_space_operations, current_time, dentry, dev_t, file_operations,
    get_next_ino, gfp_t, inc_nlink, init_special_inode, inode, inode_init_owner, inode_nohighmem,
    inode_operations, kfree, super_block, super_operations, umode_t,
};
use crate::c_fns::rs_new_inode;

extern "C" {
    fn _mapping_set_gfp_mask(m: *mut address_space, mask: gfp_t);
    fn _mapping_set_unevictable(m: *mut address_space);
}

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

pub const DEFAULT_ADDRESS_SPACE_OPERATIONS: address_space_operations = address_space_operations {
    readpage: None,
    write_begin: None,
    write_end: None,
    set_page_dirty: None,
    writepage: None,
    writepages: None,
    readpages: None,
    bmap: None,
    invalidatepage: None,
    releasepage: None,
    freepage: None,
    direct_IO: None,
    migratepage: None,
    isolate_page: None,
    putback_page: None,
    launder_page: None,
    is_partially_uptodate: None,
    is_dirty_writeback: None,
    error_remove_page: None,
    swap_activate: None,
    swap_deactivate: None,
};

pub const DEFAULT_INODE_OPERATIONS: inode_operations = inode_operations {
    create: None,
    lookup: None,
    link: None,
    unlink: None,
    symlink: None,
    mkdir: None,
    rmdir: None,
    mknod: None,
    rename: None,
    listxattr: None,
    fiemap: None,
    update_time: None,
    tmpfile: None,
    set_acl: None,
    get_link: None,
    permission: None,
    get_acl: None,
    readlink: None,
    setattr: None,
    getattr: None,
    atomic_open: None,
};

pub const DEFAULT_FILE_OPERATIONS: file_operations = file_operations {
    read_iter: None,
    write_iter: None,
    mmap: None,
    fsync: None,
    splice_read: None,
    splice_write: None,
    llseek: None,
    get_unmapped_area: None,
    read: None,
    write: None,
    iterate: None,
    iterate_shared: None,
    poll: None,
    unlocked_ioctl: None,
    compat_ioctl: None,
    open: None,
    flush: None,
    release: None,
    fasync: None,
    lock: None,
    sendpage: None,
    check_flags: None,
    flock: None,
    setlease: None,
    fallocate: None,
    show_fdinfo: None,
    copy_file_range: None,
    clone_file_range: None,
    dedupe_file_range: None,
    fadvise: None,
    mmap_supported_flags: 0,
    owner: core::ptr::null_mut(),
};

#[derive(Copy, Clone)]
pub struct Inode {
    ptr: *mut inode,
}

impl Inode {
    pub fn new(sb: SuperBlock) -> Option<Self> {
        Self::from_ptr(rs_new_inode(sb))
    }

    pub fn null() -> Self {
        Self {
            ptr: core::ptr::null_mut(),
        }
    }

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
    pub fn get_sb(self) -> SuperBlock {
        SuperBlock::from_ptr_unchecked(unsafe { (*self.ptr).i_sb })
    }
    pub fn set_amctime_current(self) {
        unsafe { (*self.ptr).i_atime = current_time(self.ptr) };
        unsafe { (*self.ptr).i_mtime = (*self.ptr).i_atime };
        unsafe { (*self.ptr).i_ctime = (*self.ptr).i_mtime };
    }
    pub fn set_mctime_current(self) {
        unsafe { (*self.ptr).i_mtime = current_time(self.ptr) };
        unsafe { (*self.ptr).i_ctime = (*self.ptr).i_mtime };
    }

    pub fn set_ino(&self) {
        unsafe { (*self.ptr).i_ino = get_next_ino().into() }
    }

    pub fn init_owner(&self, dir: Inode, mode: umode_t) {
        unsafe { inode_init_owner(self.get_ptr(), dir.get_ptr(), mode) };
    }

    pub fn set_aops(&self, aops: &address_space_operations) {
        unsafe { (*(*self.ptr).i_mapping).a_ops = aops }
    }

    pub fn mapping_set_gfp_mask(&self, mask: gfp_t) {
        unsafe { _mapping_set_gfp_mask((*self.ptr).i_mapping, mask) }
    }

    pub fn mapping_set_unevictable(&self) {
        unsafe { _mapping_set_unevictable((*self.ptr).i_mapping) }
    }

    pub fn set_inode_operations(&self, iop: &inode_operations) {
        unsafe { (*self.ptr).i_op = iop }
    }

    pub fn set_file_operations(&self, fop: &file_operations) {
        unsafe { (*self.ptr).i_fop = fop }
    }

    pub fn inc_nlink(&self) {
        unsafe { inc_nlink(self.ptr) }
    }

    pub fn nohighmem(&self) {
        unsafe { inode_nohighmem(self.ptr) }
    }

    pub fn init_special_inode(&self, mode: umode_t, dev: dev_t) {
        unsafe { init_special_inode(self.ptr, mode, dev) };
    }
}

#[derive(Clone, Copy)]
pub struct SuperBlock {
    pub ptr: *mut super_block,
}

impl SuperBlock {
    pub fn from_ptr_unchecked(sb: *mut super_block) -> Self {
        Self { ptr: sb }
    }

    pub fn from_ptr(sb: *mut super_block) -> Option<Self> {
        if sb == core::ptr::null_mut() {
            None
        } else {
            Some(Self { ptr: sb })
        }
    }

    pub fn get_ptr(&self) -> *mut super_block {
        self.ptr
    }

    pub fn set_fs_info(&self, fsi: &mut crate::RamfsFsInfo) {
        unsafe { (*self.ptr).s_fs_info = fsi as *mut _ as *mut cty::c_void };
    }

    pub fn free_fs_info(&self) {
        if unsafe { (*self.ptr).s_fs_info } != core::ptr::null_mut() {
            unsafe { kfree((*self.ptr).s_fs_info) }
        }
    }

    pub fn set_root(&self, root: *mut dentry) {
        unsafe { (*self.ptr).s_root = root };
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

pub struct Dentry {
    ptr: *mut dentry,
}

impl Dentry {
    pub fn from_ptr(ptr: *mut dentry) -> Self {
        Self { ptr }
    }

    pub fn get_ptr(&self) -> *mut dentry {
        self.ptr
    }

    pub fn get_name(&self) -> &str {
        unsafe {
            cstr_core::CStr::from_ptr((*self.ptr).d_name.name as *const cty::c_char)
                .to_str()
                .unwrap()
        }
    }

    pub fn get_sb(&self) -> SuperBlock {
        SuperBlock::from_ptr_unchecked(unsafe { (*self.ptr).d_sb })
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RamfsMountOpts {
    pub mode: umode_t,
    pub debug: bool,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RamfsFsInfo {
    pub mount_opts: RamfsMountOpts,
}

pub trait RamfsSuperBlockOps {
    fn is_in_debug_mode(&self) -> bool;
    fn get_fs_info(&self) -> RamfsFsInfo;
}

pub trait RamfsInodeOps {
    fn is_in_debug_mode(&self) -> bool;
}

impl RamfsSuperBlockOps for SuperBlock {
    fn is_in_debug_mode(&self) -> bool {
        unsafe {
            (*((*self.ptr).s_fs_info as *mut RamfsFsInfo))
                .mount_opts
                .debug
        }
    }

    fn get_fs_info(&self) -> RamfsFsInfo {
        unsafe { *((*self.ptr).s_fs_info as *mut RamfsFsInfo) }
    }
}

impl RamfsInodeOps for Inode {
    fn is_in_debug_mode(&self) -> bool {
        self.get_sb().is_in_debug_mode()
    }
}
