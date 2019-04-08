// source: examples/cs3.rs

#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m::{asm, register::basepri};
use cortex_m_rt::{entry, exception};

// priority = 0 (lowest)
#[inline(never)]
#[entry]
fn main() -> ! {
    // omitted: enabling interrupts and setting up their priorities

    loop {
        asm::nop();
    }
}

static mut COUNTER: u64 = 0;

// priority = 2
#[exception]
fn SysTick() {
    // exclusive access to `COUNTER`
    let counter: &mut u64 = unsafe { &mut COUNTER };

    *counter += 1;
}

// priority = 1
#[exception]
fn SVCall() {
    // `SysTick` can preempt `SVCall` at this point

    // start of critical section: raise the running priority to 2
    raise(2);

    // `SysTick` can *not* preempt this block because it has a priority of 2 (equal)
    // `PendSV` *can* preempt this block because it has a priority of 3 (higher)
    {
        // exclusive access to `COUNTER`
        let counter: &mut u64 = unsafe { &mut COUNTER };

        *counter *= 2;
    }

    // start of critical section: lower the running priority to its original value
    unsafe { lower() }

    // `SysTick` can preempt `SVCall` again
}

// priority = 3
#[exception]
fn PendSV() {
    // .. does not access `COUNTER` ..
}

fn raise(priority: u8) {
    const PRIO_BITS: u8 = 3;

    // (priority is encoded in hardware in the higher order bits of a byte)
    // (also in this encoding a bigger number means lower priority)
    let p = ((1 << PRIO_BITS) - priority) << (8 - PRIO_BITS);

    unsafe { basepri::write(p) }
    //^= `asm!("MSR BASEPRI, $0" : "=r"(p) : : "memory" : "volatile")`
    //                                         ^^^^^^^^
}

unsafe fn lower() {
    basepri::write(0)
}
