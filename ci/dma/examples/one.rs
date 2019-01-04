//! A first stab

#![deny(missing_docs, warnings)]

use shared::{Dma1Channel1, USART1_RX, USART1_TX};

/// A singleton that represents serial port #1
pub struct Serial1 {
    // NOTE: we extend this struct by adding the DMA channel singleton
    dma: Dma1Channel1,
    // ..
}

impl Serial1 {
    /// Sends out the given `buffer`
    ///
    /// Returns a value that represents the in-progress DMA transfer
    pub fn write_all<'a>(mut self, buffer: &'a [u8]) -> Transfer<&'a [u8]> {
        self.dma.set_destination_address(USART1_TX, false);
        self.dma.set_source_address(buffer.as_ptr() as usize, true);
        self.dma.set_transfer_length(buffer.len());

        self.dma.start();

        Transfer { buffer }
    }
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

impl Serial1 {
    /// Receives data into the given `buffer` until it's filled
    ///
    /// Returns a value that represents the in-progress DMA transfer
    pub fn read_exact<'a>(&mut self, buffer: &'a mut [u8]) -> Transfer<&'a mut [u8]> {
        self.dma.set_source_address(USART1_RX, false);
        self.dma
            .set_destination_address(buffer.as_mut_ptr() as usize, true);
        self.dma.set_transfer_length(buffer.len());

        self.dma.start();

        Transfer { buffer }
    }
}

#[allow(dead_code)]
fn write(serial: Serial1) {
    // fire and forget
    serial.write_all(b"Hello, world!\n");

    // do other stuff
}

#[allow(dead_code)]
fn read(mut serial: Serial1) {
    let mut buf = [0; 16];
    let t = serial.read_exact(&mut buf);

    // do other stuff

    t.wait();

    match buf.split(|b| *b == b'\n').next() {
        Some(b"some-command") => { /* do something */ }
        _ => { /* do something else */ }
    }
}

use core::mem;

#[allow(dead_code)]
fn unsound(mut serial: Serial1) {
    start(&mut serial);
    bar();
}

#[inline(never)]
fn start(serial: &mut Serial1) {
    let mut buf = [0; 16];

    // start a DMA transfer and forget the returned `Transfer` value
    mem::forget(serial.read_exact(&mut buf));
}

#[allow(unused_variables, unused_mut)]
#[inline(never)]
fn bar() {
    // stack variables
    let mut x = 0;
    let mut y = 0;

    // use `x` and `y`
}

fn main() {}
