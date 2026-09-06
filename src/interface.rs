/// provide logging primitives
use defmt_or_log::*;
const TAG: &str = "[EpdInterface]";

/// use standard display errors
use display_interface::DisplayError;
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

#[cfg(not(feature = "async"))]
pub trait WaitUntilIdle {
    /// blocks until idle, or returns error upon timeout
    fn wait_until_idle(&mut self) -> Result<(), display_interface::DisplayError>;
}
#[cfg(feature = "async")]
pub trait AsyncWaitUntilIdle {
    /// yields until idle, or returns error upon timeout
    async fn wait_until_idle(&mut self) -> Result<(), display_interface::DisplayError>;
}

// Provide [WaitUntilIdle/AysncWaitUntilIdle]
//------------------------------------------------------------------------------
#[maybe_async_cfg::maybe(
    sync(keep_self, cfg(not(feature = "async"))),
    async(keep_self, feature = "async"),
    idents(AsyncWaitUntilIdle(async, sync = "WaitUntilIdle"),)
)]
impl<SPI, DC, NBUSY, NRESET, DELAY> AsyncWaitUntilIdle
    for EpdInterface<SPI, DC, NBUSY, NRESET, DELAY>
where
    SPI: SpiDevice,
    NBUSY: InputPin,
    NRESET: OutputPin,
    DELAY: DelayNs,
{
    async fn wait_until_idle(&mut self) -> Result<(), DisplayError> {
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
/// proxy for display_interface_spi
#[cfg(feature = "async")]
use display_interface::AsyncWriteOnlyDataCommand;
#[cfg(not(feature = "async"))]
use display_interface::WriteOnlyDataCommand;

#[maybe_async_cfg::maybe(
    sync(keep_self, cfg(not(feature = "async"))),
    async(keep_self, feature = "async"),
    idents(AsyncWriteOnlyDataCommand(async, sync = "WriteOnlyDataCommand"),)
)]
impl<SPI, DC, NBUSY, NRESET, DELAY> AsyncWriteOnlyDataCommand
    for EpdInterface<SPI, DC, NBUSY, NRESET, DELAY>
where
    display_interface_spi::SPIInterface<SPI, DC>: AsyncWriteOnlyDataCommand,
{
    async fn send_commands(
        &mut self,
        data: display_interface::DataFormat<'_>,
    ) -> Result<(), DisplayError> {
        self.spi_interface.send_commands(data).await
    }

    async fn send_data(
        &mut self,
        data: display_interface::DataFormat<'_>,
    ) -> Result<(), DisplayError> {
        self.spi_interface.send_data(data).await
    }
}
