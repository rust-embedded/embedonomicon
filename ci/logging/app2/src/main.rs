#![no_main]
#![no_std]

use cortex_m_semihosting::{debug, hio};

use rt::entry;

entry!(main);

fn main() -> ! {
    let mut hstdout = hio::hstdout().unwrap();

    #[unsafe(export_name = "Hello, world!")]
    #[unsafe(link_section = ".log")] // <- NEW!
    static A: u8 = 0;

    let address = &A as *const u8 as usize as u8;
    hstdout.write_all(&[address]).unwrap(); // <- CHANGED!

    #[unsafe(export_name = "Goodbye")]
    #[unsafe(link_section = ".log")] // <- NEW!
    static B: u8 = 0;

    let address = &B as *const u8 as usize as u8;
    hstdout.write_all(&[address]).unwrap(); // <- CHANGED!

    debug::exit(debug::EXIT_SUCCESS);

    loop {}
}
