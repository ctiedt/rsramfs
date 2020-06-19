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
    inode_operations, iter_file_splice_write, module, noop_fsync, page_symlink, page_symlink_inode_operations,
    simple_dir_operations, simple_getattr, simple_link, simple_lookup, simple_readpage,
    simple_rename, simple_rmdir, simple_setattr, simple_unlink, simple_write_begin,
    simple_write_end, super_block, super_operations, umode_t,
};

extern "C" {
    fn ramfs_get_inode(
        sb: *mut super_block,
        dir: *const inode,
        mode: umode_t,
        dev: dev_t,
    ) -> *mut inode;
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

fn rs_ramfs_get_inode(
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

#[no_mangle]
pub extern "C" fn ramfs_mknod(
    dir: *mut inode,
    dentry: *mut dentry,
    mode: umode_t,
    dev: dev_t,
) -> Result<(), cty::c_int> {
    use bindings::{current_time, d_instantiate, ENOSPC};

    let sb = unsafe { (*dir).i_sb };

    match rs_ramfs_get_inode(sb, dir, mode, dev) {
        Some(inode) => {
            unsafe { d_instantiate(dentry, inode) };
            unsafe { c_dget(dentry) };
            unsafe { (*dir).i_mtime = current_time(dir) };
            unsafe { (*dir).i_ctime = (*dir).i_mtime };
            Ok(())
        },
        None => Err(-(ENOSPC as i32))
    }
}

#[no_mangle]
pub extern "C" fn ramfs_mkdir(
    dir: *mut inode, 
    dentry: *mut dentry, 
    mode: umode_t) 
    -> cty::c_int {
    use bindings::{inc_nlink, S_IFDIR};
    match ramfs_mknod(dir, dentry, mode | (S_IFDIR as u16), 0){
        Ok(_) =>{
            unsafe { inc_nlink(dir)};
            0
        },
        Err(e) => e
    }
}

#[no_mangle]
pub extern "C" fn ramfs_create(
    dir: *mut inode,
    dentry: *mut dentry,
    mode: umode_t,
    _excl: bool,
) -> cty::c_int {
    use bindings::S_IFREG;
    match ramfs_mknod(dir, dentry, mode | (S_IFREG as u16), 0){
        Ok(_) => 0,
        Err(e) => e
    }
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

#[no_mangle]
pub extern "C" fn ramfs_symlink(
    dir: *mut inode,
    dentry: *mut dentry,
    symname: *const cty::c_char,
) -> cty::c_int {
    use bindings::{current_time, d_instantiate, iput, ENOSPC, S_IFLNK, S_IRWXUGO};
    let name = unsafe { cstr_core::CStr::from_ptr(symname) };

    match rs_ramfs_get_inode((*dir).i_sb, dir, (S_IFLNK as u16) | (S_IRWXUGO as u16), 0){
        Some(inode) => {
            let len = name.to_str().unwrap().len() + 1;
            match rs_page_symlink(inode, symname, len as cty::c_int) }{
                Ok(_) => {
                    unsafe { d_instantiate(dentry, inode) };
                    unsafe { c_dget(dentry) };
                    unsafe { (*dir).i_mtime = current_time(dir) };
                    unsafe { (*dir).i_ctime = (*dir).i_mtime };
                    0
            },
                Err(v) => {
                    unsafe { iput(inode) };
                    v
                }             
        },
        None => -(ENOSPC as cty::c_int)
    }
}

#[no_mangle]
pub extern "C" fn ramfs_kill_super(sb: *mut bindings::super_block) {
    use bindings::{kfree, kill_litter_super};
    unsafe { kfree((*sb).s_fs_info) };
    unsafe { kill_litter_super(sb) };
}
