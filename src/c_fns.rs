use crate::bindings::{
    d_instantiate, d_make_root, dentry, file_system_type, inc_nlink, iput, kill_litter_super,
    mount_nodev, page_symlink, super_block,
};
use crate::c_structs::{Inode, SuperBlock};

extern "C" {
    fn c_dget(dentry: *mut dentry) -> *mut dentry;
}

pub fn rs_d_instantiate(dentry: *mut dentry, inode: Inode) {
    unsafe { d_instantiate(dentry, inode.get_ptr()) }
}

pub fn rs_dget(dentry: *mut dentry) -> *mut dentry {
    unsafe { c_dget(dentry) }
}

pub fn rs_inc_nlink(inode: Inode) {
    unsafe { inc_nlink(inode.get_ptr()) }
}

pub fn rs_iput(inode: Inode) {
    unsafe { iput(inode.get_ptr()) }
}

pub fn rs_d_make_root(inode: Inode) -> *mut dentry {
    unsafe { d_make_root(inode.get_ptr()) }
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
    inode: Inode,
    symname: *const cty::c_char,
    len: cty::c_int,
) -> Result<(), cty::c_int> {
    match unsafe { page_symlink(inode.get_ptr(), symname, len) } {
        0 => Ok(()),
        v => Err(v),
    }
}

pub fn rs_kill_litter_super(sb: SuperBlock) {
    unsafe { kill_litter_super(sb.get_ptr()) };
}
