# minimal-rust-km

A simple prototype to show how you can build a linux kernel module using Rust.

## How to build

You need a working C toolchain, headers for the kernel you are compiling for and a (nightly) Rust toolchain with `cargo-xbuild`.
To compile the module, run `make all` from the base directory. Try it out with `make test` or `sudo insmod build/minimod.ko` 
(don't forget to unload the module with `sudo rmmod minimod` afterwards).

## A short overview

### What `module.c` does

The easiest way to create a kernel module in a language other than C is to write some C code that does all the basic work for initializing
a kernel module (which uses C macros and/or inline functions which are hard to call via FFI) and then calls a Rust function to take over.
We also include a C function to be called from Rust as an example of how to additionally use this C file as a sort of library.

### What `lib.rs` does

This is where most of the code in a real kernel module should go. In our case, we just have a `rust_main` function that calls a C function.
Since we are in a `#[no_std]` context, we need to define some language items ourselves.

### What's still missing

One of the biggest missing things is the `print!()` macro. To implement this, you could create a struct that implements 
the formatting trait and use a static instance of it to define your `print!()` macro using the C `kprintf` function. You
might also want to create an allocator using `krealloc` and `kfree` to be able to use heap-allocated data types.
