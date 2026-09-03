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
pub struct EpdInterface<SPI, DC, NBUSY, NRESET>
{
    spi_interface: display_interface_spi::SPIInterface<SPI, DC>,
    /// low while busy
    n_busy: NBUSY,
    /// low to hold in reset
    n_reset: NRESET,
}
impl<SPI, DC, NBUSY, NRESET> EpdInterface<SPI, DC, NBUSY, NRESET>
where
    SPI: SpiDevice,
    NBUSY: InputPin,
    NRESET: OutputPin,
{
    pub fn new(
        spi_interface: display_interface_spi::SPIInterface<SPI, DC>,
        n_busy: NBUSY,
        n_reset: NRESET,
    ) -> Self {
        Self { spi_interface, n_busy, n_reset }
    }

    pub fn reset(&mut self, delay: &mut impl DelayNs)
    {
        self.n_reset.set_low().unwrap();
        delay.delay_ms(10);
        self.n_reset.set_high().unwrap();
        delay.delay_ms(10);
    }

    pub fn wait_until_idle(&mut self, delay: &mut impl DelayNs) -> Result<(), DisplayError>
    {
        for _ in 0..3
        {
            if self.n_busy.is_high().expect("failed to read busy pin")
            {
                return Ok(())
            }
            delay.delay_ms(10);
        }

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
impl<SPI, DC, NBUSY, NRESET> AsyncWriteOnlyDataCommand for EpdInterface<SPI, DC, NBUSY, NRESET>
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

