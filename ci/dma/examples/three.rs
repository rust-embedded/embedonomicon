//! Overlapping use

#![deny(missing_docs, warnings)]

use shared::{Dma1Channel1, USART1_RX, USART1_TX};

/// A DMA transfer
pub struct Transfer<B> {
    buffer: B,
    // NOTE: added
    serial: Serial1,
}

impl<B> Transfer<B> {
    /// Blocks until the transfer is done and returns the buffer
    // NOTE: the return value has changed
    pub fn wait(self) -> (B, Serial1) {
        // Busy wait until the transfer is done
        while !self.is_done() {}

        (self.buffer, self.serial)
    }

    // ..
}

impl Serial1 {
    /// Receives data into the given `buffer` until it's filled
    ///
    /// Returns a value that represents the in-progress DMA transfer
    // NOTE we now take `self` by value
    pub fn read_exact(mut self, buffer: &'static mut [u8]) -> Transfer<&'static mut [u8]> {
        self.dma.set_source_address(USART1_RX, false);
        self.dma
            .set_destination_address(buffer.as_mut_ptr() as usize, true);
        self.dma.set_transfer_length(buffer.len());

        self.dma.start();

        // .. same as before ..

        Transfer {
            buffer,
            // NOTE: added
            serial: self,
        }
    }

    /// Sends out the given `buffer`
    ///
    /// Returns a value that represents the in-progress DMA transfer
    // NOTE we now take `self` by value
    pub fn write_all(mut self, buffer: &'static [u8]) -> Transfer<&'static [u8]> {
        self.dma.set_destination_address(USART1_TX, false);
        self.dma.set_source_address(buffer.as_ptr() as usize, true);
        self.dma.set_transfer_length(buffer.len());

        self.dma.start();

        // .. same as before ..

        Transfer {
            buffer,
            // NOTE: added
            serial: self,
        }
    }
}

#[allow(dead_code, unused_variables)]
fn read(serial: Serial1, buf: &'static mut [u8; 16]) {
    let t = serial.read_exact(buf);

    // let byte = serial.read(); //~ ERROR: `serial` has been moved

    // .. do stuff ..

    let (serial, buf) = t.wait();

    // .. do more stuff ..
}

#[allow(dead_code, unused_variables)]
fn reorder(serial: Serial1, buf: &'static mut [u8]) {
    // zero the buffer (for no particular reason)
    buf.iter_mut().for_each(|byte| *byte = 0);

    let t = serial.read_exact(buf);

    // ... do other stuff ..

    let (buf, serial) = t.wait();

    buf.reverse();

    // .. do stuff with `buf` ..
}

// UNCHANGED

fn main() {}

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
