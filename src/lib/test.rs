#![feature(local_waker)]
#![feature(noop_waker)]
use std::task::Poll::{Pending, Ready};
use std::task::Waker;
use std::task::{ContextBuilder, LocalWaker};
use std::time::Duration;

use anyhow::Result;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::prelude::Peripherals;
use sensesp::application::Application;
use sensesp::sensor::{Attachable, ConstantSensor, TimedSensor};
use smol::stream::StreamExt;
use toml_cfg::toml_config;

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

    //power pin
    PinDriver::output(peripherals.pins.gpio4)?.set_high()?;

    let mut constant_sensor = ConstantSensor::new(42, Duration::from_secs(2));
    let mut constant_subscriber = constant_sensor.attach();

    let digital_input = PinDriver::input(peripherals.pins.gpio18)?;

    //bool sensor maps from Level enums
    let mut digital_sensor = TimedSensor::new(
        move || match digital_input.get_level() {
            esp_idf_hal::gpio::Level::Low => true,
            esp_idf_hal::gpio::Level::High => false,
        },
        Duration::from_millis(500),
    );

    let mut digital_subscriber = digital_sensor.attach();
    let mut app = Application::new()
        .register(constant_sensor)
        .register(digital_sensor);

    let _handle = std::thread::spawn(move || {
        let local_waker = LocalWaker::noop();
        let waker = Waker::noop();

        let mut cx = ContextBuilder::from_waker(&waker)
            .local_waker(&local_waker)
            .build();
        loop {
            match constant_subscriber.poll_next(&mut cx) {
                Ready(val) => match val {
                    Some(i) => log::info!("New constant value found: {}", i),
                    None => log::warn!("No new constant value found."),
                },
                Pending => (),
            }

            match digital_subscriber.poll_next(&mut cx) {
                Ready(val) => match val {
                    Some(i) => log::info!("New digital value found: {}", i),

                    None => log::warn!("No new digital value found."),
                },
                Pending => (),
            }

            std::thread::sleep(Duration::from_millis(100));
        }
    });

    loop {
        app.tick();
        std::thread::sleep(Duration::from_millis(10));
    }
}
