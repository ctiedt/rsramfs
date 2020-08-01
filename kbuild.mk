obj-m += rsramfs.o
src := $(BASE_DIR)/src
rsramfs-objs := inode.o librsramfs.o

$(obj)/librsramfs.o: $(src)/*.rs
	(cd $(BASE_DIR); env RUST_TARGET_PATH=$(BASE_DIR) cargo build -Z build-std=core,alloc --target $(TARGET))
	cp $(BASE_DIR)/target/$(TARGET)/debug/librsramfs.a $(obj)/librsramfs.o