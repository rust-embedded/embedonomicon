#![no_main]
#![no_std]

use core::{ptr, arch::asm};

use rt::entry;

entry!(main);

static mut DATA: i32 = 1;

fn main() -> ! {
    unsafe {
        // check that DATA is properly initialized
        if ptr::read_volatile(&DATA) != 1 {
            // this makes QEMU crash
            asm!("BKPT");
        }
    }

    loop {}
}
