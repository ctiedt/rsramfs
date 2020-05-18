obj-m += minimod.o
src := $(BASE_DIR)/src
minimod-objs := module.o libminimod.a

$(obj)/libminimod.a: $(src)/lib.rs
	(cd $(BASE_DIR); env RUST_TARGET_PATH=$(BASE_DIR) cargo xbuild --target x86_64-unknown-none-gnu)
	cp $(BASE_DIR)/target/x86_64-unknown-none-gnu/debug/libminimod.a $(obj)/libminimod.a