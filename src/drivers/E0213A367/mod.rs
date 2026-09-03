// /// provide command protocol
// mod commands;
// use commands::*;

/// provide display_interface primitives
use display_interface::DataFormat;

#[allow(non_camel_case_types)]
#[allow(dead_code)]

pub struct E0213A367<DI> {
    epd_interface: DI,
    // width and height in pixels
    dimensions: embedded_graphics::geometry::Size,
    rotation: crate::DisplayRotation,
    frame_buffer: fixedbitset::FixedBitSet,
}

#[maybe_async_cfg::maybe(
    sync(keep_self, cfg(not(feature = "async"))),
    async(keep_self, feature = "async"),
    idents(
        AsyncWriteOnlyDataCommand(async, sync = "WriteOnlyDataCommand"),
        init(keep),
        flush(keep),
    )
)]
impl<DI> E0213A367<DI>
where
    DI: display_interface::AsyncWriteOnlyDataCommand,
{
    pub async fn new(
        epd_interface: DI,
        rotation: crate::DisplayRotation,
    ) -> Result<Self, display_interface::DisplayError> {
        let mut this = Self {
            epd_interface,
            dimensions: embedded_graphics::geometry::Size::new(255, 122),
            rotation,
            frame_buffer: fixedbitset::FixedBitSet::with_capacity(
                ((255/8) * 122) as usize,
            ),
        };

        // initialize the hardware
        this.init().await?;

        Ok(this)
    }

    pub async fn init(&mut self) -> Result<(), display_interface::DisplayError> {
        // // power setting
        // self.epd_interface
        //     .send_commands(DataFormat::U8(&[0x01]))
        //     .await?;
        // self.epd_interface
        //     .send_data(DataFormat::U8(&[0x03, 0x00, 0x2b, 0x2b, 0x03]))
        //     .await?;

        Ok(())
    }

    pub async fn flush(&mut self) -> Result<(), display_interface::DisplayError> {
        Ok(())
    }
}
