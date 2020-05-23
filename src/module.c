#include <linux/init.h>
#include <linux/kernel.h>
#include <linux/module.h>
#include <linux/slab.h>

MODULE_LICENSE("GPL");

extern int rust_main(void);

void *c_alloc(size_t size)
{
    return kmalloc(size, GFP_KERNEL);
}

void c_print(char *msg, int len)
{
    printk(KERN_INFO "%.*s", len, msg);
}

int init_km(void)
{
    printk(KERN_INFO "Starting module...\n");
    return rust_main();
}

void exit_km(void)
{
    printk(KERN_INFO "Stopping module...\n");
}

module_init(init_km);
module_exit(exit_km);