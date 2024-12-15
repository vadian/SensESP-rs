#![feature(local_waker)]
#![feature(noop_waker)]

use anyhow::Result;
use esp_idf_hal::prelude::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use log::{error, info};
use sensesp::{signalk::connect::signalk_server, wifi::wifi};
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

fn main() -> Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Starting...");

    let app_config = CONFIG;

    // Setup Wifi
    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;

    // Connect to the Wi-Fi network
    let _wifi = match wifi(
        app_config.wifi_ssid,
        app_config.wifi_psk,
        peripherals.modem,
        sys_loop,
    ) {
        Ok(inner) => inner,
        Err(err) => {
            error!("Could not connect to Wi-Fi network: {:?}", err);
            return Err(err);
        }
    };

    signalk_server(app_config.server_root)
}
