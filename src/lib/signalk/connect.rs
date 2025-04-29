use core::str;
use core::time::Duration;
use std::marker::PhantomData;

use crate::sensor::SensESPSensor;
use crate::signalk::auth::get_token;
use crate::wifi::wifi;
use anyhow::Result;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::io::EspIOError;
use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use esp_idf_svc::wifi::EspWifi;
use esp_idf_svc::ws::client::{
    EspWebSocketClient, EspWebSocketClientConfig, WebSocketEvent, WebSocketEventType,
};
use log::{error, info, warn};
use serde_json::json;
use signalk::delta::{V1DeltaFormatBuilder, V1UpdateTypeBuilder};
use signalk::{SignalKStreamMessage, V1DefSource, V1UpdateValue};

use super::config;

pub struct SignalKServer<T: ServerState> {
    sensors: Vec<Box<dyn SensESPSensor>>,
    _wifi: Option<Box<EspWifi<'static>>>,
    status: PhantomData<T>,
}

pub trait ServerState {}
pub struct New {}
impl ServerState for New {}
pub struct Initialized {}
impl ServerState for Initialized {}
pub struct Running {}
impl ServerState for Running {}

impl<T: ServerState> SignalKServer<T> {
    pub fn new(
        config: config::SignalKConnection,
        server: &config::SignalKServerDetails,
    ) -> Result<SignalKServer<New>> {
        let wifi = match config {
            config::SignalKConnection::ExistingWifi => None::<Box<EspWifi<'static>>>,
            config::SignalKConnection::WifiPsk {
                ssid,
                password,
                modem,
            } => {
                let sys_loop = EspSystemEventLoop::take()?;

                // Connect to the Wi-Fi network
                let wifi = match wifi(&ssid, &password, modem, sys_loop, None) {
                    Ok(inner) => inner,
                    Err(err) => {
                        error!("Could not connect to Wi-Fi network: {:?}", err);
                        return Err(err);
                    }
                };
                Some(wifi)
            }
            config::SignalKConnection::AccessPointConfigurable => todo!(),
        };
        Ok(SignalKServer::<New> {
            _wifi: wifi,
            sensors: vec![],
            status: PhantomData,
        })
    }
}
impl SignalKServer<New> {
    pub fn attach(&mut self, sensor: Box<dyn SensESPSensor>) -> &mut SignalKServer<New> {
        self.sensors.push(sensor);
        self
    }
    pub fn init(self) -> Result<SignalKServer<Initialized>> {
        todo!();
        Ok(SignalKServer::<Initialized> {
            sensors: self.sensors,
            _wifi: self._wifi,
            status: PhantomData,
        })
    }
}

impl SignalKServer<Initialized> {
    pub fn attach(&mut self, sensor: Box<dyn SensESPSensor>) -> &mut SignalKServer<Initialized> {
        self.sensors.push(sensor);
        self
    }

    pub fn tick(mut self) -> SignalKServer<Running> {
        for sensor in &mut self.sensors {
            sensor.tick();
        }
        SignalKServer::<Running> {
            sensors: self.sensors,
            _wifi: None,
            status: PhantomData,
        }
    }

    pub fn run(self) -> ! {
        let running = SignalKServer::<Running> {
            sensors: self.sensors,
            _wifi: None,
            status: PhantomData,
        };
        running.run();
    }
}

impl SignalKServer<Running> {
    pub fn run(mut self) -> ! {
        loop {
            self.tick();
        }
    }

    pub fn tick(&mut self) -> &mut SignalKServer<Running> {
        for sensor in &mut self.sensors {
            sensor.tick();
        }
        self
    }
}

impl<T: ServerState> SignalKServer<T> {
    pub fn signalk_server(server_root: &str, nvs: Option<EspNvs<NvsDefault>>) -> Result<!> {
        //get info from signalk api
        let token = get_token(server_root, nvs)?;

        let token = format!("Authorization: Bearer {}\r\n", token);

        // Connect websocket
        let config = EspWebSocketClientConfig {
            crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
            headers: Some(token.as_str()),
            ..Default::default()
        };
        let timeout = Duration::from_secs(10);

        //change this to subscribe=all to get flooded with all the server deltas on terminal :D
        let url = format!("ws://{}/signalk/v1/stream?subscribe=all", server_root);
        let mut client = EspWebSocketClient::new(url.as_str(), &config, timeout, move |event| {
            Self::handle_signalk_server_event(event)
        })?;

        loop {
            std::thread::sleep(Duration::from_millis(2000));
            match client.send(
                esp_idf_svc::ws::FrameType::Text(false),
                serde_json::to_string(&SignalKStreamMessage::Delta(
                    V1DeltaFormatBuilder::default()
                        .context("self".to_string())
                        .add_update(
                            V1UpdateTypeBuilder::default()
                                .source(
                                    V1DefSource::builder()
                                        .label("Basic SensESP-rs Sensor example".to_string())
                                        .build(),
                                )
                                .add_update(V1UpdateValue {
                                    path: "navigation.speedOverGround".to_string(),
                                    value: json!(7.85),
                                })
                                .build(),
                        )
                        .build(),
                ))?
                .as_bytes(),
            ) {
                Ok(()) => info!("Successfully sent delta."),
                Err(e) => error!("Error sending delta: {:?}", e),
            }
        }
    }

    fn handle_signalk_server_event(event: &Result<WebSocketEvent, EspIOError>) {
        match event {
            Ok(event) => match event.event_type {
                WebSocketEventType::BeforeConnect => {
                    info!("Websocket before connect");
                }
                WebSocketEventType::Connected => {
                    info!("Websocket connected");
                }
                WebSocketEventType::Disconnected => {
                    info!("Websocket disconnected");
                }
                WebSocketEventType::Close(reason) => {
                    info!("Websocket close, reason: {reason:?}");
                }
                WebSocketEventType::Closed => {
                    info!("Websocket closed");
                    //tx.send(ExampleEvent::Closed).ok();
                }
                WebSocketEventType::Text(text) => {
                    info!("Websocket recv, text: {text}");
                }
                WebSocketEventType::Binary(binary) => {
                    info!("Websocket recv, binary: {binary:?}");
                    let text = match str::from_utf8(binary) {
                        Ok(t) => Some(t),
                        Err(e) => {
                            warn!("Unable to parse binary to ascii: {e:?}");
                            None
                        }
                    };
                    info!("Parsed binary: {text:?}")
                }
                WebSocketEventType::Ping => {
                    info!("Websocket ping");
                }
                WebSocketEventType::Pong => {
                    info!("Websocket pong");
                }
            },
            Err(e) => {
                error!("Error handling websocket event: {:?}", e);
            }
        }
    }
}
