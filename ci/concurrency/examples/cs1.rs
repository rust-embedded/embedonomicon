// source: examples/cs1.rs

#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m::interrupt;
use cortex_m_rt::{entry, exception};

static mut COUNTER: u64 = 0;

#[inline(never)]
#[entry]
fn main() -> ! {
    loop {
        // `SysTick` can preempt `main` at this point

        // start of critical section: disable interrupts
        interrupt::disable(); // = `asm!("CPSID I" : : : "memory" : "volatile")`
                              //                         ^^^^^^^^

        // `SysTick` can not preempt this block
        {
            let counter: &mut u64 = unsafe { &mut COUNTER };

            *counter += 1;
        }

        // end of critical section: re-enable interrupts
        unsafe { interrupt::enable() }
        //^= `asm!("CPSIE I" : : : "memory" : "volatile")`
        //                         ^^^^^^^^

        // `SysTick` can start at this point
    }
}

#[exception]
fn SysTick() {
    // exclusive access to `COUNTER`
    let counter: &mut u64 = unsafe { &mut COUNTER };

    *counter += 1;
}
