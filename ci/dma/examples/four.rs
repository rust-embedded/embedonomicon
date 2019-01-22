//! Compiler (mis)optimizations

#![deny(missing_docs, warnings)]

use core::sync::atomic::{self, Ordering};

use shared::{Dma1Channel1, USART1_RX, USART1_TX};

impl Serial1 {
    /// Receives data into the given `buffer` until it's filled
    ///
    /// Returns a value that represents the in-progress DMA transfer
    pub fn read_exact(mut self, buffer: &'static mut [u8]) -> Transfer<&'static mut [u8]> {
        self.dma.set_source_address(USART1_RX, false);
        self.dma
            .set_destination_address(buffer.as_mut_ptr() as usize, true);
        self.dma.set_transfer_length(buffer.len());

        // NOTE: added
        atomic::compiler_fence(Ordering::Release);

        // NOTE: this is a volatile *write*
        self.dma.start();

        Transfer {
            buffer,
            serial: self,
        }
    }

    /// Sends out the given `buffer`
    ///
    /// Returns a value that represents the in-progress DMA transfer
    pub fn write_all(mut self, buffer: &'static [u8]) -> Transfer<&'static [u8]> {
        self.dma.set_destination_address(USART1_TX, false);
        self.dma.set_source_address(buffer.as_ptr() as usize, true);
        self.dma.set_transfer_length(buffer.len());

        // NOTE: added
        atomic::compiler_fence(Ordering::Release);

        // NOTE: this is a volatile *write*
        self.dma.start();

        Transfer {
            buffer,
            serial: self,
        }
    }
}

impl<B> Transfer<B> {
    /// Blocks until the transfer is done and returns the buffer
    pub fn wait(self) -> (B, Serial1) {
        // NOTE: this is a volatile *read*
        while !self.is_done() {}

        // NOTE: added
        atomic::compiler_fence(Ordering::Acquire);

        (self.buffer, self.serial)
    }

    // ..
}

#[allow(dead_code, unused_variables)]
fn reorder(serial: Serial1, buf: &'static mut [u8], x: &mut u32) {
    // zero the buffer (for no particular reason)
    buf.iter_mut().for_each(|byte| *byte = 0);

    *x += 1;

    let t = serial.read_exact(buf); // compiler_fence(Ordering::Release) ▲

    // NOTE: the processor can't access `buf` between the fences
    // ... do other stuff ..
    *x += 2;

    let (buf, serial) = t.wait(); // compiler_fence(Ordering::Acquire) ▼

    *x += 3;

    buf.reverse();

    // .. do stuff with `buf` ..
}

// UNCHANGED

fn main() {}

/// A DMA transfer
pub struct Transfer<B> {
    buffer: B,
    serial: Serial1,
}

/// A singleton that represents serial port #1
pub struct Serial1 {
    dma: Dma1Channel1,
    // ..
}

impl<B> Transfer<B> {
    /// Returns `true` if the DMA transfer has finished
    pub fn is_done(&self) -> bool {
        !Dma1Channel1::in_progress()
    }
}
