#![feature(lang_items, const_fn, box_into_raw_non_null)]
#![no_std]
#![feature(allocator_api)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![allow(improper_ctypes)]
extern crate alloc;

#[macro_use]
mod io;
mod c_fns;
mod c_structs;
mod mem;

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
mod bindings;

use bindings::{
    address_space, dentry, dev_t, file_system_type, generic_delete_inode, gfp_t, inode, seq_file,
    simple_statfs, super_block, super_operations, umode_t,
};

use c_fns::rs_page_symlink;
use c_structs::{Inode, DEFAULT_SUPER_OPS};

extern "C" {
    fn _mapping_set_gfp_mask(m: *mut address_space, mask: gfp_t);
    fn _mapping_set_unevictable(m: *mut address_space);
    fn ramfs_show_options(m: *mut seq_file, root: *mut dentry) -> cty::c_int;
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

fn rs_ramfs_mknod(
    dir: Inode,
    dentry: *mut dentry,
    mode: umode_t,
    dev: dev_t,
) -> Result<(), cty::c_int> {
    use bindings::ENOSPC;
    use c_fns::{rs_d_instantiate, rs_dget, rs_ramfs_get_inode};

    match rs_ramfs_get_inode(dir.get_sb(), dir, mode, dev) {
        Some(inode) => {
            rs_d_instantiate(dentry, inode);
            rs_dget(dentry);
            dir.set_mctime_current();
            Ok(())
        }
        None => Err(-(ENOSPC as i32)),
    }
}

#[no_mangle]
pub extern "C" fn ramfs_mknod(
    dir: *mut inode,
    dentry: *mut dentry,
    mode: umode_t,
    dev: dev_t,
) -> cty::c_int {
    match rs_ramfs_mknod(Inode::from_ptr_unchecked(dir), dentry, mode, dev) {
        Ok(()) => 0,
        Err(e) => e,
    }
}

#[no_mangle]
pub extern "C" fn ramfs_mkdir(dir: *mut inode, dentry: *mut dentry, mode: umode_t) -> cty::c_int {
    use bindings::S_IFDIR;
    use c_fns::rs_inc_nlink;
    match rs_ramfs_mknod(
        Inode::from_ptr_unchecked(dir),
        dentry,
        mode | (S_IFDIR as u16),
        0,
    ) {
        Ok(_) => {
            rs_inc_nlink(Inode::from_ptr_unchecked(dir));
            0
        }
        Err(e) => e,
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
    match rs_ramfs_mknod(
        Inode::from_ptr_unchecked(dir),
        dentry,
        mode | (S_IFREG as u16),
        0,
    ) {
        Ok(_) => 0,
        Err(e) => e,
    }
}

#[no_mangle]
pub extern "C" fn ramfs_symlink(
    dir: *mut inode,
    dentry: *mut dentry,
    symname: *const cty::c_char,
) -> cty::c_int {
    use bindings::{ENOSPC, S_IFLNK, S_IRWXUGO};
    use c_fns::{rs_d_instantiate, rs_dget, rs_iput, rs_ramfs_get_inode};
    let name = unsafe { cstr_core::CStr::from_ptr(symname) };

    let dir_inode = Inode::from_ptr_unchecked(dir);

    match rs_ramfs_get_inode(
        dir_inode.get_sb(),
        dir_inode,
        (S_IFLNK as u16) | (S_IRWXUGO as u16),
        0,
    ) {
        Some(inode) => {
            let len = name.to_str().unwrap().len() + 1;
            match rs_page_symlink(inode, symname, len as cty::c_int) {
                Ok(_) => {
                    rs_d_instantiate(dentry, inode);
                    rs_dget(dentry);
                    dir_inode.set_mctime_current();
                    0
                }
                Err(e) => {
                    rs_iput(inode);
                    e
                }
            }
        }
        None => -(ENOSPC as cty::c_int),
    }
}

const RAMFS_OPS: super_operations = super_operations {
    statfs: Some(simple_statfs),
    drop_inode: Some(generic_delete_inode),
    show_options: Some(ramfs_show_options),
    ..DEFAULT_SUPER_OPS
};

const RAMFS_DEFAULT_MODE: umode_t = 0775;

struct RamfsMountOpts {
    mode: umode_t,
}

pub struct RamfsFsInfo {
    mount_opts: RamfsMountOpts,
}

#[no_mangle]
pub extern "C" fn ramfs_fill_super(
    sb: *mut super_block,
    _data: *mut cty::c_void,
    _silent: cty::c_int,
) -> cty::c_int {
    use bindings::{ENOMEM, PAGE_SHIFT, RAMFS_MAGIC, S_IFDIR};
    use c_fns::{rs_d_make_root, rs_ramfs_get_inode};
    use c_structs::SuperBlock;
    const MAX_LFS_FILESIZE: i64 = core::i64::MAX;

    let mut fsi = alloc::boxed::Box::new(RamfsFsInfo {
        mount_opts: RamfsMountOpts {
            mode: RAMFS_DEFAULT_MODE,
        },
    });

    if let Some(super_block) = SuperBlock::from_ptr(sb) {
        super_block.set_fs_info(&mut *fsi);
        super_block.set_fields(
            MAX_LFS_FILESIZE,
            PAGE_SHIFT as cty::c_uchar,
            RAMFS_MAGIC as cty::c_ulonglong,
            &RAMFS_OPS,
            1,
        );
    }

    match rs_ramfs_get_inode(sb, Inode::null(), S_IFDIR as u16 | fsi.mount_opts.mode, 0) {
        Some(inode) => {
            unsafe { (*sb).s_root = rs_d_make_root(inode) };
            0
        }
        None => -(ENOMEM as i32),
    }
}

#[no_mangle]
pub extern "C" fn ramfs_mount(
    fs_type: *mut file_system_type,
    flags: cty::c_int,
    _dev_name: *const cty::c_char,
    data: *mut cty::c_void,
) -> *mut dentry {
    use c_fns::rs_mount_nodev;
    rs_mount_nodev(fs_type, flags, data, Some(ramfs_fill_super))
}

#[no_mangle]
pub extern "C" fn ramfs_kill_super(sb: *mut super_block) {
    use c_fns::rs_kill_litter_super;
    use c_structs::SuperBlock;
    if let Some(super_block) = SuperBlock::from_ptr(sb) {
        super_block.free_fs_info();
        rs_kill_litter_super(super_block);
    }
}
