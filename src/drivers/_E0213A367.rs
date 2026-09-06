/// provide logging primitives
use defmt_or_log::*;
const TAG: &str = "[E0213A367]";

// /// provide command protocol
// mod commands;
// use commands::*;

/// provide display_interface primitives
use display_interface::DisplayError;
use display_interface::DataFormat;

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
        trace!("{TAG} configuring screen controller...");
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

        trace!("{TAG} configured screen controller");
        Ok(())
    }

    /// POST: send b/w buffer and then red buffer
    pub async fn start_update(&mut self) -> Result<(), DisplayError> {
        trace!("{TAG} starting update...");
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
        trace!("{TAG} started update");
        Ok(())
    }

    pub async fn finish_update_and_refresh(&mut self) -> Result<(), DisplayError> {
        trace!("{TAG} starting refresh...");
        // start refresh
        self.epd_interface
            .send_commands(DataFormat::U8(&[0x20]))
            .await?;
        self.epd_interface.wait_until_idle().await?;

        trace!("{TAG} refresh completed");
        Ok(())
    }
}


#[maybe_async_cfg::maybe(
    sync(keep_self, cfg(not(feature = "async"))),
    async(keep_self, feature = "async"),
    idents(
        AsyncWriteOnlyDataCommand(async, sync = "WriteOnlyDataCommand"),
        AsyncWaitUntilIdle(async, sync = "WaitUntilIdle"),
        dimensions(keep),
        refresh(keep),
    )
)]
impl<DI> crate::graphics::EpdDriver for E0213A367<DI>
where
    DI: display_interface::AsyncWriteOnlyDataCommand + crate::interface::AsyncWaitUntilIdle,
{
    fn dimensions(&self) -> embedded_graphics::geometry::Size
    {
        self.dimensions
    }

    async fn refresh(
        &mut self,
        frame_buffer: &fixedbitset::FixedBitSet,
    ) -> Result<(), DisplayError>
    {
        self.start_update().await?;

        // update B/W (fast mode OFF)
        trace!("{TAG} updating B/W buffer...");
        self.epd_interface
            .send_commands(DataFormat::U8(&[0x24]))
            .await?;
        self.epd_interface
            .send_data(DataFormat::U8(&[0xFF; ((122 / 8) * 255)]))
            .await?;
        // for block in frame_buffer.as_slice() {
        //     let buffer = block.reverse_bits().to_be_bytes();
        //     self.epd_interface.send_data(DataFormat::U8(&buffer[..])).await?
        // }
        trace!("{TAG} updated B/W buffer");

        // update RED (fast mode OFF)
        trace!("{TAG} updating RED buffer...");
        self.epd_interface
            .send_commands(DataFormat::U8(&[0x26]))
            .await?;
        self.epd_interface
            .send_data(DataFormat::U8(&[0xFF; ((122 / 8) * 255)]))
            .await?;
        // for block in frame_buffer.as_slice() {
        //     let buffer = block.reverse_bits().to_be_bytes();
        //     self.epd_interface.send_data(DataFormat::U8(&buffer[..])).await?
        // }
        trace!("{TAG} updated RED buffer");

        self.finish_update_and_refresh().await?;
        Ok(())
    }

}
