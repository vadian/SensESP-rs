use core::str;
use core::time::Duration;
use std::fmt::Display;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crate::sensor::{Attachable, NamedSensor, SensESPSensor};
use crate::signalk::auth::get_token;
use crate::wifi::wifi;
use anyhow::Result;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::io::EspIOError;
use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use esp_idf_svc::wifi::EspWifi;
use esp_idf_svc::ws::FrameType;
use esp_idf_svc::ws::client::{
    EspWebSocketClient, EspWebSocketClientConfig, WebSocketEvent, WebSocketEventType,
};
use log::{error, info, warn};
use serde::Serialize;
use serde_json::json;
use signalk::delta::{V1DeltaFormatBuilder, V1UpdateTypeBuilder};
use signalk::{SignalKStreamMessage, V1DefSource, V1UpdateValue};

pub mod auth;
pub mod config;

pub struct SignalKServer<T: ServerState> {
    sensors: Vec<Box<dyn SensESPSensor>>,
    subscribers: Vec<JoinHandle<()>>,
    device_name: Option<String>,
    _wifi: Option<Box<EspWifi<'static>>>,
    ws: Option<Arc<Mutex<EspWebSocketClient<'static>>>>,
    status: PhantomData<T>,
}

pub trait ServerState {}
pub struct New {}
impl ServerState for New {}
pub struct Initialized {}
impl ServerState for Initialized {}
pub struct Running {}
impl ServerState for Running {}

pub fn new(config: config::SignalKConnection) -> Result<SignalKServer<New>> {
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
        sensors: vec![],
        subscribers: vec![],
        device_name: None,
        ws: None,
        _wifi: wifi,
        status: PhantomData,
    })
}

impl SignalKServer<New> {
    pub fn init(
        self,
        server: &config::SignalKServerDetails,
        nvs: Option<EspNvs<NvsDefault>>,
    ) -> Result<SignalKServer<Initialized>> {
        //get info from signalk api
        let token = get_token(&server.hostname, nvs)?;

        let token = format!("Authorization: Bearer {}\r\n", token);

        // Connect websocket
        let config = EspWebSocketClientConfig {
            crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
            headers: Some(token.as_str()),
            ..Default::default()
        };
        let timeout = Duration::from_secs(10);

        //change this to subscribe=all to get flooded with all the server deltas on terminal :D
        let url = format!("ws://{}/signalk/v1/stream?subscribe=all", server.hostname);
        let client = EspWebSocketClient::new(url.as_str(), &config, timeout, move |event| {
            Self::handle_signalk_server_event(event)
        })?;
        while !client.is_connected() {
            info!("Waiting for websocket connection...");
            std::thread::sleep(Duration::from_millis(100));
        }
        let client = Arc::new(Mutex::new(client));

        Ok(SignalKServer::<Initialized> {
            sensors: self.sensors,
            subscribers: self.subscribers,
            device_name: server.sensor_name.clone(),
            ws: Some(client),
            _wifi: self._wifi,
            status: PhantomData,
        })
    }
}

impl SignalKServer<Initialized> {
    pub fn attach<T: std::clone::Clone + Display + Serialize + Send + Sync + 'static>(
        &mut self,
        mut sensor: Box<impl SensESPSensor + NamedSensor + Attachable<T> + 'static>,
    ) -> Result<()> {
        let mut attachable = sensor.attach();
        let name = sensor.name();
        let device_name = self
            .device_name
            .clone()
            .unwrap_or("Basic SensESP-rs Sensor example".to_string());
        let ws = if let Some(ws) = &self.ws {
            Some(ws.clone())
        } else {
            warn!("No websocket client available when attaching.");
            None
        };

        let thread = std::thread::Builder::new()
            .stack_size(8192)
            .spawn(move || {
                // Use a runtime to execute the async block
                esp_idf_hal::task::block_on(async move {
                    loop {
                        match attachable.next().await {
                            Some(v) => {
                                info!("New value found: {}", v);
                                if let Some(ws) = &ws {
                                    let msg = create_message(device_name.clone(), name.clone(), v);
                                    let _ = send_message(ws.clone(), msg).await;
                                } else {
                                    warn!("No websocket client available.");
                                }
                            }
                            None => warn!("No new value found."),
                        };
                        std::thread::sleep(Duration::from_millis(25));
                    }
                });
            })?;

        self.subscribers.push(thread);

        self.sensors.push(sensor);

        Ok(())
    }

    pub fn tick(self) -> SignalKServer<Running> {
        let mut running = SignalKServer::<Running> {
            sensors: self.sensors,
            subscribers: self.subscribers,
            ws: self.ws,
            device_name: self.device_name,
            _wifi: self._wifi,
            status: PhantomData,
        };
        running.tick();
        running
    }

    pub fn run(self) -> ! {
        let running = SignalKServer::<Running> {
            sensors: self.sensors,
            subscribers: self.subscribers,
            device_name: self.device_name,
            ws: self.ws,
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
            std::thread::sleep(Duration::from_millis(50));
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

fn create_message<T: std::clone::Clone + Display + Serialize>(
    device_name: String,
    name: String,
    value: T,
) -> SignalKStreamMessage {
    let update = V1UpdateTypeBuilder::default()
        .source(V1DefSource::builder().label(device_name.clone()).build())
        .add_update(V1UpdateValue {
            path: name.clone(),
            value: json!(value),
        })
        .build();
    let msg = SignalKStreamMessage::Delta(
        V1DeltaFormatBuilder::default()
            .context("self".to_string())
            .add_update(update)
            .build(),
    );
    msg
}

async fn send_message(
    ws: Arc<Mutex<EspWebSocketClient<'static>>>,
    msg: SignalKStreamMessage,
) -> Result<()> {
    let data = serde_json::to_string(&msg).unwrap_or("SerializationFail".to_string());
    info!("Sending data: {:?}", data);
    {
        let mut client = ws.lock().unwrap();

        let res = client.send(FrameType::Text(false), data.as_bytes());
        match res {
            Ok(()) => info!("Successfully sent delta."),
            Err(e) => error!("Error sending delta: {:?}", e),
        }
        Ok(res?)
    }
}
