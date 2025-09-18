#![no_main]
#![no_std]

use core::cell::RefCell;
use cortex_m::interrupt;
use cortex_m::interrupt::Mutex;
use cortex_m_semihosting::{
    debug,
    hio::{self, HostStream},
};

use log::{GlobalLog, global_logger, log};
use rt::entry;

struct Logger;

global_logger!(Logger);

entry!(main);

fn main() -> ! {
    log!("Hello, world!");

    log!("Goodbye");

    debug::exit(debug::EXIT_SUCCESS);

    loop {}
}

impl GlobalLog for Logger {
    fn log(&self, address: u8) {
        // we use a critical section (`interrupt::free`) to make the access to the
        // `HSTDOUT` variable interrupt-safe which is required for memory safety
        interrupt::free(|cs| {
            static HSTDOUT: Mutex<RefCell<Option<HostStream>>> = Mutex::new(RefCell::new(None));
            let mut hstdout = HSTDOUT.borrow(cs).borrow_mut();

            // lazy initialization
            if hstdout.is_none() {
                hstdout.replace(hio::hstdout()?);
            }

            let hstdout = hstdout.as_mut().unwrap();

            hstdout.write_all(&[address])
        })
        .ok(); // `.ok()` = ignore errors
    }
}
