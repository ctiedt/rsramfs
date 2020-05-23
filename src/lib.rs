#![feature(lang_items)]
#![no_std]
#![feature(allocator_api)]
#![feature(alloc_error_handler)]
#![feature(const_fn)]
#![feature(panic_info_message)]
extern crate alloc;

#[macro_use]
mod io;
mod mem;

#[cfg(not(test))]
#[lang = "eh_personality"]
fn rust_eh_personality() {}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[global_allocator]
static A: mem::KernelAllocator = mem::KernelAllocator {};

#[no_mangle]
pub extern "C" fn rust_main() -> i64 {
    println!("Hello, fairies and unicorns!");
    0
}
