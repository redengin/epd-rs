/// provide embedded graphics primitives
use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::BinaryColor;

/// use standard display errors
use display_interface::DisplayError;
use embedded_graphics::primitives::Rectangle;

pub struct EpdDrawTarget<DRIVER> {
    driver: DRIVER,
    frame_buffer: fixedbitset::FixedBitSet,
    rotation: DisplayRotation,
}

#[maybe_async_cfg::maybe(
    sync(keep_self, cfg(not(feature = "async"))),
    async(keep_self, feature = "async")
)]
impl<DRIVER> EpdDrawTarget<DRIVER>
where
    DRIVER: EpdDriver,
{
    pub fn new(driver: DRIVER, rotation: DisplayRotation) -> Self {
        let dimensions = driver.dimensions();
        let width = dimensions.width;
        let height = dimensions.height;
        Self {
            driver,
            frame_buffer: fixedbitset::FixedBitSet::with_capacity(((width / 8) * height) as usize),
            rotation,
        }
    }

    pub fn frame_buffer_bit(&self, point: embedded_graphics::geometry::Point) -> Option<usize> {
        let dimensions = self.driver.dimensions();
        let width = dimensions.width as i32;
        let height = dimensions.height as i32;
        let translated_point = match self.rotation {
            DisplayRotation::Rotate0 => point,
            DisplayRotation::Rotate90 => Point::new(point.y, point.x),
            DisplayRotation::Rotate180 => Point::new(width - point.x, height - point.y),
            DisplayRotation::Rotate270 => Point::new(height - point.x, width - point.y),
        };

        return if (translated_point.x < 0)
            || (translated_point.y < 0)
            || (translated_point.x > width)
            || (translated_point.y > height)
        {
            None
        } else {
            Some((translated_point.x + (translated_point.y * width)) as usize)
        };
    }

    pub async fn refresh(&mut self) -> Result<(), DisplayError> {
        self.driver.refresh(&self.frame_buffer).await
    }
}

impl<DRIVER> embedded_graphics::draw_target::DrawTarget for EpdDrawTarget<DRIVER>
where
    DRIVER: EpdDriver,
{
    type Color = BinaryColor;

    type Error = DisplayError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::prelude::Pixel<Self::Color>>,
    {
        for pixel in pixels.into_iter() {
            let Pixel(point, color) = pixel;
            match self.frame_buffer_bit(point) {
                None => { /* ignore pixels outside of dimensions */ }
                Some(bit) => match color {
                    BinaryColor::Off => self.frame_buffer.remove(bit),
                    BinaryColor::On => self.frame_buffer.insert(bit),
                },
            }
        }
        Ok(())
    }

    /// provide optimized function to clear display
    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        match color {
            BinaryColor::Off => self.frame_buffer.clear(),
            BinaryColor::On => self.frame_buffer.insert_range(..),
        }
        Ok(())
    }
}

impl<DRIVER> embedded_graphics::geometry::Dimensions for EpdDrawTarget<DRIVER>
where
    DRIVER: EpdDriver,
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

#[maybe_async_cfg::maybe(
    sync(keep_self, cfg(not(feature = "async"))),
    async(keep_self, feature = "async")
)]
pub trait EpdDriver {
    fn dimensions(&self) -> Size;

    async fn refresh(
        &mut self,
        frame_buffer: &fixedbitset::FixedBitSet,
    ) -> Result<(), DisplayError>;
}

/// Display rotation.
// #[derive(Copy, Clone, Debug)]
pub enum DisplayRotation {
    /// No rotation, normal display
    Rotate0,
    /// Rotate by 90 degrees clockwise
    Rotate90,
    /// Rotate by 180 degrees clockwise, upside down display
    Rotate180,
    /// Rotate 270 degrees clockwise
    Rotate270,
}

