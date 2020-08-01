# rsramfs

An implementation of the `ramfs` file system in [Rust](https://rust-lang.org). We are currently at the stage of porting the C source code to unsafe Rust.

## How to build

You need a working C toolchain, headers for the kernel you are compiling for and a (nightly) Rust toolchain with `cargo-xbuild`.
This also requires `rust-src` (install it with `rustup component add rust-src`).
To compile the module, run `make all` from the base directory. Try it out with `./test.sh` or `sudo insmod build/rsramfs.ko` 
(don't forget to unload the module with `sudo rmmod rsramfs` afterwards).

## A short overview

### What `inode.c` does

The easiest way to create a kernel module in a language other than C is to write some C code that does all the basic work for initializing
a kernel module (which uses C macros and/or inline functions which are hard to call via FFI) and then calls a Rust function to take over.
It also exports some functions to do with memory allocation and IO.

### What the Rust files do

### `io.rs` and `mem.rs`

These files contain functions that have to do with memory allocation and IO, such as a definition of Rust's `print!` macro using `kprintf`.

### `bindings.rs`, `c_structs.rs` and `c_fns.rs`

These files contain bindings to the Linux kernel interface we use and Rust wrappers around them. Most of these wrappers are hand-written.

### `lib.rs`

This is probably the most interesting module. It contains all of the `rsramfs` operations such as mounting, creating files etc.
