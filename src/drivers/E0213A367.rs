// /// provide command protocol
// mod commands;
// use commands::*;

use display_interface::DataFormat;
/// provide display_interface primitives
use display_interface::DisplayError;

#[allow(non_camel_case_types)]
#[allow(dead_code)]

pub struct E0213A367<DI> {
    epd_interface: DI,
    // width and height in pixels
    dimensions: embedded_graphics::geometry::Size,
}

#[maybe_async_cfg::maybe(
    sync(keep_self, cfg(not(feature = "async"))),
    async(keep_self, feature = "async"),
    idents(
        AsyncWriteOnlyDataCommand(async, sync = "WriteOnlyDataCommand"),
        AsyncWaitUntilIdle(async, sync = "WaitUntilIdle"),
        init(keep),
        update(keep),
    )
)]
impl<DI> E0213A367<DI>
where
    DI: display_interface::AsyncWriteOnlyDataCommand + crate::interface::AsyncWaitUntilIdle,
{
    pub async fn new(
        epd_interface: DI,
        dimensions: embedded_graphics::geometry::Size,
    ) -> Result<Self, DisplayError> {
        let mut this = Self {
            epd_interface,
            dimensions,
        };

        // initialize the hardware
        this.init().await?;

        Ok(this)
    }

    pub async fn init(&mut self) -> Result<(), DisplayError> {
        // set display option to FULLSCREEN
        self.epd_interface
            .send_commands(DataFormat::U8(&[0x37]))
            .await?;
        self.epd_interface
            .send_data(DataFormat::U8(&[0x00, 0x80, 0x03, 0x0E]))
            .await?;

        // set border waveform
        self.epd_interface
            .send_commands(DataFormat::U8(&[0x3C]))
            .await?;
        self.epd_interface
            .send_data(DataFormat::U8(&[0x01]))
            .await?;
        self.epd_interface.wait_until_idle().await?;

        // configure data entry mode
        self.epd_interface
            .send_commands(DataFormat::U8(&[0x11]))
            .await?;
        self.epd_interface
            .send_data(DataFormat::U8(&[0x03]))
            .await?;

        // select memory region X
        self.epd_interface
            .send_commands(DataFormat::U8(&[0x44]))
            .await?;
        self.epd_interface
            .send_data(DataFormat::U8(&[0x00, (self.dimensions.width / 8) as u8]))
            .await?;
        // select memory region Y
        self.epd_interface
            .send_commands(DataFormat::U8(&[0x45]))
            .await?;
        self.epd_interface
            .send_data(DataFormat::U8(&[0x00, self.dimensions.height as u8]))
            .await?;

        // configure normal mode
        self.epd_interface
            .send_commands(DataFormat::U8(&[0x22]))
            .await?;
        self.epd_interface
            .send_data(DataFormat::U8(&[0xF7]))
            .await?;
        // configure fast mode
        // self.epd_interface.send_commands(DataFormat::U8(&[0x22])).await?;
        // self.epd_interface.send_data(DataFormat::U8(&[0xFF])).await?;

        Ok(())
    }

    async fn update(
        &mut self,
        // FIXME
        _frame_buffer: &fixedbitset::FixedBitSet,
    ) -> Result<(), DisplayError> {
         // set the X cursor
        self.epd_interface
            .send_commands(DataFormat::U8(&[0x4E]))
            .await?;
        self.epd_interface
            .send_data(DataFormat::U8(&[0x00]))
            .await?;
        // set the Y cursor
        self.epd_interface
            .send_commands(DataFormat::U8(&[0x4F]))
            .await?;
        self.epd_interface
            .send_data(DataFormat::U8(&[0x00]))
            .await?;

        // update B/W (fast mode OFF)
        self.epd_interface
            .send_commands(DataFormat::U8(&[0x24]))
            .await?;
        self.epd_interface
            .send_data(DataFormat::U8(&[0xFF; ((122 / 8 + 1) * 255)]))
            .await?;
        // update B/W "red" (fast mode OFF)
        self.epd_interface
            .send_commands(DataFormat::U8(&[0x26]))
            .await?;
        self.epd_interface
            .send_data(DataFormat::U8(&[0xFF; ((122 / 8 + 1) * 255)]))
            .await?;

        // start refresh
        self.epd_interface
            .send_commands(DataFormat::U8(&[0x20]))
            .await?;
        self.epd_interface.wait_until_idle().await?;

        Ok(())
    }
}

/// provide embedded graphics primitives
use embedded_graphics::prelude::*;

#[maybe_async_cfg::maybe(
    sync(keep_self, cfg(not(feature = "async"))),
    async(keep_self, feature = "async"),
    idents(
        AsyncWriteOnlyDataCommand(async, sync = "WriteOnlyDataCommand"),
        AsyncWaitUntilIdle(async, sync = "WaitUntilIdle"),
        refresh(keep),
    )
)]
impl<DI> crate::graphics::EpdDriver for E0213A367<DI>
where
    DI: display_interface::AsyncWriteOnlyDataCommand + crate::interface::AsyncWaitUntilIdle,
{
    fn dimensions(&self) -> Size {
        self.dimensions
    }

    async fn refresh(
        &mut self,
        frame_buffer: &fixedbitset::FixedBitSet,
    ) -> Result<(), DisplayError> {
        self.update(frame_buffer).await
    }
}

// {
//     fn dimensions(&self) ->  embedded_graphics::prelude::Size {
//         self.dimensions
//     }
// }

// #[maybe_async_cfg::maybe(
//     sync(keep_self, cfg(not(feature = "async"))),
//     async(keep_self, feature = "async"),
//     idents(
//         AsyncWriteOnlyDataCommand(async, sync = "WriteOnlyDataCommand"),
//         flush(keep),
//     )
// )]
// impl<DI> crate::EpdDrawTarget for E0213A367<DI>
// where
//     DI: display_interface::AsyncWriteOnlyDataCommand + crate::WaitUntilIdle,
// {
//     // type Color = embedded_graphics::pixelcolor::BinaryColor;

//     async fn flush(&mut self) -> Result<(), display_interface::DisplayError>
//     {
//         self.refresh().await?;

//         Ok(())
//     }
// }
