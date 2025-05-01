#![feature(local_waker)]
#![feature(never_type)]

use anyhow::Result;
use esp_idf_hal::prelude::Peripherals;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs};
use log::{error, info};
use sensesp::{
    sensor::TimedSensor,
    signalk::{
        self,
        config::{SignalKConnection, SignalKServerDetails},
    },
};
use std::time::Duration;
use toml_cfg::toml_config;

#[derive(Debug)]
#[toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
    #[default("")]
    server_root: &'static str,
}

fn main() -> Result<!> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Starting...");

    let app_config = CONFIG;

    // Setup Wifi
    let peripherals = Peripherals::take()?;

    //NVS must be taken before wifi connect
    let nvs = match EspDefaultNvsPartition::take() {
        Ok(n) => match EspNvs::new(n, "default", true) {
            Ok(n) => Some(n),
            Err(e) => {
                error!("Error creating NVS reader: {}", e);
                None
            }
        },
        Err(e) => {
            error!("Error taking NVS partition: {}", e);
            None
        }
    };

    let config = SignalKConnection::WifiPsk {
        ssid: app_config.wifi_ssid.to_string(),
        password: app_config.wifi_psk.to_string(),
        modem: peripherals.modem,
    };

    let details = SignalKServerDetails {
        hostname: app_config.server_root.to_string(),
        sensor_name: "Waterline Height over Deck Sensor".to_string().into(),
    };

    let server = signalk::new(config)?;
    dbg!("Created server.");
    let mut server = server.init(&details, nvs)?;
    dbg!("Initialized server.");

    let digital_sensor = TimedSensor::new(
        move || {
            (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                % 100) as i32 - 50
        },
        Duration::from_millis(500),
        Some("navigation.waterline.aboveDeck".to_string()),
    );
    dbg!("Connected.");
    server.attach(Box::new(digital_sensor))?;
    dbg!("Attached sensor.");

    server.run();
}
