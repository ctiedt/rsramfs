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

//extern struct dentry *dget(struct dentry *);

// Called on unmount.
// Frees a pointer we allocated in the superblock
// and kills it.
extern void ramfs_kill_super(struct super_block *sb);

// Create a "generic" inode and place it in a directory.
// We use this as a building block for more complex operations.
extern int ramfs_mknod(struct inode *dir, struct dentry *dentry, umode_t mode, dev_t dev);

// Uses our mknod to create a directory
// Notably, we need to increase its link count
// upon creation because any directory has
// at least to links
extern int ramfs_mkdir(struct inode *dir, struct dentry *dentry, umode_t mode);

// Creates a regular file
extern int ramfs_create(struct inode *dir, struct dentry *dentry, umode_t mode, bool excl);

// Create a symlink.
// We need some error checking here
// since symlinking could fail.
extern int ramfs_symlink(struct inode *dir, struct dentry *dentry, const char *symname);

// Creates a new inode and fills in the required fields,
// i.e. the supported operations.
// We only have to define some manually.
extern struct inode *ramfs_get_inode(struct super_block *sb, const struct inode *dir, umode_t mode, dev_t dev);

// Calls a function that mounts the fs without a block device
// since we only use RAM pages.
// Also initializes the superblock.
extern struct dentry *ramfs_mount(struct file_system_type *fs_type, int flags, const char *dev_name, void *data);

// Fills our superblock with the relevant info,
// namely some constants and supported operations.
// Also initializes an initial inode.
extern int ramfs_fill_super(struct super_block *sb, void *data, int silent);