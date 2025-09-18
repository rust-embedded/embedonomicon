#![no_main]
#![no_std]

use core::arch::asm;

use rt::entry;

entry!(main);

fn main() -> ! {
    unsafe { asm!("udf #0", options(noreturn)) };
}

#[unsafe(no_mangle)]
pub extern "C" fn HardFault() -> ! {
    // do something interesting here
    loop {}
}
