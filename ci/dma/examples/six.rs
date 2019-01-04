//! Immovable buffers

#![deny(missing_docs, warnings)]

use core::{
    marker::Unpin,
    mem,
    ops::{Deref, DerefMut},
    pin::Pin,
    sync::atomic::{self, Ordering},
};

use as_slice::{AsMutSlice, AsSlice};
use shared::{Dma1Channel1, USART1_RX, USART1_TX};

/// A DMA transfer
pub struct Transfer<B> {
    // NOTE: changed
    buffer: Pin<B>,
    serial: Serial1,
}

impl Serial1 {
    /// Receives data into the given `buffer` until it's filled
    ///
    /// Returns a value that represents the in-progress DMA transfer
    pub fn read_exact<B>(mut self, mut buffer: Pin<B>) -> Transfer<B>
    where
        // NOTE: bounds changed
        B: DerefMut,
        B::Target: AsMutSlice<Element = u8> + Unpin,
    {
        // .. same as before ..
        let slice = buffer.as_mut_slice();
        let (ptr, len) = (slice.as_mut_ptr(), slice.len());

        self.dma.set_source_address(USART1_RX, false);
        self.dma.set_destination_address(ptr as usize, true);
        self.dma.set_transfer_length(len);

        atomic::compiler_fence(Ordering::Release);
        self.dma.start();

        Transfer {
            buffer,
            serial: self,
        }
    }

    /// Sends out the given `buffer`
    ///
    /// Returns a value that represents the in-progress DMA transfer
    pub fn write_all<B>(mut self, buffer: Pin<B>) -> Transfer<B>
    where
        // NOTE: bounds changed
        B: Deref,
        B::Target: AsSlice<Element = u8>,
    {
        // .. same as before ..
        let slice = buffer.as_slice();
        let (ptr, len) = (slice.as_ptr(), slice.len());

        self.dma.set_destination_address(USART1_TX, false);
        self.dma.set_source_address(ptr as usize, true);
        self.dma.set_transfer_length(len);

        atomic::compiler_fence(Ordering::Release);
        self.dma.start();

        Transfer {
            buffer,
            serial: self,
        }
    }
}

#[allow(dead_code, unused_variables)]
fn static_mut(serial: Serial1, buf: &'static mut [u8]) {
    let buf = Pin::new(buf);

    let t = serial.read_exact(buf);

    // ..

    let (buf, serial) = t.wait();

    // ..
}

#[allow(dead_code, unused_variables)]
fn boxed(serial: Serial1, buf: Box<[u8]>) {
    let buf = Pin::new(buf);

    let t = serial.read_exact(buf);

    // ..

    let (buf, serial) = t.wait();

    // ..
}

#[allow(dead_code)]
fn unsound(serial: Serial1) {
    start(serial);

    bar();
}

// pin-utils = "0.1.0-alpha.4"
use pin_utils::pin_mut;

#[inline(never)]
fn start(serial: Serial1) {
    let buffer = [0; 16];

    // pin the `buffer` to this stack frame
    // `buffer` now has type `Pin<&mut [u8; 16]>`
    pin_mut!(buffer);

    mem::forget(serial.read_exact(buffer));
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

impl<B> Transfer<B> {
    /// Returns `true` if the DMA transfer has finished
    pub fn is_done(&self) -> bool {
        !Dma1Channel1::in_progress()
    }

    /// Blocks until the transfer is done and returns the buffer
    pub fn wait(self) -> (Pin<B>, Serial1) {
        while self.is_done() {}

        atomic::compiler_fence(Ordering::Acquire);

        (self.buffer, self.serial)
    }
}
