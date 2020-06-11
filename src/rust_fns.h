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