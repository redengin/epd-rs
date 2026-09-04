/// provide logging primitives
use defmt_or_log::*;

/// use standard display errors
use display_interface::DisplayError;
/// provide embedded-hal abstractions
use embedded_hal::{delay::DelayNs, digital::{InputPin, OutputPin}};

/// choose async abstraction
#[cfg(not(feature = "async"))]
use embedded_hal::spi::SpiDevice;
#[cfg(feature = "async")]
use embedded_hal_async::spi::SpiDevice;

#[allow(dead_code)]
pub struct EpdInterface<SPI, DC, NBUSY, NRESET, DELAY>
{
    spi_interface: display_interface_spi::SPIInterface<SPI, DC>,
    /// low while busy
    n_busy: NBUSY,
    /// low to hold in reset
    n_reset: NRESET,
    delay: DELAY,
}
impl<SPI, DC, NBUSY, NRESET, DELAY> EpdInterface<SPI, DC, NBUSY, NRESET, DELAY>
where
    SPI: SpiDevice,
    NBUSY: InputPin,
    NRESET: OutputPin,
    DELAY: embedded_hal::delay::DelayNs
{
    pub fn new(
        spi_interface: display_interface_spi::SPIInterface<SPI, DC>,
        n_busy: NBUSY,
        n_reset: NRESET,
        delay: DELAY,
    ) -> Self
    {
        let mut this = Self { spi_interface, n_busy, n_reset, delay };
        this.reset();
        this
    }

    pub fn reset(&mut self)
    {
        self.n_reset.set_low().unwrap();
        self.delay.delay_ms(100);
        self.n_reset.set_high().unwrap();
        self.delay.delay_ms(100);
    }
}

impl<SPI, DC, NBUSY, NRESET, DELAY> crate::WaitUntilIdle for EpdInterface<SPI, DC, NBUSY, NRESET, DELAY>
 where
    SPI: SpiDevice,
    NBUSY: InputPin,
    NRESET: OutputPin,
    DELAY: embedded_hal::delay::DelayNs
{
    fn wait_until_idle(&mut self) -> Result<(), DisplayError>
    {
        for _ in 0..4
        {
            if self.n_busy.is_low().expect("failed to read busy pin")
            {
                info!("idle asserted");
                return Ok(())
            }
            self.delay.delay_ms(500);
        }

        error!("idle not asserted");
        Err(display_interface::DisplayError::RSError)
    }
}

// Provide display_interface_spi primitives
//------------------------------------------------------------------------------
#[cfg(not(feature = "async"))]
use display_interface::WriteOnlyDataCommand;
#[cfg(feature = "async")]
use display_interface::AsyncWriteOnlyDataCommand;

use display_interface::DataFormat;

#[maybe_async_cfg::maybe(
    sync(keep_self, cfg(not(feature="async"))),
    async(keep_self, feature="async"),
    idents(
        AsyncWriteOnlyDataCommand(async, sync = "WriteOnlyDataCommand"),
    )
)]
impl<SPI, DC, NBUSY, NRESET, DELAY> AsyncWriteOnlyDataCommand for EpdInterface<SPI, DC, NBUSY, NRESET, DELAY>
where
    display_interface_spi::SPIInterface<SPI, DC>: AsyncWriteOnlyDataCommand,
{
    async fn send_commands(&mut self, data: DataFormat<'_>) -> Result<(), DisplayError>
    {
        self.spi_interface.send_commands(data).await
    }

    async fn send_data(&mut self, data: DataFormat<'_>) -> Result<(), DisplayError>
    {
        self.spi_interface.send_data(data).await
    }
}

