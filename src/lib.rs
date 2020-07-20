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
    __set_page_dirty_nobuffers, address_space, address_space_operations, dentry, dev_t, file,
    file_operations, file_system_type, generic_delete_inode, generic_file_llseek,
    generic_file_mmap, generic_file_read_iter, generic_file_splice_read, generic_file_write_iter,
    gfp_t, inode, inode_nohighmem, inode_operations, iter_file_splice_write, noop_fsync,
    page_symlink_inode_operations, seq_file, simple_dir_operations, simple_getattr, simple_link,
    simple_lookup, simple_readpage, simple_rename, simple_rmdir, simple_setattr, simple_statfs,
    simple_unlink, simple_write_begin, simple_write_end, super_block, super_operations, umode_t,
};

use c_fns::rs_page_symlink;
use c_structs::{Inode, DEFAULT_SUPER_OPS};

extern "C" {
    fn _mapping_set_gfp_mask(m: *mut address_space, mask: gfp_t);
    fn _mapping_set_unevictable(m: *mut address_space);
    fn ramfs_show_options(m: *mut seq_file, root: *mut dentry) -> cty::c_int;
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

// The page operations all inodes must support.
const RAMFS_AOPS: address_space_operations = address_space_operations {
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

// The operations we support on directories.
// We provide some ourselves and use generic
// implementations for others.
const RAMFS_DIR_INODE_OPS: inode_operations = inode_operations {
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

// Operations on regular file inodes.
// Provided by <linux/fs.h>.
static mut RAMFS_FILE_INODE_OPS: inode_operations = inode_operations {
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

// Operations supported by files.
// All of these are provided by generic functions.
static mut RAMFS_FILE_OPS: file_operations = file_operations {
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
pub extern "C" fn rs_ramfs_get_inode(
    sb: *mut super_block,
    dir: Inode,
    mode: umode_t,
    dev: dev_t,
) -> Option<Inode> {
    use bindings::{
        current_time, get_next_ino, inc_nlink, init_special_inode, inode_init_owner, new_inode,
        S_IFDIR, S_IFLNK, S_IFMT, S_IFREG,
    };

    const GFP_HIGHUSER: u32 = 6422722;
    let inode: *mut inode = unsafe { new_inode(sb) };

    if inode != core::ptr::null_mut() {
        unsafe { (*inode).i_ino = get_next_ino().into() };
        unsafe { inode_init_owner(inode, dir.get_ptr(), mode) };
        unsafe { (*(*inode).i_mapping).a_ops = &RAMFS_AOPS };
        unsafe { _mapping_set_gfp_mask((*inode).i_mapping, GFP_HIGHUSER) };
        unsafe { _mapping_set_unevictable((*inode).i_mapping) };
        unsafe { (*inode).i_atime = current_time(inode) };
        unsafe { (*inode).i_mtime = (*inode).i_atime };
        unsafe { (*inode).i_ctime = (*inode).i_atime };
        let _mode = u32::from(mode) & S_IFMT;
        match _mode {
            _ if _mode == S_IFREG => {
                unsafe { (*inode).i_op = &RAMFS_FILE_INODE_OPS };
                unsafe { (*inode).i_fop = &RAMFS_FILE_OPS };
            }
            _ if _mode == S_IFDIR => {
                unsafe { (*inode).i_op = &RAMFS_DIR_INODE_OPS };
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

    Inode::from_ptr(inode)
}

// Creates a new inode and fills in the required fields,
// i.e. the supported operations.
// We only have to define some manually.
#[no_mangle]
pub extern "C" fn ramfs_get_inode(
    sb: *mut super_block,
    dir: *mut inode,
    mode: umode_t,
    dev: dev_t,
) -> *mut inode {
    if let Some(inode) = rs_ramfs_get_inode(sb, Inode::from_ptr_unchecked(dir), mode, dev) {
        inode.get_ptr()
    } else {
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn rs_ramfs_mknod(
    dir: Inode,
    dentry: *mut dentry,
    mode: umode_t,
    dev: dev_t,
) -> Result<(), cty::c_int> {
    use bindings::ENOSPC;
    use c_fns::{rs_d_instantiate, rs_dget};

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
    use c_fns::{rs_d_instantiate, rs_dget, rs_iput};
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

// The operations our superblock uses to communicate
// with outside programs
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
    use c_fns::rs_d_make_root;
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

        return match rs_ramfs_get_inode(sb, Inode::null(), S_IFDIR as u16 | fsi.mount_opts.mode, 0)
        {
            Some(inode) => {
                super_block.set_root(rs_d_make_root(inode));
                0
            }
            None => -(ENOMEM as i32),
        };
    }

    -(ENOMEM as i32)
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
