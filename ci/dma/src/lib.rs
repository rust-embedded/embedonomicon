#![allow(unused_variables)]

pub const USART1_TX: usize = 0x4000_0000;
pub const USART1_RX: usize = 0x4000_0004;

/// A singleton that represents a single DMA channel (channel 1 in this case)
///
/// This singleton has exclusive access to the registers of the DMA channel 1
pub struct Dma1Channel1 {
    // ..
}

impl Dma1Channel1 {
    /// Data will be written to this `address`
    ///
    /// `inc` indicates whether the address will be incremented after every byte
    /// transfer
    ///
    /// NOTE this performs a volatile write
    pub fn set_destination_address(&mut self, address: usize, inc: bool) {
        // ..
    }

    /// Data will be read from this `address`
    ///
    /// `inc` indicates whether the address will be incremented after every byte
    /// transfer
    ///
    /// NOTE this performs a volatile write
    pub fn set_source_address(&mut self, address: usize, inc: bool) {
        // ..
    }

    /// Number of bytes to transfer
    ///
    /// NOTE this performs a volatile write
    pub fn set_transfer_length(&mut self, len: usize) {
        // ..
    }

    /// Starts the DMA transfer
    ///
    /// NOTE this performs a volatile write
    pub fn start(&mut self) {
        // ..
    }

    /// Stops the DMA transfer
    ///
    /// NOTE this performs a volatile write
    pub fn stop(&mut self) {
        // ..
    }

    /// Returns `true` if there's a transfer in progress
    ///
    /// NOTE this performs a volatile read
    pub fn in_progress() -> bool {
        // ..
        false
    }
}

/// A singleton that represents serial port #1
pub struct Serial1 {
    // ..
}

impl Serial1 {
    /// Reads out a single byte
    ///
    /// NOTE: blocks if no byte is available to be read
    pub fn read(&mut self) -> Result<u8, Error> {
        // ..
        Ok(0)
    }

    /// Sends out a single byte
    ///
    /// NOTE: blocks if the output FIFO buffer is full
    pub fn write(&mut self, byte: u8) -> Result<(), Error> {
        // ..
        Ok(())
    }
}

pub enum Error {}
