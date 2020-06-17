#![feature(lang_items, const_fn, box_into_raw_non_null)]
#![no_std]
#![feature(allocator_api)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![allow(improper_ctypes)]
extern crate alloc;

#[macro_use]
mod io;
mod mem;

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
mod bindings;

use bindings::{
    __set_page_dirty_nobuffers, __this_module, address_space, address_space_operations, dentry,
    dev_t, file, file_operations, generic_file_llseek, generic_file_mmap, generic_file_read_iter,
    generic_file_splice_read, generic_file_write_iter, gfp_t, inode, inode_nohighmem,
    inode_operations, iter_file_splice_write, module, noop_fsync, page_symlink_inode_operations,
    simple_dir_operations, simple_getattr, simple_link, simple_lookup, simple_readpage,
    simple_rename, simple_rmdir, simple_setattr, simple_unlink, simple_write_begin,
    simple_write_end, super_block, super_operations, umode_t,
};

extern "C" {
    fn c_dget(dentry: *mut dentry) -> *mut dentry;
    fn _mapping_set_gfp_mask(m: *mut address_space, mask: gfp_t);
    fn _mapping_set_unevictable(m: *mut address_space);
    fn ramfs_mmu_get_unmapped_area(
        file: *mut file,
        addr: cty::c_ulong,
        len: cty::c_ulong,
        pgoff: cty::c_ulong,
        flags: cty::c_ulong,
    ) -> cty::c_ulong;
}

#[cfg(not(test))]
#[lang = "eh_personality"]
fn rust_eh_personality() {}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[global_allocator]
static A: mem::KernelAllocator = mem::KernelAllocator {};

const ramfs_aops: address_space_operations = address_space_operations {
    readpage: Some(simple_readpage),
    write_begin: Some(simple_write_begin),
    write_end: Some(simple_write_end),
    set_page_dirty: Some(__set_page_dirty_nobuffers),
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

const ramfs_dir_inode_ops: inode_operations = inode_operations {
    create: Some(ramfs_create),
    lookup: Some(simple_lookup),
    link: Some(simple_link),
    unlink: Some(simple_unlink),
    symlink: Some(ramfs_symlink),
    mkdir: Some(ramfs_mkdir),
    rmdir: Some(simple_rmdir),
    mknod: Some(ramfs_mknod),
    rename: Some(simple_rename),
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

static mut ramfs_file_inode_ops: inode_operations = inode_operations {
    setattr: Some(simple_setattr),
    getattr: Some(simple_getattr),
    atomic_open: None,
    create: None,
    lookup: None,
    get_link: None,
    permission: None,
    get_acl: None,
    readlink: None,
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
};

static mut ramfs_file_ops: file_operations = file_operations {
    read_iter: Some(generic_file_read_iter),
    write_iter: Some(generic_file_write_iter),
    mmap: Some(generic_file_mmap),
    fsync: Some(noop_fsync),
    splice_read: Some(generic_file_splice_read),
    splice_write: Some(iter_file_splice_write),
    llseek: Some(generic_file_llseek),
    get_unmapped_area: Some(ramfs_mmu_get_unmapped_area),
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

#[no_mangle]
pub extern "C" fn ramfs_get_inode(
    sb: *mut super_block,
    dir: *const inode,
    mode: umode_t,
    dev: dev_t,
) -> *mut inode {
    use bindings::{
        current_time, get_next_ino, inc_nlink, init_special_inode, inode_init_owner, new_inode,
        S_IFDIR, S_IFLNK, S_IFMT, S_IFREG,
    };

    const GFP_HIGHUSER: u32 = 6422722;
    let inode: *mut inode = unsafe { new_inode(sb) };

    if inode != core::ptr::null_mut() {
        unsafe { (*inode).i_ino = get_next_ino().into() };
        unsafe { inode_init_owner(inode, dir, mode) };
        unsafe { (*(*inode).i_mapping).a_ops = &ramfs_aops };
        unsafe { _mapping_set_gfp_mask((*inode).i_mapping, GFP_HIGHUSER) };
        unsafe { _mapping_set_unevictable((*inode).i_mapping) };
        unsafe { (*inode).i_atime = current_time(inode) };
        unsafe { (*inode).i_mtime = (*inode).i_atime };
        unsafe { (*inode).i_ctime = (*inode).i_atime };
        let _mode = u32::from(mode) & S_IFMT;
        match _mode {
            _ if _mode == S_IFREG => {
                unsafe { (*inode).i_op = &ramfs_file_inode_ops };
                unsafe { (*inode).i_fop = &ramfs_file_ops };
            }
            _ if _mode == S_IFDIR => {
                unsafe { (*inode).i_op = &ramfs_dir_inode_ops };
                unsafe { (*inode).i_fop = &simple_dir_operations };
                unsafe { inc_nlink(inode) };
            }
            _ if _mode == S_IFLNK => {
                unsafe { (*inode).i_op = &page_symlink_inode_operations };
                unsafe { inode_nohighmem(inode) };
            }
            _ => {
                unsafe { init_special_inode(inode, mode, dev) };
            }
        }
    }

    inode
}

#[no_mangle]
pub extern "C" fn ramfs_mknod(
    dir: *mut inode,
    dentry: *mut dentry,
    mode: umode_t,
    dev: dev_t,
) -> cty::c_int {
    use bindings::{current_time, d_instantiate, ENOSPC};

    let inode = unsafe { ramfs_get_inode((*dir).i_sb, dir, mode, dev) };
    let mut error = -(ENOSPC as i32);

    if inode != core::ptr::null_mut() {
        unsafe { d_instantiate(dentry, inode) };
        unsafe { c_dget(dentry) };
        error = 0;
        unsafe { (*dir).i_mtime = current_time(dir) };
        unsafe { (*dir).i_ctime = (*dir).i_mtime };
    }

    error
}

#[no_mangle]
pub extern "C" fn ramfs_mkdir(dir: *mut inode, dentry: *mut dentry, mode: umode_t) -> cty::c_int {
    use bindings::{inc_nlink, S_IFDIR};
    let retval = ramfs_mknod(dir, dentry, mode | (S_IFDIR as u16), 0);
    if retval == 0 {
        unsafe { inc_nlink(dir) };
    }
    retval
}

#[no_mangle]
pub extern "C" fn ramfs_create(
    dir: *mut inode,
    dentry: *mut dentry,
    mode: umode_t,
    _excl: bool,
) -> cty::c_int {
    use bindings::S_IFREG;
    ramfs_mknod(dir, dentry, mode | (S_IFREG as u16), 0)
}

#[no_mangle]
pub extern "C" fn ramfs_symlink(
    dir: *mut inode,
    dentry: *mut dentry,
    symname: *const cty::c_char,
) -> cty::c_int {
    use bindings::{current_time, d_instantiate, iput, page_symlink, ENOSPC, S_IFLNK, S_IRWXUGO};
    let mut error = -(ENOSPC as cty::c_int);
    let name = unsafe { cstr_core::CStr::from_ptr(symname) };

    let inode =
        unsafe { ramfs_get_inode((*dir).i_sb, dir, (S_IFLNK as u16) | (S_IRWXUGO as u16), 0) };
    if inode != core::ptr::null_mut() {
        let len = name.to_str().unwrap().len() + 1;
        error = unsafe { page_symlink(inode, symname, len as cty::c_int) };
        if error == 0 {
            unsafe { d_instantiate(dentry, inode) };
            unsafe { c_dget(dentry) };
            unsafe { (*dir).i_mtime = current_time(dir) };
            unsafe { (*dir).i_ctime = (*dir).i_mtime };
        } else {
            unsafe { iput(inode) };
        }
    }

    error
}

#[no_mangle]
pub extern "C" fn ramfs_kill_super(sb: *mut bindings::super_block) {
    use bindings::{kfree, kill_litter_super};
    unsafe { kfree((*sb).s_fs_info) };
    unsafe { kill_litter_super(sb) };
}

//TODO: Remove this
#[no_mangle]
pub extern "C" fn rust_main() -> i64 {
    println!("Hello, fairies and unicorns!");
    0
}
