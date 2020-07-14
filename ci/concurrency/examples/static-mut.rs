//! THIS PROGRAM IS UNSOUND!
// source: examples/static-mut.rs

#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m::asm;
use cortex_m_rt::{entry, exception};

static mut X: u32 = 0;

#[inline(never)]
#[entry]
fn main() -> ! {
    // omitted: configuring and enabling the `SysTick` interrupt

    let x: &mut u32 = unsafe { &mut X };

    loop {
        *x = 0;

        // <~ preemption could occur here and change the value behind `x`

        if *x != 0 {
            // the compiler may optimize away this branch
            panic!();
        } else {
            asm::nop();
        }
    }
}

#[exception]
fn SysTick() {
    unsafe {
        X = 1;

        asm::nop();
    }
}
