#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::info;
use embedded_graphics::mono_font::ascii::FONT_5X8;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::prelude::{Point, Primitive, Size};
use embedded_graphics::primitives::{Circle, PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::text::{Text, TextStyleBuilder};
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
use esp_println as _;

use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::spi::master::Config as SpiConfig;
use esp_hal::spi::master::Spi;
use esp_hal::spi::Mode as SpiMode;
use esp_hal::time::Rate;
use max7219_eg::driver::Max7219;
use max7219_eg::led_matrix::LedMatrix;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // generator version: 0.4.0

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let spi = Spi::new(
        peripherals.SPI2,
        SpiConfig::default()
            .with_frequency(Rate::from_mhz(10))
            .with_mode(SpiMode::_0),
    )
    .unwrap()
    //CLK
    .with_sck(peripherals.GPIO18)
    //DIN
    .with_mosi(peripherals.GPIO23);
    let cs = Output::new(peripherals.GPIO21, Level::High, OutputConfig::default());

    let spi_dev = ExclusiveDevice::new_no_delay(spi, cs).unwrap();

    let mut driver = Max7219::new(spi_dev);
    driver.init().unwrap();
    let mut display: LedMatrix<_> = LedMatrix::from_driver(driver).expect("valid device count");

    let delay = Delay::new();

    // --- Draw Square ---
    let square = PrimitiveStyleBuilder::new()
        .stroke_color(BinaryColor::On) // Only draw the border
        .stroke_width(1) // Border thickness of 1 pixel
        .build();
    let rect = Rectangle::new(Point::new(1, 1), Size::new(6, 6)).into_styled(square);
    rect.draw(&mut display).unwrap();
    display.flush().unwrap();

    delay.delay_millis(1000);

    // Uncomment to Clear the screen and buffer
    // Without this, it will draw the circle inside the previous square
    // display.clear_screen().unwrap();

    // --- Draw Circle ---
    let hollow_circle_style = PrimitiveStyleBuilder::new()
        .stroke_color(BinaryColor::On)
        .stroke_width(1)
        .build();
    let circle = Circle::new(Point::new(2, 2), 4).into_styled(hollow_circle_style);
    circle.draw(&mut display).unwrap();
    display.flush().unwrap();

    delay.delay_millis(1000);

    // Just clear the buffer. it wont send request to the devices until the flush.
    display.clear_buffer();

    //  Write Text (in single device, just a character)
    let text_style = TextStyleBuilder::new()
        .alignment(embedded_graphics::text::Alignment::Center)
        .baseline(embedded_graphics::text::Baseline::Top)
        .build();
    let character_style = MonoTextStyle::new(&FONT_5X8, BinaryColor::On);
    let text = Text::with_text_style("R", Point::new(4, 0), character_style, text_style);
    text.draw(&mut display).unwrap();
    display.flush().unwrap();

    loop {
        info!("Hello world!");
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(500) {}
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-beta.1/examples/src/bin
}
