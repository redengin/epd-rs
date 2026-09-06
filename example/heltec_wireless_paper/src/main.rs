#![no_std]
#![no_main]

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

/// provide logging primitives
use log::*;

// support esp32
// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();
use esp_hal::clock::CpuClock;
use esp_hal::main;
use esp_hal::time::Instant;
extern crate alloc;

#[main]
fn main() -> ! {
    // generator version: 1.3.0
    // generator parameters: --chip esp32s3 -o unstable-hal -o alloc

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::default());
    let peripherals = esp_hal::init(config);

    // enable loggging
    esp_println::logger::init_logger_from_env();

    // create the heap space (reclaiming bootloader RAM)
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 73744);

    // CONFIGURATION - using HELTEC WIRELESS PAPER board
    // ------------------------------------------------------------------
    let vext_control = peripherals.GPIO45; // power screen [high - disable]/[low - enable]
    let sck_pin = peripherals.GPIO3;
    let mosi_pin = peripherals.GPIO2;
    let cs_pin = peripherals.GPIO4;
    let dc_pin = peripherals.GPIO5;
    let reset_pin = peripherals.GPIO6;
    let busy_pin = peripherals.GPIO7;
    const WIDTH: u32 = 122;
    const HEIGHT: u32 = 250;
    // ------------------------------------------------------------------

    // power on the screen
    esp_hal::gpio::Output::new(
        vext_control,
        esp_hal::gpio::Level::Low,
        esp_hal::gpio::OutputConfig::default(),
    );
    // give power time to synchronize
    esp_hal::delay::Delay::new().delay_millis(50);

    // create the SOC SPI bus
    let spi = esp_hal::spi::master::Spi::new(
        peripherals.SPI3,
        esp_hal::spi::master::Config::default()
            .with_frequency(esp_hal::time::Rate::from_mhz(20)),
    )
    .unwrap()
    .with_sck(sck_pin)
    .with_mosi(mosi_pin);

    // create SPI interface for display
    let spi_interface = display_interface_spi::SPIInterface::new(
        embedded_hal_bus::spi::ExclusiveDevice::new_no_delay(
            spi,
            esp_hal::gpio::Output::new(
                cs_pin,
                esp_hal::gpio::Level::Low,
                esp_hal::gpio::OutputConfig::default(),
            ),
        )
        .unwrap(),
        esp_hal::gpio::Output::new(
            dc_pin,
            esp_hal::gpio::Level::Low,
            esp_hal::gpio::OutputConfig::default(),
        ),
    );

    // create the driver object
    let n_busy = esp_hal::gpio::Input::new(busy_pin, esp_hal::gpio::InputConfig::default());
    let n_reset = esp_hal::gpio::Output::new(
        reset_pin,
        esp_hal::gpio::Level::Low,
        esp_hal::gpio::OutputConfig::default(),
    );
    let display_interface = epd_rs::EpdInterface::new(
        spi_interface,
        n_busy,
        n_reset,
        esp_hal::delay::Delay::new(),
    );

    // Choose driver
    let driver = epd_rs::drivers::E0213A367::new(
        display_interface,
        embedded_graphics::geometry::Size::new(WIDTH, HEIGHT),
    ).unwrap();

    // create the display
    let mut display = epd_rs::EpdDrawTarget::new(driver, epd_rs::DisplayRotation::Rotate0);

    // let example_screen = ExampleScreen::new();

    let mut frame_rate: f32 = 0.0;
    loop {
        // update the screen
        // let _ = example_screen
        //     .update(&mut display, frame_rate)
        //     .map_err(|e| error!("{:?}", e));

        // monitor the frame rate
        trace!("starting refresh...");
        let frame_start = Instant::now();

        let _ = display.refresh().map_err(|e| error!("{:?}", e));

        trace!("refresh completed");

        let frame_period = frame_start.elapsed();
        frame_rate = 1000.0 / (frame_period.as_millis() as f32);
        info!(
            "frame_period: {} ms   FPS: {frame_rate:.0} Hz",
            frame_period.as_millis()
        );
    }
}

use embedded_graphics::prelude::*;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::pixelcolor::BinaryColor;
use display_interface::DisplayError;
struct ExampleScreen {
    text: embedded_graphics::mono_font::MonoTextStyle<'static, BinaryColor>,
}
impl ExampleScreen {
    pub fn new() -> Self {
        Self {
            text: MonoTextStyle::new(&FONT_10X20, BinaryColor::On),
        }
    }

    // pub fn update<DI>(
    pub fn update(
        &self,
        display: &mut impl DrawTarget<Color = BinaryColor>,
        frame_rate: f32,
    ) -> Result<(), DisplayError> {
        // clear the display
        let _ = display.clear(embedded_graphics::pixelcolor::BinaryColor::Off);

        // update the display
        info!("udpdating display...");
        // let _ = Text::new("Hello World!", Point::zero(), self.text).draw(display);

        // let mut fps_text_string = String::new();
        // let _ = write!(&mut fps_text_string, "FPS: {frame_rate:.0}");
        // let _ = Text::with_alignment(
        //     &fps_text_string,
        //     display.bounding_box().center(),
        //     self.text,
        //     embedded_graphics::text::Alignment::Center,
        // )
        // .draw(display);

        info!("udpdated display");
        Ok(())
    }
}