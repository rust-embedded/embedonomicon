#![feature(core_intrinsics)]
#![no_main]
#![no_std]

#[macro_use]
extern crate rt;

use core::intrinsics;

entry!(main);

fn main() -> ! {
    // this executes the undefined instruction (UDF) and causes a HardFault exception
    unsafe { intrinsics::abort() }
}
