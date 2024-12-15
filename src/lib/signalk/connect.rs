use core::time::Duration;

use crate::signalk::auth::get_token;
use anyhow::Result;
use esp_idf_svc::io::EspIOError;
use esp_idf_svc::ws::client::{
    EspWebSocketClient, EspWebSocketClientConfig, WebSocketEvent, WebSocketEventType,
};
use log::{error, info};
use signalk::SignalKStreamMessage;
use std::sync::mpsc;

pub fn signalk_server(server_root: &str) -> Result<()> {
    //get info from signalk api
    let token = get_token(server_root)?;

    let token = format!("Authorization: Bearer {}\r\n", token);

    // Connect websocket
    let config = EspWebSocketClientConfig {
        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
        headers: Some(token.as_str()),
        ..Default::default()
    };
    let timeout = Duration::from_secs(10);
    let (tx, _rx) = mpsc::channel::<SignalKStreamMessage>();

    let url = format!("ws://{}/signalk/v1/stream?subscribe=all", server_root);
    let _client = EspWebSocketClient::new(url.as_str(), &config, timeout, move |event| {
        handle_signalk_server_event(&tx, event)
    })?;

    loop {
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn handle_signalk_server_event(
    _tx: &mpsc::Sender<SignalKStreamMessage>,
    event: &Result<WebSocketEvent, EspIOError>,
) {
    match event {
        Ok(event) => match event.event_type {
            WebSocketEventType::BeforeConnect => {
                info!("Websocket before connect");
                //tx.send(SignalKStreamMessage::Hello(V1HelloBuilder::default().build()))
                //.ok();
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
                if text == "Hello, World!" {
                    //tx.send(ExampleEvent::MessageReceived).ok();
                }
            }
            WebSocketEventType::Binary(binary) => {
                info!("Websocket recv, binary: {binary:?}");
            }
            WebSocketEventType::Ping => {
                info!("Websocket ping");
            }
            WebSocketEventType::Pong => {
                info!("Websocket pong");
            }
        },
        Err(e) => {
            error!("{}", e);
        }
    }
}
