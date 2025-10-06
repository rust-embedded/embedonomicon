#![no_main]
#![no_std]

use rt::entry;

entry!(main);

fn main() -> ! {
    loop {}
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub fn HardFault(_ef: *const u32) -> ! {
    loop {}
}
