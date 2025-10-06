#![no_main]
#![no_std]

use core::{arch::asm, ptr};

use rt::entry;

entry!(main);

static mut DATA: i32 = 1;

#[allow(static_mut_refs)]
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
