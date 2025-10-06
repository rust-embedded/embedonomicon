#![no_std]

// NEW!
pub trait GlobalLog: Sync {
    fn log(&self, address: u8);
}

pub trait Log {
    type Error;

    fn log(&mut self, address: u8) -> Result<(), Self::Error>;
}

#[macro_export]
macro_rules! log {
    // NEW!
    ($string:expr) => {
        unsafe {
            unsafe extern "Rust" {
                static LOGGER: &'static dyn $crate::GlobalLog;
            }

            #[unsafe(export_name = $string)]
            #[unsafe(link_section = ".log")]
            static SYMBOL: u8 = 0;

            $crate::GlobalLog::log(LOGGER, &SYMBOL as *const u8 as usize as u8)
        }
    };

    ($logger:expr, $string:expr) => {{
        #[unsafe(export_name = $string)]
        #[unsafe(link_section = ".log")]
        static SYMBOL: u8 = 0;

        $crate::Log::log(&mut $logger, &SYMBOL as *const u8 as usize as u8)
    }};
}

// NEW!
#[macro_export]
macro_rules! global_logger {
    ($logger:expr) => {
        #[unsafe(no_mangle)]
        pub static LOGGER: &dyn $crate::GlobalLog = &$logger;
    };
}
