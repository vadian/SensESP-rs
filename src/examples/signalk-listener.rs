#![feature(local_waker)]
#![feature(never_type)]

use anyhow::Result;
use esp_idf_hal::prelude::Peripherals;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::io::EspIOError,
    nvs::{EspDefaultNvsPartition, EspNvs},
    ws::client::{
        EspWebSocketClient, EspWebSocketClientConfig, WebSocketEvent, WebSocketEventType,
    },
};
use log::{error, info, warn};
use sensesp::{signalk::config::SignalKServerDetails, wifi::wifi};
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

    info!("Got peris...");
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
    info!("got nvs");

    let sys_loop = EspSystemEventLoop::take()?;

    info!("got sys loop");
    // Connect to the Wi-Fi network
    // Setup Wifi
    let modem = peripherals.modem;

    info!("got modem");
    let wifi = match wifi(
        app_config.wifi_ssid,
        app_config.wifi_psk,
        modem,
        sys_loop,
        None,
    ) {
        Ok(inner) => inner,
        Err(err) => {
            error!("Could not connect to Wi-Fi network: {:?}", err);
            return Err(err);
        }
    };

    info!("Connected to Wi-Fi network");

    // Connect websocket
    let config = EspWebSocketClientConfig {
        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
        headers: None,
        task_prio: 1,
        task_stack: 8192,
        ..Default::default()
    };

    let details = SignalKServerDetails {
        hostname: app_config.server_root.to_string(),
        sensor_name: "Waterline Height over Deck Sensor".to_string().into(),
    };
    let timeout = Duration::from_secs(2);
    let url = format!("ws://{}/signalk/v1/stream?subscribe=none", details.hostname);
    let listener = EspWebSocketClient::new(&url, &config, timeout, move |event| {
        handle_signalk_server_event("RECVR", event)
    })?;
    loop {
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn handle_signalk_server_event(name: &str, event: &Result<WebSocketEvent, EspIOError>) {
    match event {
        Ok(event) => match event.event_type {
            WebSocketEventType::BeforeConnect => {
                info!("{name}: Websocket before connect");
            }
            WebSocketEventType::Connected => {
                info!("{name}: Websocket connected");
            }
            WebSocketEventType::Disconnected => {
                info!("{name}: Websocket disconnected");
            }
            WebSocketEventType::Close(reason) => {
                info!("{name}: Websocket close, reason: {reason:?}");
            }
            WebSocketEventType::Closed => {
                info!("{name}: Websocket closed");
                //tx.send(ExampleEvent::Closed).ok();
            }
            WebSocketEventType::Text(text) => {
                info!("{name}: Websocket recv, text: {text}");
            }
            WebSocketEventType::Binary(binary) => {
                info!("{name}: Websocket recv, binary: {binary:?}");
                let text = match std::str::from_utf8(binary) {
                    Ok(t) => Some(t),
                    Err(e) => {
                        warn!("{name}: Unable to parse binary to ascii: {e:?}");
                        None
                    }
                };
                info!("{name}: Parsed binary: {text:?}")
            }
            WebSocketEventType::Ping => {
                info!("{name}: Websocket ping");
            }
            WebSocketEventType::Pong => {
                info!("{name}: Websocket pong");
            }
            WebSocketEventType::Error(_) => todo!(),
        },
        Err(e) => {
            error!("{name}: Error handling websocket event: {:?}", e);
        }
    }
}
