//! # Smart Leds Matrix
//!
//! This is a library that adapts [smart-leds](https://crates.io/crates/smart-leds) driver implementations to the
//! [embedded-graphics](https://docs.rs/embedded-graphics/latest/embedded_graphics/) crate by wrapping the LED
//! driver into a `Drawable` display target.
//!

#![no_std]

use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::{Rgb888, RgbColor},
    Pixel,
};

use smart_leds::{brightness, gamma, hsv::RGB8, SmartLedsWrite};

pub mod layout;
use layout::Layout;

/// The wrapper for the LED driver.
///
/// This receives the `SmartLedsWriter` trait implementations along with a
/// `Transformation` that describes the pixels mapping between the LED
/// strip placement and the matrix's x y coordinates.
pub struct SmartLedMatrix<L, const N: usize> {
    layout: L,
    content: [RGB8; N],
    brightness: u8,
}

impl<L, const N: usize> SmartLedMatrix<L, N> {
    pub fn set_brightness(&mut self, new_brightness: u8) {
        self.brightness = new_brightness;
    }

    pub fn brightness(&self) -> u8 {
        self.brightness
    }
}

impl<L: Layout, const N: usize> SmartLedMatrix<L, N> {
    pub fn new(layout: L) -> Self {
        Self {
            layout,
            content: [RGB8::default(); N],
            brightness: 255,
        }
    }

    pub fn flush<T: SmartLedsWrite>(&mut self, writer: &mut T) -> Result<(), T::Error>
    where
        <T as SmartLedsWrite>::Color: From<RGB8>,
    {
        let iter = brightness(self.content.as_slice().iter().cloned(), self.brightness);
        writer.write(iter)
    }

    pub fn flush_with_gamma<T: SmartLedsWrite>(&mut self, writer: &mut T) -> Result<(), T::Error>
    where
        <T as SmartLedsWrite>::Color: From<RGB8>,
    {
        let iter = brightness(
            gamma(self.content.as_slice().iter().cloned()),
            self.brightness,
        );
        writer.write(iter)
    }
}

impl<L: Layout, const N: usize> DrawTarget for SmartLedMatrix<L, N> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Rgb888>>,
    {
        for Pixel(pos, color) in pixels {
            if let Some(t) = self
                .layout
                .map(pos)
                .and_then(|index| self.content.get_mut(index))
            {
                *t = RGB8::new(color.r(), color.g(), color.b());
            }
        }

        Ok(())
    }
}

impl<L: Layout, const N: usize> OriginDimensions for SmartLedMatrix<L, N> {
    fn size(&self) -> Size {
        self.layout.size()
    }
}
