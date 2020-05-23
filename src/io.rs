// IO taken from https://github.com/souvik1997/kernel-roulette (GPL-3.0)

use alloc::vec::Vec;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

extern "C" {
    fn c_print(msg: *const u8, len: usize);
}

lazy_static! {
    pub static ref K_WRITER: Mutex<KernelWriter> = Mutex::new(KernelWriter::default());
}

#[derive(Default)]
pub struct KernelWriter {
    buffer: Vec<u8>,
}

impl KernelWriter {
    fn flush(&mut self) {
        unsafe { c_print(self.buffer.as_ptr(), self.buffer.len()) };
        self.buffer.clear();
    }
}

impl fmt::Write for KernelWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.buffer.extend(s.bytes());
        Ok(())
    }
}

#[allow(unused_macros)]
macro_rules! print {
    ($($arg:tt)*) => ($crate::io::print(format_args!($($arg)*)));
}

#[allow(unused_macros)]
macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

pub fn print(args: fmt::Arguments) {
    use core::fmt::Write;
    let mut writer = K_WRITER.lock();
    writer.write_fmt(args).unwrap();
    writer.flush();
}
