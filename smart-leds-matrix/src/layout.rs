//! LED layout.

use core::marker::PhantomData;

use embedded_graphics_core::geometry::{Point, Size};
use invert_axis::{InvertX, InvertXY, InvertY, NoInvert};

/// Trait that represents a certain type of LED matrix.
///
/// The map() function shall fix any x y coordinate mismatch. Mismatch means
/// the matrix might display the result being drawn in mirrored or otherwise
/// incorrect ways due to the LEDs order on the PCB.
/// Grid type matrixes (like 2x2  of 1x4 grid of 8x8 matrixes) should be also
/// handled using this trait.
pub trait Layout {
    fn map(&self, p: Point) -> Option<usize>;
    fn size(&self) -> Size;
}

/// Rectangular LED matrix.
pub struct Rectangular<I> {
    size: Size,
    invert_axis: PhantomData<I>,
}

impl<I> Rectangular<I> {
    const fn new_common(width: u32, height: u32) -> Self {
        Self {
            size: Size::new(width, height),
            invert_axis: PhantomData,
        }
    }
}

impl Rectangular<NoInvert> {
    /// Creates a new rectangular layout.
    pub const fn new(width: u32, height: u32) -> Self {
        Self::new_common(width, height)
    }
}

impl Rectangular<InvertX> {
    /// Creates a new rectangular layout with inverted X axis.
    pub const fn new_invert_x(width: u32, height: u32) -> Self {
        Self::new_common(width, height)
    }
}

impl Rectangular<InvertY> {
    /// Creates a new rectangular layout with inverted Y axis.
    pub const fn new_invert_y(width: u32, height: u32) -> Self {
        Self::new_common(width, height)
    }
}

impl Rectangular<InvertXY> {
    /// Creates a new rectangular layout with inverted X and Y axis.
    pub const fn new_invert_xy(width: u32, height: u32) -> Self {
        Self::new_common(width, height)
    }
}

macro_rules! impl_layout {
    ($invert_type:ty, $invert_x:expr, $invert_y:expr) => {
        impl Layout for Rectangular<$invert_type> {
            fn map(&self, mut p: Point) -> Option<usize> {
                if $invert_x {
                    p.x = (self.size.width - 1) as i32 - p.x;
                }
                if $invert_y {
                    p.y = (self.size.height - 1) as i32 - p.y;
                }

                (p.x >= 0
                    && p.y >= 0
                    && p.x < self.size.width as i32
                    && p.y < self.size.height as i32)
                    .then(|| p.y as usize * self.size.width as usize + p.x as usize)
            }

            fn size(&self) -> Size {
                return self.size;
            }
        }
    };
}

impl_layout!(NoInvert, false, false);
impl_layout!(InvertX, true, false);
impl_layout!(InvertY, false, true);
impl_layout!(InvertXY, true, true);

/// Marker types for axis inversion.
pub mod invert_axis {
    /// No inverted axis.
    pub enum NoInvert {}

    /// Inverted X axis.
    pub enum InvertX {}

    /// Inverted Y axis.
    pub enum InvertY {}

    /// Inverted X and Y axis.
    pub enum InvertXY {}
}
