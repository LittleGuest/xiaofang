# Smart LEDs matrix
![](pacman.gif)

A `DrawTarget` implementation to use (one, or more) smart LED matrixes as a graphics display driven by [embedded-graphics](https://docs.rs/embedded-graphics/latest/embedded_graphics/) `Drawable` objects.
The integrated driver is from [smart-leds](https://docs.rs/smart-leds/latest/smart_leds/) crate.

# Known issues (with my setup: stm32f401 + 8x8 ws2812 matrix):
* circles with the same parameters are not exactly drawn always to the same position, not sure if this is the same with bigger resolution displays or not
* write operation usually gets back with an overrun error, while the display is still updated for ~every second time (workaround: flush always twice)

# Plan
* Add more display types (like 2x2 or 1x4 grids of 8x8 matrixes), though user can add those anytime by implementing another `layout`.

# Usage
You may start by creating a driver for your LED and controller. Some examples can be found [here](https://github.com/smart-leds-rs/smart-leds-samples).

Once you have it, you can plug it into the `DrawTarget` implemented by this crate.

Example:
```rust
use ws2812_spi as ws2812;

use smart_leds_matrix::{SmartLedMatrix, layout::Rectangular};

use embedded_graphics::{
    pixelcolor::*,
    prelude::*,
    primitives::{
        PrimitiveStyleBuilder, Rectangle,
    },
};

fn main() -> ! {
[...]
    let ws = ws2812::Ws2812::new(spi);
    let mut matrix = SmartLedMatrix::<_, _, {8 * 8}>::new(ws, Rectangular::new_inverted_y(8, 8));
    matrix.set_brightness(15);
    matrix.clear(Rgb888::new(0, 0, 0));

    // Drawable objects are calling draw_iter() function of the matrix.
    // That is, only the internal frame buffer is updated, no real 
    // communication happens yet. That is useful when a frame is composed
    // of multiple objects, text, etc.
    Rectangle::new(Point::new(1, 1), Size::new(6, 6))
    .into_styled(
        PrimitiveStyleBuilder::new()
        .fill_color(Rgb888::RED)
        .build(),
    ).draw(&mut matrix)?;
    // Trigger the actual frame update on the matrix with gamma correction.
    matrix.flush_with_gamma();
    loop{}
}
```
