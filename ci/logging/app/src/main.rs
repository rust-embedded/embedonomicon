#![no_main]
#![no_std]

use core::fmt::Write;
use cortex_m_semihosting::{debug, hio};

use rt::entry;

entry!(main);

fn main() -> ! {
    let mut hstdout = hio::hstdout().unwrap();

    #[unsafe(export_name = "Hello, world!")]
    static A: u8 = 0;

    let _ = writeln!(hstdout, "{:#x}", &A as *const u8 as usize);

    #[unsafe(export_name = "Goodbye")]
    static B: u8 = 0;

    let _ = writeln!(hstdout, "{:#x}", &B as *const u8 as usize);

    debug::exit(debug::EXIT_SUCCESS);

    loop {}
}
