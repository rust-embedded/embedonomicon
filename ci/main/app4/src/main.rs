#![feature(asm)]
#![no_main]
#![no_std]

#[macro_use]
extern crate rt;

use core::ptr;

entry!(main);

static mut DATA: i32 = 1;

fn main() -> ! {
    unsafe {
        // check that DATA is properly initialized
        if ptr::read_volatile(&DATA) != 1 {
            // this makes QEMU crash
            asm!("BKPT" :::: "volatile");
        }
    }

    loop {}
}
