obj-m += ext2rs.o
src := $(BASE_DIR)/src
ext2rs-objs := module.o ext2rs.o

$(obj)/ext2rs.o: $(src)/lib.rs
	(cd $(BASE_DIR); env RUST_TARGET_PATH=$(BASE_DIR) cargo xbuild --target x86_64-unknown-none-gnu)
	cp $(BASE_DIR)/target/x86_64-unknown-none-gnu/debug/ext2rs.a $(obj)/ext2rs.o