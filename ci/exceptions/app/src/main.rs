#![no_main]
#![no_std]

use core::arch::asm;
use rt::entry;

entry!(main);

fn main() -> ! {
    // this executes the undefined instruction (UDF) and causes a HardFault exception
    unsafe { asm!("udf #0", options(noreturn)) };
}
