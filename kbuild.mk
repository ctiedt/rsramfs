obj-m += rsramfs.o
src := $(BASE_DIR)/src
rsramfs-objs := inode.o librsramfs.o

$(obj)/librsramfs.o: $(src)/lib.rs
	(cd $(BASE_DIR); env RUST_TARGET_PATH=$(BASE_DIR) cargo xbuild --target x86_64-unknown-none-gnu)
	cp $(BASE_DIR)/target/x86_64-unknown-none-gnu/debug/librsramfs.a $(obj)/librsramfs.o