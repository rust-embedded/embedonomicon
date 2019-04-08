//! THIS PROGRAM IS UNSOUND!
// source: examples/volatile.rs

#![no_main]
#![no_std]

extern crate panic_halt;

use core::ptr;

use cortex_m::asm;
use cortex_m_rt::{entry, exception};

#[repr(u64)]
enum Enum {
    A = 0x0000_0000_ffff_ffff,
    B = 0xffff_ffff_0000_0000,
}

static mut X: Enum = Enum::A;

#[entry]
fn main() -> ! {
    // omitted: configuring and enabling the `SysTick` interrupt

    loop {
        // this write operation is not atomic: it's performed in two moves
        unsafe { ptr::write_volatile(&mut X, Enum::A) } // <~ preemption

        unsafe { ptr::write_volatile(&mut X, Enum::B) }
    }
}

#[exception]
fn SysTick() {
    unsafe {
        // here we may observe `X` having the value `0x0000_0000_0000_0000`
        // or `0xffff_ffff_ffff_ffff` which are not valid `Enum` variants
        match X {
            Enum::A => asm::nop(),
            Enum::B => asm::bkpt(),
        }
    }
}
