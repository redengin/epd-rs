/// use standard display errors
use display_interface::DisplayError;
use embedded_graphics::primitives::Rectangle;

/// provide embedded graphics primitives
use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::BinaryColor;

/// Display rotation.
pub enum DisplayRotation {
    /// No rotation, normal display
    Rotate0,
    /// Rotate by 90 degrees clockwise
    Rotate90,
    /// Rotate by 180 degrees clockwise
    Rotate180,
    /// Rotate 270 degrees clockwise
    Rotate270,
}

pub struct EpdDisplay<DRIVER> {
    driver: DRIVER,
    rotation: DisplayRotation,
    frame_buffer: fixedbitset::FixedBitSet,
}
#[maybe_async_cfg::maybe(
    sync(keep_self, cfg(not(feature = "async"))),
    async(keep_self, feature = "async")
)]
impl<DRIVER> EpdDisplay<DRIVER>
where
    DRIVER: crate::drivers::EpdDriver,
{
    pub fn new(driver: DRIVER, rotation: DisplayRotation) -> Self {
        let dimensions = driver.dimensions();
        let pixel_count = dimensions.width * dimensions.height;
        Self {
            driver,
            rotation,
            frame_buffer: fixedbitset::FixedBitSet::with_capacity(pixel_count as usize),
        }
    }

    /// perform a hardware reset and initialize the driver
    pub async fn init(&mut self) -> Result<(), display_interface::DisplayError>
    {
        self.driver.init().await
    }

    /// refresh the screen
    pub async fn refresh(&mut self) -> Result<(), DisplayError> {
        self.driver.refresh(&self.frame_buffer).await
    }

    fn frame_buffer_bit(&self, point: embedded_graphics::geometry::Point) -> Option<usize> {
        let dimensions = self.driver.dimensions();
        let width = dimensions.width as i32;
        let height = dimensions.height as i32;
        let translated_point = match self.rotation {
            DisplayRotation::Rotate0 => point,
            DisplayRotation::Rotate90 => Point::new(width - point.y, point.x),
            DisplayRotation::Rotate180 => Point::new(width - point.x, height - point.y),
            DisplayRotation::Rotate270 => Point::new(point.y, height - point.x),
        };

        return if (translated_point.x < 0)
            || (translated_point.y < 0)
            || (translated_point.x >= width)
            || (translated_point.y >= height)
        {
            None
        } else {
            Some((translated_point.x + (translated_point.y * width)) as usize)
        };
    }
}

/// provide embedded_graphics DrawTarget
impl<DRIVER> embedded_graphics::draw_target::DrawTarget for EpdDisplay<DRIVER>
where
    DRIVER: crate::drivers::EpdDriver,
{
    type Color = BinaryColor;

    type Error = DisplayError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::prelude::Pixel<Self::Color>>,
    {
        for pixel in pixels.into_iter() {
            let Pixel(point, color) = pixel;
            if let Some(bit) = self.frame_buffer_bit(point) {
                match color {
                    BinaryColor::On => self.frame_buffer.remove(bit),
                    BinaryColor::Off => self.frame_buffer.insert(bit),
                };
            };
        }
        Ok(())
    }

    /// provide optimized function to clear display
    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        match color {
            BinaryColor::On => self.frame_buffer.clear(),
            BinaryColor::Off => self.frame_buffer.insert_range(..),
        }
        Ok(())
    }
}

/// required support for embedded_graphics DrawTarget
impl<DRIVER> embedded_graphics::geometry::Dimensions for EpdDisplay<DRIVER>
where
    DRIVER: crate::drivers::EpdDriver,
{
    fn bounding_box(&self) -> Rectangle {
        let dimensions = self.driver.dimensions();
        return match self.rotation {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => {
                Rectangle::new(Point::zero(), dimensions)
            }

            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => Rectangle::new(
                Point::zero(),
                Size::new(dimensions.height, dimensions.width),
            ),
        };
    }
}
