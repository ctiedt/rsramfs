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
    __set_page_dirty_nobuffers, address_space_operations, dentry, dev_t, 
    file_system_type, generic_delete_inode, 
     gfp_t, inode,
    inode_operations, page_symlink_inode_operations, seq_file,
    simple_dir_operations, simple_link, simple_lookup, simple_readpage,
    simple_rename, simple_rmdir, simple_statfs, simple_unlink, simple_write_begin,
    simple_write_end, super_block, super_operations, umode_t,
};

use c_fns::rs_page_symlink;
use c_structs::{
    Inode, SuperBlock, DEFAULT_ADDRESS_SPACE_OPERATIONS, 
     DEFAULT_SUPER_OPS, DEFAULT_INODE_OPERATIONS, RamfsFsInfo, RamfsMountOpts
};

extern "C" {
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

// The page operations all inodes must support.
const RAMFS_AOPS: address_space_operations = address_space_operations {
    readpage: Some(simple_readpage),
    write_begin: Some(simple_write_begin),
    write_end: Some(simple_write_end),
    set_page_dirty: Some(__set_page_dirty_nobuffers),
    ..DEFAULT_ADDRESS_SPACE_OPERATIONS
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
    ..DEFAULT_INODE_OPERATIONS
};

fn rs_ramfs_get_inode(
    sb: SuperBlock,
    dir: Inode,
    mode: umode_t,
    dev: dev_t,
) -> Option<Inode> {
    use bindings::{S_IFDIR, S_IFLNK, S_IFMT, S_IFREG};
    use c_structs::RamfsInodeOps;

    const GFP_HIGHUSER: gfp_t = 6422722;
    if let Some(inode) = Inode::new(sb) {
        inode.set_ino();
        inode.init_owner(dir, mode);
        inode.set_aops(&RAMFS_AOPS);
        inode.mapping_set_gfp_mask(GFP_HIGHUSER);
        inode.mapping_set_unevictable();
        inode.set_amctime_current();

        let _mode = u32::from(mode) & S_IFMT;
        match _mode {
            _ if _mode == S_IFREG => {
                inode.ramfs_set_inode_ops();
            }
            _ if _mode == S_IFDIR => {
                inode.set_inode_operations(&RAMFS_DIR_INODE_OPS);
                unsafe { inode.set_file_operations(&simple_dir_operations) };
                inode.inc_nlink();
            }
            _ if _mode == S_IFLNK => {
                unsafe { inode.set_inode_operations(&page_symlink_inode_operations) };
                inode.nohighmem();
            }
            _ => {
                inode.init_special_inode(mode, dev);
            }
        }

        return Some(inode);
    }

    None
}

#[no_mangle]
pub extern "C" fn ramfs_get_inode(
    sb: *mut super_block,
    dir: *mut inode,
    mode: umode_t,
    dev: dev_t,
) -> *mut inode {
    if let Some(inode) = rs_ramfs_get_inode(
        SuperBlock::from_ptr_unchecked(sb),
        Inode::from_ptr_unchecked(dir),
        mode,
        dev,
    ) {
        inode.get_ptr()
    } else {
        core::ptr::null_mut()
    }
}

fn rs_ramfs_mknod(
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



fn parse_octal(data: &str) -> Option<umode_t> {
    if data.chars().all(|c| "012345678".contains(c)) {
        Some(data.parse::<umode_t>().unwrap())
    } else {
        None
    }
}

fn ramfs_parse_options(
    data: &str,
    opts: &mut RamfsMountOpts,
) {
    for substr in data.split_terminator(","){
        match substr{
            _ if substr.starts_with("mode=") => opts.mode = parse_octal(substr.split_at(substr.find("=").unwrap()).1).unwrap(),
            "debug" => opts.debug = true,
            _ => {}
        }
    }
}

#[no_mangle]
pub extern "C" fn ramfs_fill_super(
    sb: *mut super_block,
    data: *mut cty::c_void,
    _silent: cty::c_int,
) -> cty::c_int {
    use bindings::{ENOMEM, PAGE_SHIFT, RAMFS_MAGIC, S_IFDIR};
    use c_fns::rs_d_make_root;
    const MAX_LFS_FILESIZE: i64 = core::i64::MAX;

    let mut fsi = alloc::boxed::Box::new(RamfsFsInfo {
        mount_opts: RamfsMountOpts {
            mode: RAMFS_DEFAULT_MODE,
            debug: false
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

        if data != core::ptr::null_mut() {
            let rsdata = unsafe { cstr_core::CStr::from_ptr(data as *const cty::c_char) };
            ramfs_parse_options((rsdata.to_str()).unwrap(), &mut fsi.mount_opts);
        }


        return match rs_ramfs_get_inode(
            SuperBlock::from_ptr_unchecked(sb),
            Inode::null(),
            S_IFDIR as u16 | fsi.mount_opts.mode,
            0,
        ) {
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
    if let Some(super_block) = SuperBlock::from_ptr(sb) {
        super_block.free_fs_info();
        rs_kill_litter_super(super_block);
    }
}
