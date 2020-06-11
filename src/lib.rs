#![feature(lang_items)]
#![no_std]
#![feature(allocator_api)]
#![feature(alloc_error_handler)]
#![feature(const_fn)]
#![feature(panic_info_message)]
extern crate alloc;

#[macro_use]
mod io;
mod mem;

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
mod bindings;

use bindings::{dentry, inode, super_block, umode_t};

extern "C" {
    fn ramfs_get_inode(
        sb: *mut super_block,
        dir: *const inode,
        mode: umode_t,
        dev: cty::c_int,
    ) -> *mut inode;
    fn c_dget(dentry: *mut dentry) -> *mut dentry;
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

#[no_mangle]
pub extern "C" fn ramfs_mknod(
    dir: *mut inode,
    dentry: *mut dentry,
    mode: umode_t,
    dev: cty::c_int,
) -> cty::c_int {
    use bindings::{current_time, d_instantiate, ENOSPC};

    let inode = unsafe { ramfs_get_inode((*dir).i_sb, dir, mode, dev) };
    let mut error = -(ENOSPC as i32);

    if inode != core::ptr::null_mut() {
        unsafe { d_instantiate(dentry, inode) };
        unsafe { c_dget(dentry) };
        error = 0;
        let current_time = unsafe { current_time(dir) };
        unsafe { (*dir).i_mtime = current_time };
        unsafe { (*dir).i_ctime = current_time };
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
extern "C" fn ramfs_kill_super(sb: *mut bindings::super_block) {
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
