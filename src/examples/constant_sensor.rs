use std::time::Duration;

use anyhow::Result;
use esp_idf_hal::gpio::{PinDriver, Pull};
use esp_idf_hal::peripherals::Peripherals;
use sensesp::application::Application;
use sensesp::sensor::{Attachable, ConstantSensor, TimedSensor};

fn main() -> Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    //power pin
    PinDriver::output(peripherals.pins.gpio4)?.set_high()?;

    let mut constant_sensor = ConstantSensor::new(42, Duration::from_secs(2), None);
    let constant_subscriber = constant_sensor.attach();

    let digital_input = PinDriver::input(peripherals.pins.gpio18, Pull::Floating)?;

    //bool sensor maps from Level enums
    let mut digital_sensor = TimedSensor::new(
        move || match digital_input.get_level() {
            esp_idf_hal::gpio::Level::Low => true,
            esp_idf_hal::gpio::Level::High => false,
        },
        Duration::from_millis(500),
        None,
    );

    let digital_subscriber = digital_sensor.attach();
    let mut app = Application::new()
        .register(constant_sensor)
        .register(digital_sensor);

    let _constant_handle = std::thread::spawn(move || {
        esp_idf_hal::task::block_on(async move {
            let mut subscriber = constant_subscriber;
            loop {
                if let Some(v) = subscriber.next().await {
                    log::info!("New constant value found: {}", v);
                }
            }
        });
    });

    let _digital_handle = std::thread::spawn(move || {
        esp_idf_hal::task::block_on(async move {
            let mut subscriber = digital_subscriber;
            loop {
                if let Some(v) = subscriber.next().await {
                    log::info!("New digital value found: {}", v);
                }
            }
        });
    });

    loop {
        app.tick();
        std::thread::sleep(Duration::from_millis(10));
    }
}
