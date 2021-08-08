#![feature(core_intrinsics)]
#![no_main]
#![no_std]

use core::intrinsics;

use rt::entry;

entry!(main);

fn main() -> ! {
    // this executes the undefined instruction (UDF) and causes a HardFault exception
    intrinsics::abort()
}
