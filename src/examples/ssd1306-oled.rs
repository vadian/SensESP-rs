use std::time::Duration;

use anyhow::{bail, Result};
use esp_idf_hal::gpio::PinDriver;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::prelude::Peripherals;
use sensesp::wifi::wifi;
use toml_cfg::toml_config;

use esp_idf_svc::hal::i2c::config;
use esp_idf_svc::hal::i2c::I2cDriver;

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

#[derive(Debug)]
#[toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

fn main() -> Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;
    let mut power_pin = PinDriver::output(peripherals.pins.gpio4)?;
    let mut led = PinDriver::output(peripherals.pins.gpio2)?;
    power_pin.set_high()?;
    led.set_high()?;

    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio22;

    log::info!("Preparing to initialize...");
    led.set_low()?;

    let config = config::Config::default()
        //.scl_enable_pullup(false)
        //.sda_enable_pullup(false)
        ;
    log::info!("{:?}", &config);

    // Initialize I2C driver
    let i2c = I2cDriver::new(peripherals.i2c1, sda, scl, &config)?;

    log::info!("Creating Display interface...");

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    log::info!("Creating Barometer...");

    std::thread::sleep(std::time::Duration::from_secs(1));

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline("Hello world!", Point::zero(), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    Text::with_baseline("Hello Rust!", Point::new(0, 16), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    display.flush().unwrap();

    // The constant `CONFIG` is auto-generated by `toml_config`.
    let app_config = CONFIG;

    log::info!("{:#?}", CONFIG);

    // Connect to the Wi-Fi network
    let _wifi = match wifi(
        app_config.wifi_ssid,
        app_config.wifi_psk,
        peripherals.modem,
        sysloop,
    ) {
        Ok(inner) => inner,
        Err(err) => {
            bail!("Could not connect to Wi-Fi network: {:?}", err)
        }
    };

    let mut i = 0;
    std::thread::sleep(Duration::from_secs(5));
    loop {
        match display.clear(BinaryColor::Off){
            Ok(_) => (),
            Err(e) => bail!("Fail clearing display: {:?}", e),
        };

        Text::with_baseline("Hello world!", Point::zero(), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();
        Text::with_baseline(
            &format!("Loop: {}", &i),
            Point::new(0, 16),
            text_style,
            Baseline::Top,
        )
        .draw(&mut display)
        .unwrap();

        display.flush().unwrap();

        i += 1;
        
        // Wait...
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
