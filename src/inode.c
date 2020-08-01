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

void _mapping_set_unevictable(struct address_space *m)
{
    mapping_set_unevictable(m);
}

void _mapping_set_gfp_mask(struct address_space *m, gfp_t mask)
{
    m->gfp_mask = mask;
}

struct dentry *c_dget(struct dentry *dentry)
{
    return dget(dentry);
}

void *c_alloc(size_t size)
{
    return krealloc(NULL, size, GFP_KERNEL);
    //return kmalloc(size, GFP_KERNEL);
}

void c_print(char *msg, int len)
{
    printk(KERN_INFO "%.*s", len, msg);
}

unsigned long
ramfs_mmu_get_unmapped_area(struct file *file, unsigned long addr, unsigned long len,
                            unsigned long pgoff, unsigned long flags)
{
    return current->mm->get_unmapped_area(file, addr, len, pgoff, flags);
}

const struct super_operations ramfs_ops = {
    .statfs = simple_statfs,
    .drop_inode = generic_delete_inode,
    .show_options = ramfs_show_options,
};

void ramfs_sb_set_ops(struct super_block *sb)
{
    sb->s_op = &ramfs_ops;
}

// Operations supported by files.
// All of these are provided by generic functions.
const struct file_operations ramfs_file_operations = {
    .read_iter = generic_file_read_iter,
    .write_iter = generic_file_write_iter,
    .mmap = generic_file_mmap,
    .fsync = noop_fsync,
    .splice_read = generic_file_splice_read,
    .splice_write = iter_file_splice_write,
    .llseek = generic_file_llseek,
    .get_unmapped_area = ramfs_mmu_get_unmapped_area,
};

// Operations on regular file inodes.
// Provided by <linux/fs.h>.
const struct inode_operations ramfs_file_inode_operations = {
    .setattr = simple_setattr,
    .getattr = simple_getattr,
};

void ramfs_set_inode_ops(struct inode *inode)
{
    inode->i_op = &ramfs_file_inode_operations;
    inode->i_fop = &ramfs_file_operations;
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