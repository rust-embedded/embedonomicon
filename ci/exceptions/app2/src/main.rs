#![feature(core_intrinsics)]
#![no_main]
#![no_std]

#[macro_use]
extern crate rt;

use core::intrinsics;

entry!(main);

fn main() -> ! {
    unsafe { intrinsics::abort() }
}

#[no_mangle]
pub extern "C" fn HardFault() -> ! {
    // do something interesting here
    loop {}
}
