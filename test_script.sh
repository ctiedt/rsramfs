#!/bin/sh

make
sudo insmod build/rsramfs.ko
sudo mount -t rsramfs none test
sudo chmod a+rwx test
touch test/file
echo "Hello World" > test/file2
dmesg
sudo umount test
sudo rmmod rsramfs