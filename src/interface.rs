/// provide logging primitives
use defmt_or_log::*;
const TAG: &str = "[EpdInterface]";

/// provide embedded-hal abstractions
use embedded_hal::digital::{InputPin, OutputPin};

/// choose SpiDevice abstraction
#[cfg(not(feature = "async"))]
use embedded_hal::spi::SpiDevice;
#[cfg(feature = "async")]
use embedded_hal_async::spi::SpiDevice;

/// choose Delay abstraction
#[cfg(not(feature = "async"))]
use embedded_hal::delay::DelayNs;
#[cfg(feature = "async")]
use embedded_hal_async::delay::DelayNs;

/// Hardware interface to an EPD
pub struct EpdInterface<SPI, DC, NBUSY, NRESET, DELAY> {
    pub spi_interface: display_interface_spi::SPIInterface<SPI, DC>,
    /// low while busy
    n_busy: NBUSY,
    /// low to hold in reset
    n_reset: NRESET,
    delay: DELAY,
}
#[maybe_async_cfg::maybe(
    sync(keep_self, cfg(not(feature = "async"))),
    async(keep_self, feature = "async"),
    idents(new(keep), reset(keep),)
)]
impl<SPI, DC, NBUSY, NRESET, DELAY> EpdInterface<SPI, DC, NBUSY, NRESET, DELAY>
where
    SPI: SpiDevice,
    NBUSY: InputPin,
    NRESET: OutputPin,
    DELAY: DelayNs,
{
    pub async fn new(
        spi_interface: display_interface_spi::SPIInterface<SPI, DC>,
        n_busy: NBUSY,
        n_reset: NRESET,
        delay: DELAY,
    ) -> Self {
        let mut this = Self {
            spi_interface,
            n_busy,
            n_reset,
            delay,
        };
        this.reset().await;
        this
    }

    /// perform a hardware reset of the EPD
    pub async fn reset(&mut self) {
        trace!("{TAG} performing hardware reset...");
        self.n_reset.set_low().unwrap();
        self.delay.delay_ms(10).await;
        self.n_reset.set_high().unwrap();
        self.delay.delay_ms(10).await;
        trace!("{TAG} hardware reset completed");
    }
}

#[maybe_async_cfg::maybe(
    sync(keep_self, cfg(not(feature = "async"))),
    async(keep_self, feature = "async"),
    idents(
        reset(keep),
        wait_until_idle(keep),
    )
)]
pub trait IEpdInterface {
    /// perform a hardware reset
    async fn reset(&mut self) -> Result<(), display_interface::DisplayError>;

    async fn wait_until_idle(&mut self) -> Result<(), display_interface::DisplayError>;
}
#[maybe_async_cfg::maybe(
    sync(keep_self, cfg(not(feature = "async"))),
    async(keep_self, feature = "async"),
)]
impl<SPI, DC, NBUSY, NRESET, DELAY> IEpdInterface
    for EpdInterface<SPI, DC, NBUSY, NRESET, DELAY>
where
    SPI: SpiDevice,
    NBUSY: InputPin,
    NRESET: OutputPin,
    DELAY: DelayNs,
{
    async fn reset(&mut self) -> Result<(), display_interface::DisplayError>
    {
        self.reset().await;
        Ok(())
    }

    async fn wait_until_idle(&mut self) -> Result<(), display_interface::DisplayError> {
        for _ in 0..4 {
            if self.n_busy.is_low().expect("failed to read busy pin") {
                trace!("{TAG} idle asserted");
                return Ok(());
            }
            self.delay.delay_ms(500).await;
        }
        error!("{TAG} idle not asserted");
        Err(display_interface::DisplayError::RSError)
    }
}

/// Provide display_interface::[WriteOnlyDataCommand/AsycnWriteOnlyDataCommand]
#[maybe_async_cfg::maybe(
    sync(keep_self, cfg(not(feature = "async"))),
    async(keep_self, feature = "async"),
    idents(AsyncWriteOnlyDataCommand(async, sync = "WriteOnlyDataCommand"),)
)]
impl<SPI, DC, NBUSY, NRESET, DELAY> display_interface::AsyncWriteOnlyDataCommand
    for EpdInterface<SPI, DC, NBUSY, NRESET, DELAY>
where
    display_interface_spi::SPIInterface<SPI, DC>: display_interface::AsyncWriteOnlyDataCommand,
{
    async fn send_commands(
        &mut self,
        data: display_interface::DataFormat<'_>,
    ) -> Result<(), display_interface::DisplayError> {
        self.spi_interface.send_commands(data).await
    }

    async fn send_data(
        &mut self,
        data: display_interface::DataFormat<'_>,
    ) -> Result<(), display_interface::DisplayError> {
        self.spi_interface.send_data(data).await
    }
}
