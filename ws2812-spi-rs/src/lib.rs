//! # Use ws2812 leds via spi
//!
//! - For usage with `smart-leds`
//! - Implements the `SmartLedsWrite` trait
//!
//! Needs a type implementing the `spi::FullDuplex` trait.
//!
//! The spi peripheral should run at 2MHz to 3.8 MHz

// Timings for ws2812 from https://cpldcpu.files.wordpress.com/2014/01/ws2812_timing_table.png
// Timings for sk6812 from https://cpldcpu.wordpress.com/2016/03/09/the-sk6812-another-intelligent-rgb-led/

#![no_std]

use embedded_hal as hal;

pub mod prerendered;

use hal::spi::{Mode, Phase, Polarity, SpiBus};

use core::marker::PhantomData;

use smart_leds_trait::{SmartLedsWrite, RGB8, RGBW};

/// SPI mode that can be used for this crate
///
/// Provided for convenience
/// Doesn't really matter
pub const MODE: Mode = Mode {
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnFirstTransition,
};

pub mod devices {
    pub struct Ws2812;
    pub struct Sk6812w;
}

pub struct Ws2812<SPI, DEVICE = devices::Ws2812> {
    spi: SPI,
    device: PhantomData<DEVICE>,
}

impl<SPI> Ws2812<SPI>
where
    SPI: SpiBus,
{
    /// Use ws2812 devices via spi
    ///
    /// The SPI bus should run within 2 MHz to 3.8 MHz
    ///
    /// You may need to look at the datasheet and your own hal to verify this.
    ///
    /// Please ensure that the mcu is pretty fast, otherwise weird timing
    /// issues will occur
    pub fn new(spi: SPI) -> Self {
        Self {
            spi,
            device: PhantomData {},
        }
    }
}

impl<SPI> Ws2812<SPI, devices::Sk6812w>
where
    SPI: SpiBus,
{
    /// Use sk6812w devices via spi
    ///
    /// The SPI bus should run within 2.3 MHz to 3.8 MHz at least.
    ///
    /// You may need to look at the datasheet and your own hal to verify this.
    ///
    /// Please ensure that the mcu is pretty fast, otherwise weird timing
    /// issues will occur
    // The spi frequencies are just the limits, the available timing data isn't
    // complete
    pub fn new_sk6812w(spi: SPI) -> Self {
        Self {
            spi,
            device: PhantomData {},
        }
    }
}

impl<SPI, D> Ws2812<SPI, D>
where
    SPI: SpiBus,
{
    /// Write a single byte for ws2812 devices
    fn write_byte(&mut self, mut data: u8) -> Result<(), SPI::Error> {
        // Send two bits in one spi byte. High time first, then the low time
        // The maximum for T0H is 500ns, the minimum for one bit 1063 ns.
        // These result in the upper and lower spi frequency limits
        let patterns: [u8; 4] = [0b1000_1000, 0b1000_1110, 0b11101000, 0b11101110];
        for _ in 0..4 {
            let bits = (data & 0b1100_0000) >> 6;
            self.spi.write(&[patterns[bits as usize]])?;
            self.spi.read(&mut [0; 2]).ok();
            data <<= 2;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), SPI::Error> {
        // Should be > 300μs, so for an SPI Freq. of 3.8MHz, we have to send at least 1140 low bits or 140 low bytes
        for _ in 0..140 {
            self.spi.write(&[0x00])?;
            self.spi.read(&mut [0; 2]).ok();
        }
        Ok(())
    }
}

impl<SPI> SmartLedsWrite for Ws2812<SPI>
where
    SPI: SpiBus,
{
    type Error = SPI::Error;
    type Color = RGB8;
    /// Write all the items of an iterator to a ws2812 strip
    fn write<T, I>(&mut self, iterator: T) -> Result<(), SPI::Error>
    where
        T: IntoIterator<Item = I>,
        I: Into<Self::Color>,
    {
        // We introduce an offset in the fifo here, so there's always one byte in transit
        // Some MCUs (like the stm32f1) only a one byte fifo, which would result
        // in overrun error if two bytes need to be stored
        self.spi.write(&[0x00])?;
        if cfg!(feature = "mosi_idle_high") {
            self.flush()?;
        }

        for item in iterator {
            let item = item.into();
            self.write_byte(item.g)?;
            self.write_byte(item.r)?;
            self.write_byte(item.b)?;
        }
        self.flush()?;
        // Now, resolve the offset we introduced at the beginning
        self.spi.read(&mut [0; 2])?;
        Ok(())
    }
}

impl<SPI> SmartLedsWrite for Ws2812<SPI, devices::Sk6812w>
where
    SPI: SpiBus,
{
    type Error = SPI::Error;
    type Color = RGBW<u8, u8>;
    /// Write all the items of an iterator to a ws2812 strip
    fn write<T, I>(&mut self, iterator: T) -> Result<(), SPI::Error>
    where
        T: IntoIterator<Item = I>,
        I: Into<Self::Color>,
    {
        // We introduce an offset in the fifo here, so there's always one byte in transit
        // Some MCUs (like the stm32f1) only a one byte fifo, which would result
        // in overrun error if two bytes need to be stored
        self.spi.write(&[0x00])?;
        if cfg!(feature = "mosi_idle_high") {
            self.flush()?;
        }

        for item in iterator {
            let item = item.into();
            self.write_byte(item.g)?;
            self.write_byte(item.r)?;
            self.write_byte(item.b)?;
            self.write_byte(item.a.0)?;
        }
        self.flush()?;
        // Now, resolve the offset we introduced at the beginning
        self.spi.read(&mut [0; 2])?;
        Ok(())
    }
}
