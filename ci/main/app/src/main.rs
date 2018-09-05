#![feature(panic_handler)]
#![no_std]
#![no_main]

extern crate rt;

use core::panic::PanicInfo;

#[no_mangle]
pub fn main() -> ! {
    let _x = 42;

    loop {}
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}
