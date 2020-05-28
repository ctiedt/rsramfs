obj-m += ext2rs.o
src := $(BASE_DIR)/src
ext2rs-objs := module.o libext2rs.o

$(obj)/libext2rs.o: $(src)/lib.rs
	(cd $(BASE_DIR); env RUST_TARGET_PATH=$(BASE_DIR) cargo xbuild --target x86_64-unknown-none-gnu)
	cp $(BASE_DIR)/target/x86_64-unknown-none-gnu/debug/libext2rs.a $(obj)/libext2rs.o