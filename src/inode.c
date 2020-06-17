#include <linux/init.h>
#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/fs.h>
#include <linux/pagemap.h>
#include <linux/highmem.h>
#include <linux/time.h>
#include <linux/string.h>
#include <linux/backing-dev.h>
#include <linux/sched.h>
#include <linux/magic.h>
#include <linux/slab.h>
#include <linux/uaccess.h>
#include "rust_fns.h"

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Linus Torvalds");
MODULE_AUTHOR("Clemens Tiedt");

void _mapping_set_unevictable(struct address_space *m) {
    mapping_set_unevictable(m);
}

void _mapping_set_gfp_mask(struct address_space *m, gfp_t mask) {
    m->gfp_mask = mask;
}

struct dentry *c_dget(struct dentry *dentry)
{
    return dget(dentry);
}

void *c_alloc(size_t size)
{
    return kmalloc(size, GFP_KERNEL);
}

void c_print(char *msg, int len)
{
    printk(KERN_INFO "%.*s", len, msg);
}

// We need this declaration here because of a
// cyclic dependency.
static const struct inode_operations ramfs_dir_inode_ops;

// The mount options on our fs.
// We only support the mode.
struct ramfs_mount_opts
{
    umode_t mode;
};

// All the info we provide about our fs
// are the mount options.
struct ramfs_fs_info
{
    struct ramfs_mount_opts mount_opts;
};

#define RAMFS_DEFAULT_MODE 0775

unsigned long
ramfs_mmu_get_unmapped_area(struct file *file, unsigned long addr, unsigned long len,
                            unsigned long pgoff, unsigned long flags)
{
    return current->mm->get_unmapped_area(file, addr, len, pgoff, flags);
}

// Shows the mount options of our fs.
// In our case that should just be the mode,
// i.e. default permissions.
static int ramfs_show_options(struct seq_file *m, struct dentry *root)
{
    struct ramfs_fs_info *fsi = root->d_sb->s_fs_info;

    if (fsi->mount_opts.mode != RAMFS_DEFAULT_MODE)
        seq_printf(m, ",mode=%o", fsi->mount_opts.mode);
    return 0;
}

// The operations our superblock uses to communicate
// with outside programs
static const struct super_operations ramfs_ops = {
    .statfs = simple_statfs,
    .drop_inode = generic_delete_inode,
    .show_options = ramfs_show_options,
};

// Fills our superblock with the relevant info,
// namely some constants and supported operations.
// Also initializes an initial inode.
int ramfs_fill_super(struct super_block *sb, void *data, int silent)
{
    struct ramfs_fs_info *fsi;
    struct inode *inode;

    fsi = kzalloc(sizeof(struct ramfs_fs_info), GFP_KERNEL);
    sb->s_fs_info = fsi;
    if (!fsi)
        return -ENOMEM;

    sb->s_maxbytes = MAX_LFS_FILESIZE;
    sb->s_blocksize = PAGE_SIZE;
    sb->s_blocksize_bits = PAGE_SHIFT;
    sb->s_magic = RAMFS_MAGIC;
    sb->s_op = &ramfs_ops;
    sb->s_time_gran = 1;

    inode = ramfs_get_inode(sb, NULL, S_IFDIR | fsi->mount_opts.mode, 0);
    sb->s_root = d_make_root(inode);
    if (!sb->s_root)
        return -ENOMEM;

    return 0;
}

// Calls a function that mounts the fs without a block device
// since we only use RAM pages.
// Also initializes the superblock.
struct dentry *
ramfs_mount(struct file_system_type *fs_type, int flags,
            const char *dev_name, void *data)
{
    return mount_nodev(fs_type, flags, data, ramfs_fill_super);
}

// Describes the important parts of an fs.
// Specifically the name and functions for
// mounting and unmounting.
struct file_system_type ramfs_type = {
    .name = "rsramfs",
    .mount = ramfs_mount,
    .kill_sb = ramfs_kill_super,
    .fs_flags = FS_USERNS_MOUNT,
};

static int __init
init_ramfs_module(void)
{
    printk(KERN_INFO "Starting module...\n");
    return register_filesystem(&ramfs_type);
}

static void __exit
exit_ramfs_module(void)
{
    unregister_filesystem(&ramfs_type);
    printk(KERN_INFO "Stopping module\n");
}

module_init(init_ramfs_module);
module_exit(exit_ramfs_module);