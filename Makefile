export BASE_DIR := $(patsubst %/,%,$(dir $(abspath $(lastword $(MAKEFILE_LIST)))))
export TARGET := x86_64-unknown-none-gnu

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

test: build/rsramfs.ko
	sudo insmod build/rsramfs.ko
	sudo rmmod rsramfs
	dmesg | tail -10