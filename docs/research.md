# Our research so far

## Previous Rust kernel module implementations

- [rust.ko](https://github.com/tsgates/rust.ko)
- [linux-kernel-module-rust](https://github.com/lizhuohua/linux-kernel-module-rust) by lizhuohua
- [kernel-roulette](https://github.com/souvik1997/kernel-roulette)
- [linux-kernel-module-rust](https://github.com/fishinabarrel/linux-kernel-module-rust) by fishinabarrel

## Resources about EXT2
- [The second extended file system](https://www.nongnu.org/ext2-doc/ext2.html) by Dave Poirier
- [The second extended file system](http://www.science.unitn.it/~fiorella/guidelinux/tlk/node95.html) by David A. Rusling
- [Design and Implementation of the Second Extended Filesystem](http://e2fsprogs.sourceforge.net/ext2intro.html) by Rémy Card et al. 
- [The ext2 repo](https://github.com/torvalds/linux/tree/master/fs/ext2)

## Getting started with file system drivers in Linux
- [Experiments with writing a filesystem driver for Linux:](https://www.uninformativ.de/blog/postings/2017-09-09/0/POSTING-en.html)
I managed to load the example file system onefilerofs into the kernel \(I think\).
Unfortunately, I haven't been able to get oneblockfs to work yet.
This is what I did \(with help from [this](https://kukuruku.co/post/writing-a-file-system-in-linux-kernel/)\):
    - `make`
    - `sudo insmod .\onefilerofs.ko`
    - `touch image`: this creates an \(empty\) disk image; I'm still unsure what I would have to do if this wasn't just an example
    - `mkdir dir`: this is the root of the file system
    - `sudo mount -o loop -t onefilerofs ./image ./dir`
    - to unmount: 
        - `sudo umount ./dir`
        - `sudo rmmod onefilerofs`
    
