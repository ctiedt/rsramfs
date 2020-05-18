#include <linux/init.h>
#include <linux/kernel.h>
#include <linux/module.h>

MODULE_LICENSE("GPL");

extern int rust_main(void);

void callme(void)
{
    printk(KERN_INFO "Test successful!\n");
}

int test_fn(void) { return 42; }

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