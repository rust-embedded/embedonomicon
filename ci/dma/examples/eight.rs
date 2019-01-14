//! Destructors

#![deny(missing_docs, warnings)]

use core::{
    hint,
    marker::Unpin,
    mem,
    ops::{Deref, DerefMut},
    pin::Pin,
    ptr,
    sync::atomic::{self, Ordering},
};

use as_slice::{AsMutSlice, AsSlice};
use shared::{Dma1Channel1, USART1_RX, USART1_TX};

/// A DMA transfer
pub struct Transfer<B> {
    // NOTE: always `Some` variant
    inner: Option<Inner<B>>,
}

// NOTE: previously named `Transfer<B>`
struct Inner<B> {
    buffer: Pin<B>,
    serial: Serial1,
}

impl<B> Transfer<B> {
    /// Blocks until the transfer is done and returns the buffer
    pub fn wait(mut self) -> (Pin<B>, Serial1) {
        while !self.is_done() {}

        atomic::compiler_fence(Ordering::Acquire);

        let inner = self
            .inner
            .take()
            .unwrap_or_else(|| unsafe { hint::unreachable_unchecked() });
        (inner.buffer, inner.serial)
    }
}

impl<B> Drop for Transfer<B> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.as_mut() {
            // NOTE: this is a volatile write
            inner.serial.dma.stop();

            // we need a read here to make the Acquire fence effective
            // we do *not* need this if `dma.stop` does a RMW operation
            unsafe {
                ptr::read_volatile(&0);
            }

            // we need a fence here for the same reason we need one in `Transfer.wait`
            atomic::compiler_fence(Ordering::Acquire);
        }
    }
}

impl Serial1 {
    /// Receives data into the given `buffer` until it's filled
    ///
    /// Returns a value that represents the in-progress DMA transfer
    pub fn read_exact<B>(mut self, mut buffer: Pin<B>) -> Transfer<B>
    where
        B: DerefMut + 'static,
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
            inner: Some(Inner {
                buffer,
                serial: self,
            }),
        }
    }

    /// Sends out the given `buffer`
    ///
    /// Returns a value that represents the in-progress DMA transfer
    pub fn write_all<B>(mut self, buffer: Pin<B>) -> Transfer<B>
    where
        B: Deref + 'static,
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
            inner: Some(Inner {
                buffer,
                serial: self,
            }),
        }
    }
}

#[allow(dead_code, unused_mut, unused_variables)]
fn reuse(serial: Serial1) {
    let buf = Pin::new(Box::new([0; 16]));

    let t = serial.read_exact(buf); // compiler_fence(Ordering::Release) ▲

    // ..

    // this stops the DMA transfer and frees memory
    mem::drop(t); // compiler_fence(Ordering::Acquire) ▼

    // this likely reuses the previous memory allocation
    let mut buf = Box::new([0; 16]);

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
