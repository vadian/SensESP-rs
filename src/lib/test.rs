#![feature(local_waker)]
#![feature(noop_waker)]
use std::task::Waker;
use std::task::{ContextBuilder, LocalWaker};
use std::time::Duration;

use anyhow::Result;
use sensesp::application::Application;
use sensesp::sensor::{ConstantSensor, Sensor};
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

    let mut sensor = ConstantSensor::new(42, Duration::from_secs(2));
    let mut subscriber = sensor.attach();
    let mut app = Application::new().register(sensor);

    let _handle = std::thread::spawn(move || {
        let local_waker = LocalWaker::noop();
        let waker = Waker::noop();

        let mut cx = ContextBuilder::from_waker(&waker)
            .local_waker(&local_waker)
            .build();
        loop {
            match subscriber.poll_next(&mut cx) {
                std::task::Poll::Ready(val) => match val {
                    Some(i) => log::info!("New value found: {}", i),
                    None => log::warn!("No new value found."),
                },
                std::task::Poll::Pending => std::thread::sleep(Duration::from_millis(200)),
            }
        }
    });

    loop {
        app.tick();
        std::thread::sleep(Duration::from_millis(10));
    }
}
