#![no_std]

use core::panic::PanicInfo;
use core::ptr;

#[unsafe(no_mangle)]
#[allow(static_mut_refs)]
pub unsafe extern "C" fn Reset() -> ! {
    // NEW!
    // Initialize RAM
    unsafe extern "C" {
        static mut _sbss: u8;
        static mut _ebss: u8;

        static mut _sdata: u8;
        static mut _edata: u8;
        static _sidata: u8;
    }

    let count = unsafe { &_ebss as *const u8 as usize - &_sbss as *const u8 as usize };
    unsafe { ptr::write_bytes(&mut _sbss as *mut u8, 0, count) };

    let count = unsafe { &_edata as *const u8 as usize - &_sdata as *const u8 as usize };
    unsafe { ptr::copy_nonoverlapping(&_sidata as *const u8, &mut _sdata as *mut u8, count) };

    // Call user entry point
    unsafe extern "Rust" {
        safe fn main() -> !;
    }

    main()
}

// The reset vector, a pointer into the reset handler
#[unsafe(link_section = ".vector_table.reset_vector")]
#[unsafe(no_mangle)]
pub static RESET_VECTOR: unsafe extern "C" fn() -> ! = Reset;

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

#[macro_export]
macro_rules! entry {
    ($path:path) => {
        #[unsafe(export_name = "main")]
        pub unsafe fn __main() -> ! {
            // type check the given path
            let f: fn() -> ! = $path;

            f()
        }
    };
}
