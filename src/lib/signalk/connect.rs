use core::str;
use core::time::Duration;

use crate::signalk::auth::get_token;
use anyhow::Result;
use esp_idf_svc::io::EspIOError;
use esp_idf_svc::ws::client::{
    EspWebSocketClient, EspWebSocketClientConfig, WebSocketEvent, WebSocketEventType,
};
use log::{error, info, warn};
use serde_json::json;
use signalk::delta::{V1DeltaFormatBuilder, V1UpdateTypeBuilder};
use signalk::{SignalKStreamMessage, V1DefSource, V1UpdateValue};
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

    //change this to subscribe=all to get flooded with all the server deltas on terminal :D
    let url = format!("ws://{}/signalk/v1/stream?subscribe=all", server_root);
    let _client = EspWebSocketClient::new(url.as_str(), &config, timeout, move |event| {
        handle_signalk_server_event(event)
    })?;

    loop {
        std::thread::sleep(Duration::from_millis(2000));
        match tx.send(SignalKStreamMessage::Delta(
            V1DeltaFormatBuilder::default()
                .add_update(
                    V1UpdateTypeBuilder::default()
                        .source(V1DefSource::builder().label("Basic SensESP-rs Sensor example".to_string()).build())
                        .add_update(V1UpdateValue {
                            path: "navigation.speedOverGround".to_string(),
                            value: json!(7.85),
                        })
                        .build(),
                )
                .build(),
        )) {
            Ok(()) => info!("Successfully sent delta."),
            Err(e) => error!("Error sending delta: {:?}", e),
        }
    }
}

fn handle_signalk_server_event(
    //     _tx: &mpsc::Sender<SignalKStreamMessage>,
    event: &Result<WebSocketEvent, EspIOError>,
) {
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
            error!("Error handling event: {:?}", e);
        }
    }
}
