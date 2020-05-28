export BASE_DIR := $(patsubst %/,%,$(dir $(abspath $(lastword $(MAKEFILE_LIST)))))

all: build/Makefile
	make -C /lib/modules/$(shell uname -r)/build M=$(BASE_DIR)/build modules

cleanbuild: clean
	make all

build/Makefile: kbuild.mk
	mkdir -p build
	cp kbuild.mk build/Makefile

clean:
	rm -r build
	cargo clean

test: build/ext2rs.ko
	sudo insmod build/ext2rs.ko
	sudo rmmod ext2rs
	dmesg | tail -10