// source: examples/coop.rs

#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m::asm;
use cortex_m_rt::{entry, exception};

// priority = 0 (lowest)
#[inline(never)]
#[entry]
fn main() -> ! {
    // omitted: enabling interrupts and setting their priorities

    loop {
        asm::nop();
    }
}

static mut COUNTER: u64 = 0;

// priority = 1
#[exception]
fn SysTick() {
    // exclusive access to `COUNTER`
    let counter: &mut u64 = unsafe { &mut COUNTER };

    *counter += 1;
}

// priority = 1
#[exception]
fn SVCall() {
    // exclusive access to `COUNTER`
    let counter: &mut u64 = unsafe { &mut COUNTER };

    *counter *= 2;
}
