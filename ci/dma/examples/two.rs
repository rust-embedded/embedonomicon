//! `mem::forget`

#![deny(missing_docs, warnings)]

use shared::{Dma1Channel1, USART1_RX, USART1_TX};

impl Serial1 {
    /// Receives data into the given `buffer` until it's filled
    ///
    /// Returns a value that represents the in-progress DMA transfer
    pub fn read_exact(&mut self, buffer: &'static mut [u8]) -> Transfer<&'static mut [u8]> {
        // .. same as before ..
        self.dma.set_source_address(USART1_RX, false);
        self.dma
            .set_destination_address(buffer.as_mut_ptr() as usize, true);
        self.dma.set_transfer_length(buffer.len());

        self.dma.start();

        Transfer { buffer }
    }

    /// Sends out the given `buffer`
    ///
    /// Returns a value that represents the in-progress DMA transfer
    pub fn write_all(mut self, buffer: &'static [u8]) -> Transfer<&'static [u8]> {
        // .. same as before ..
        self.dma.set_destination_address(USART1_TX, false);
        self.dma.set_source_address(buffer.as_ptr() as usize, true);
        self.dma.set_transfer_length(buffer.len());

        self.dma.start();

        Transfer { buffer }
    }
}

use core::mem;

#[allow(dead_code)]
fn sound(mut serial: Serial1, buf: &'static mut [u8; 16]) {
    // NOTE `buf` is moved into `foo`
    foo(&mut serial, buf);
    bar();
}

#[inline(never)]
fn foo(serial: &mut Serial1, buf: &'static mut [u8]) {
    // start a DMA transfer and forget the returned `Transfer` value
    mem::forget(serial.read_exact(buf));
}

#[allow(unused_mut, unused_variables)]
#[inline(never)]
fn bar() {
    // stack variables
    let mut x = 0;
    let mut y = 0;

    // use `x` and `y`
}

// UNCHANGED
fn main() {}

/// A singleton that represents serial port #1
pub struct Serial1 {
    dma: Dma1Channel1,
    // ..
}

/// A DMA transfer
pub struct Transfer<B> {
    buffer: B,
}

impl<B> Transfer<B> {
    /// Returns `true` if the DMA transfer has finished
    pub fn is_done(&self) -> bool {
        !Dma1Channel1::in_progress()
    }

    /// Blocks until the transfer is done and returns the buffer
    pub fn wait(self) -> B {
        // Busy wait until the transfer is done
        while !self.is_done() {}

        self.buffer
    }
}
