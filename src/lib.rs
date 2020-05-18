#![feature(lang_items)]
#![no_std]

#[lang = "eh_personality"]
fn rust_eh_personality() {}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern "C" {
    fn callme();
}

#[no_mangle]
pub extern "C" fn rust_main() -> i32 {
    unsafe {
        callme();
    }
    0
}
