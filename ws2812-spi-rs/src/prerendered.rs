//! This prerenders the data, so that no calculations have to be performed while sending the data.
//!
//! This approach minimizes timing issues, at the cost of much higher ram usage.
//! It also increases the needed time.

use embedded_hal as hal;

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

#[derive(Debug)]
pub enum Error<E> {
    OutOfBounds,
    Spi(E),
}

pub mod devices {
    pub struct Ws2812;
    pub struct Sk6812w;
}

pub struct Ws2812<'a, SPI, DEVICE = devices::Ws2812> {
    spi: SPI,
    data: &'a mut [u8],
    index: usize,
    device: PhantomData<DEVICE>,
}

impl<'a, SPI> Ws2812<'a, SPI>
where
    SPI: SpiBus,
{
    /// Use ws2812 devices via spi
    ///
    /// The SPI bus should run within 2 MHz to 3.8 MHz
    ///
    /// You may need to look at the datasheet and your own hal to verify this.
    ///
    /// You need to provide a buffer `data`, whoose length is at least 12 * the
    /// length of the led strip + 20 byes (or 40, if using the `mosi_idle_high` feature)
    ///
    /// Please ensure that the mcu is pretty fast, otherwise weird timing
    /// issues will occur
    pub fn new(spi: SPI, data: &'a mut [u8]) -> Self {
        Self {
            spi,
            data,
            index: 0,
            device: PhantomData {},
        }
    }
}

impl<'a, SPI> Ws2812<'a, SPI, devices::Sk6812w>
where
    SPI: SpiBus,
{
    /// Use sk6812w devices via spi
    ///
    /// The SPI bus should run within 2.3 MHz to 3.8 MHz at least.
    ///
    /// You may need to look at the datasheet and your own hal to verify this.
    ///
    /// You need to provide a buffer `data`, whoose length is at least 12 * the
    /// length of the led strip
    ///
    /// Please ensure that the mcu is pretty fast, otherwise weird timing
    /// issues will occur
    // The spi frequencies are just the limits, the available timing data isn't
    // complete
    pub fn new_sk6812w(spi: SPI, data: &'a mut [u8]) -> Self {
        Self {
            spi,
            data,
            index: 0,
            device: PhantomData {},
        }
    }
}

impl<'a, SPI, D> Ws2812<'a, SPI, D>
where
    SPI: SpiBus,
{
    /// Write a single byte for ws2812 devices
    fn write_byte(&mut self, mut data: u8) -> Result<(), Error<SPI::Error>> {
        // Send two bits in one spi byte. High time first, then the low time
        // The maximum for T0H is 500ns, the minimum for one bit 1063 ns.
        // These result in the upper and lower spi frequency limits
        let patterns = [0b1000_1000, 0b1000_1110, 0b11101000, 0b11101110];

        if self.index > self.data.len() - 4 {
            return Err(Error::OutOfBounds);
        }
        for _ in 0..4 {
            let bits = (data & 0b1100_0000) >> 6;
            self.data[self.index] = patterns[bits as usize];
            self.index += 1;
            data <<= 2;
        }
        Ok(())
    }

    fn send_data(&mut self) -> Result<(), SPI::Error> {
        // We introduce an offset in the fifo here, so there's always one byte in transit
        // Some MCUs (like the stm32f1) only a one byte fifo, which would result
        // in overrun error if two bytes need to be stored
        self.spi.write(&[0x00])?;
        if cfg!(feature = "mosi_idle_high") {
            for _ in 0..140 {
                self.spi.write(&[0x00])?;
                self.spi.read(&mut [0; 2])?;
            }
        }
        for b in self.data[..self.index].iter() {
            self.spi.write(&[*b])?;
            self.spi.read(&mut [0; 2])?;
        }
        for _ in 0..140 {
            self.spi.write(&[0x00])?;
            self.spi.read(&mut [0; 2])?;
        }
        // Now, resolve the offset we introduced at the beginning
        self.spi.read(&mut [0; 2])?;
        Ok(())
    }
}

impl<'a, SPI> SmartLedsWrite for Ws2812<'a, SPI>
where
    SPI: SpiBus,
{
    type Error = Error<SPI::Error>;
    type Color = RGB8;
    /// Write all the items of an iterator to a ws2812 strip
    fn write<T, I>(&mut self, iterator: T) -> Result<(), Error<SPI::Error>>
    where
        T: IntoIterator<Item = I>,
        I: Into<Self::Color>,
    {
        self.index = 0;

        for item in iterator {
            let item = item.into();
            self.write_byte(item.g)?;
            self.write_byte(item.r)?;
            self.write_byte(item.b)?;
        }
        self.send_data().map_err(|e| Error::Spi(e))
    }
}

impl<'a, SPI> SmartLedsWrite for Ws2812<'a, SPI, devices::Sk6812w>
where
    SPI: SpiBus,
{
    type Error = Error<SPI::Error>;
    type Color = RGBW<u8, u8>;
    /// Write all the items of an iterator to a ws2812 strip
    fn write<T, I>(&mut self, iterator: T) -> Result<(), Error<SPI::Error>>
    where
        T: IntoIterator<Item = I>,
        I: Into<Self::Color>,
    {
        self.index = 0;

        for item in iterator {
            let item = item.into();
            self.write_byte(item.g)?;
            self.write_byte(item.r)?;
            self.write_byte(item.b)?;
            self.write_byte(item.a.0)?;
        }
        self.send_data().map_err(|e| Error::Spi(e))
    }
}
