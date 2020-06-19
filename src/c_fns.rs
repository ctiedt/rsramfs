use crate::bindings::{
    d_instantiate, dentry, dev_t, file_system_type, inc_nlink, inode, mount_nodev, page_symlink,
    super_block, umode_t,
};

extern "C" {
    fn ramfs_get_inode(
        sb: *mut super_block,
        dir: *const inode,
        mode: umode_t,
        dev: dev_t,
    ) -> *mut inode;
    fn c_dget(dentry: *mut dentry) -> *mut dentry;
}

pub fn rs_ramfs_get_inode(
    sb: *mut super_block,
    dir: *const inode,
    mode: umode_t,
    dev: dev_t,
) -> Option<*mut inode> {
    let inode = unsafe { ramfs_get_inode(sb, dir, mode, dev) };
    if inode == core::ptr::null_mut() {
        return None;
    } else {
        return Some(inode);
    }
}

pub fn rs_d_instantiate(dentry: *mut dentry, inode: *mut inode) {
    unsafe { d_instantiate(dentry, inode) }
}

pub fn rs_dget(dentry: *mut dentry) -> *mut dentry {
    unsafe { c_dget(dentry) }
}

pub fn rs_inc_nlink(inode: *mut inode) {
    unsafe { inc_nlink(inode) }
}

pub fn rs_mount_nodev(
    fs_type: *mut file_system_type,
    flags: cty::c_int,
    data: *mut cty::c_void,
    fill_super: ::core::option::Option<
        unsafe extern "C" fn(
            arg1: *mut super_block,
            arg2: *mut cty::c_void,
            arg3: cty::c_int,
        ) -> cty::c_int,
    >,
) -> *mut dentry {
    unsafe { mount_nodev(fs_type, flags, data, fill_super) }
}

pub fn rs_page_symlink(
    inode: *mut inode,
    symname: *const cty::c_char,
    len: cty::c_int,
) -> Result<(), cty::c_int> {
    match unsafe { page_symlink(inode, symname, len) } {
        0 => Ok(()),
        v => Err(v),
    }
}
