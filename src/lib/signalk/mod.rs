pub mod connect {
    use core::time::Duration;

    use embedded_svc::{
        http::client::Client as HttpClient,
        io::Write,
        utils::io,
    };
    use esp_idf_svc::eventloop::EspSystemEventLoop;
    use esp_idf_svc::hal::peripherals::Peripherals;
    use esp_idf_svc::http::client::EspHttpConnection;
    use esp_idf_svc::io::EspIOError;
    use esp_idf_svc::nvs::EspDefaultNvsPartition;
    use esp_idf_svc::ws::client::{
        EspWebSocketClient, EspWebSocketClientConfig, WebSocketEvent, WebSocketEventType,
    };
    use anyhow::{anyhow, Result};
    use log::{error, info, warn};
    use serde::{Deserialize, Serialize};
    use signalk::{definitions::V1DateTime, SignalKStreamMessage};
    use std::sync::mpsc;
    use crate::wifi::wifi;


    const SIGNALK_SERVER_URI: &str = "signalk-demo.seapupper.me";
    #[allow(unused)]
    const SIGNALK_DEMO_SERVER_URI: &str = "wss://demo.signalk.org/signalk/v1/api/";

    #[allow(non_snake_case)]
    #[derive(Debug, Serialize)]
    struct DeviceAccessRequest {
        clientId: String,
        description: String,
    }

    #[allow(non_snake_case, unused)]
    #[derive(Debug, Deserialize)]
    struct DeviceAccessResponse {
        state: DeviceAccessState,
        requestId: String,
        statusCode: usize,
        message: Option<String>,
        accessRequest: Option<AccessRequest>,
        href: String,
        ip: String,
    }

    #[allow(non_snake_case, unused)]
    #[derive(Debug, Deserialize)]
    struct AccessRequest {
        permission: Permission,
        token: Option<String>,
        expirationTime: Option<V1DateTime>,
    }

    #[allow(clippy::upper_case_acronyms)]
    #[derive(Debug, Deserialize)]
    enum Permission {
        APPROVED,
        DENIED,
    }

    #[allow(clippy::upper_case_acronyms)]
    #[derive(Debug, Deserialize)]
    enum DeviceAccessState {
        COMPLETED,
        PENDING,
    }

    pub fn signalk_server(wifi_ssid: &str, wifi_psk: &str) -> Result<()> {
        // Setup Wifi
        let peripherals = Peripherals::take()?;
        let sys_loop = EspSystemEventLoop::take()?;
        let _nvs = EspDefaultNvsPartition::take()?;
        
        // Connect to the Wi-Fi network
        let _wifi= match wifi(
            wifi_ssid,
            wifi_psk,
            peripherals.modem,
            sys_loop,
        ) {
            Ok(inner) => inner,
            Err(err) => {
                error!("Could not connect to Wi-Fi network: {:?}", err);
                return Err(err);
            }
        };

        //get info from signalk api
        let token = get_token()?;

        let token = format!("Authorization: Bearer {}\r\n", token);

        // Connect websocket
        let config = EspWebSocketClientConfig {
            crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
            headers: Some(token.as_str()),
            ..Default::default()
        };
        let timeout = Duration::from_secs(10);
        let (tx, _rx) = mpsc::channel::<SignalKStreamMessage>();

        let url = format!(
            "ws://{}/signalk/v1/stream?subscribe=all",
            SIGNALK_SERVER_URI
        );
        let _client = EspWebSocketClient::new(url.as_str(), &config, timeout, move |event| {
            handle_signalk_server_event(&tx, event)
        })?;

        loop {
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    fn post_access_request(
        client: &mut HttpClient<EspHttpConnection>,
    ) -> Result<DeviceAccessResponse> {
        let payload: DeviceAccessRequest = DeviceAccessRequest {
            clientId: "31337-400432-6317832".to_string(),
            description: "Basic SensESP-rs Sensor example".to_string(),
        };

        let json = match serde_json::to_string(&payload) {
            Ok(s) => s,
            Err(e) => return Err(anyhow!(e)),
        };

        let url = format!("http://{}/signalk/v1/access/requests", SIGNALK_SERVER_URI);

        let content_length_header: String = format!("{}", json.len());

        let headers = [
            ("content-type", "application/json"),
            ("content-length", content_length_header.as_str()),
        ];

        // Send request
        let mut request = client.post(&url, &headers)?;
        request.write_all(json.as_bytes())?;
        request.flush()?;
        info!("-> POST {}", url);
        let mut response = request.submit()?;

        // Process response
        let status = response.status();
        info!("<- {}", status);

        match status {
            202 | 400 => {
                info!("Successfully submitted request.");
                status
            }
            404 => {
                info!("404? I think that's a completed but pending connection");
                404
            }
            501 => return Err(anyhow!("Oh no! The server does not support device auth.")),
            _ => return Err(anyhow!("Oh no! The server returned an unknown status.")),
        };

        let mut buf = [0u8; 1024];
        let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0)?;
        info!("Read {} bytes", bytes_read);
        let body_string = match std::str::from_utf8(&buf[0..bytes_read]) {
            Ok(body_string) => {
                info!(
                    "Response body (truncated to {} bytes): {:?}",
                    buf.len(),
                    &body_string
                );
                body_string
            }
            Err(e) => {
                error!("Error decoding response body: {}", e);
                return Err(anyhow!(e));
            }
        };

        let response: DeviceAccessResponse = serde_json::from_str(body_string)?;

        Ok(response)
    }

    fn get_access_request_status(
        client: &mut HttpClient<EspHttpConnection>,
        href: &str,
    ) -> Result<DeviceAccessResponse> {
        let url = "http://signalk-demo.seapupper.me";
        let url = format!("{}{}", url, href);

        // Send request
        let request = client.get(&url)?;
        info!("-> GET {}", url);
        let mut response = request.submit()?;

        // Process response
        let status = response.status();
        info!("<- {}", status);

        match status {
            200 => {
                info!("Connection pending, status request received");
                status
            }
            404 => {
                info!("404? I think that's a completed but pending connection");
                404
            }
            501 => return Err(anyhow!("Oh no! The server does not support device auth.")),
            _ => return Err(anyhow!("Oh no! The server returned an unknown status.")),
        };

        let mut buf = [0u8; 1024];
        let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0)?;
        info!("Read {} bytes", bytes_read);
        let body_string = match std::str::from_utf8(&buf[0..bytes_read]) {
            Ok(body_string) => {
                info!(
                    "Response body (truncated to {} bytes): {:?}",
                    buf.len(),
                    &body_string
                );
                body_string
            }
            Err(e) => {
                error!("Error decoding response body: {}", e);
                return Err(anyhow!(e));
            }
        };

        let response: DeviceAccessResponse = serde_json::from_str(body_string)?;

        Ok(response)
    }

    fn validate_token(
        client: &mut HttpClient<EspHttpConnection>,
        token: &str,
    ) -> Result<DeviceAccessResponse> {
        let url = format!("http://{}/signalk/v1/auth/validate", SIGNALK_SERVER_URI);

        let bearer = format!("Bearer {}", token);

        let content = "";
        let content_length_header: String = format!("{}", content.len());

        let headers = [
            ("content-type", "application/json"),
            ("content-length", content_length_header.as_str()),
            ("authorization", bearer.as_str()),
        ];

        // Send request
        let mut request = client
            .post(&url, &headers)
            .map_err(|e| anyhow!("Error creating HTTP Request: {:?}", e))?;
        request.write_all(content.as_bytes())?;
        request.flush()?;
        info!("-> POST {}", url);

        let mut response = request
            .submit()
            .map_err(|e| anyhow!("Error submitting HTTP Request: {:?}", e))?;

        // Process response
        let status = response.status();
        info!("<- {}", status);

        match status {
            200 => {
                info!("Connection pending, status request received");
                status
            }
            404 => {
                info!("404? I think that's a completed but pending connection");
                404
            }
            501 => return Err(anyhow!("Oh no! The server does not support device auth.")),
            _ => return Err(anyhow!("Oh no! The server returned an unknown status.")),
        };

        let mut buf = [0u8; 1024];
        let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0)?;
        info!("Read {} bytes", bytes_read);
        let body_string = match std::str::from_utf8(&buf[0..bytes_read]) {
            Ok(body_string) => {
                info!(
                    "Response body (truncated to {} bytes): {:?}",
                    buf.len(),
                    &body_string
                );
                body_string
            }
            Err(e) => {
                error!("Error decoding response body: {}", e);
                return Err(anyhow!(e));
            }
        };

        let response: DeviceAccessResponse = serde_json::from_str(body_string)?;

        Ok(response)
    }

    fn get_token() -> Result<String> {
        let token: Option<String> = Some("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJkZXZpY2UiOiIzMTMzNy00MDA0MzItNjMxNzgzMiIsImlhdCI6MTczNDIwMDUwN30.iYbrKnuw08XUIzMJnOTBxtXsc_S-Gl7cNLm_ExnfQdY".to_string());
        let do_validation = false;

        let mut client = HttpClient::wrap(EspHttpConnection::new(&Default::default())?);

        let token = match token {
            Some(t) => match do_validation {
                false => Some(t),
                true => match validate_token(&mut client, &t.clone()) {
                    Ok(r) => match r.accessRequest {
                        Some(a) => match a.permission {
                            Permission::APPROVED => {
                                info!("Permission approved - token validated");
                                Some(t)
                            }
                            Permission::DENIED => {
                                warn!("Permission denied - token NOT valid");
                                None
                            }
                        },
                        None => None,
                    },
                    Err(e) => {
                        warn!("Error validating token: {}", e);
                        None
                    }
                },
            },
            None => {
                info!("No token available.  Requesting new token.");
                None
            }
        };

        match token {
            Some(t) => Ok(t),
            None => match fetch_token(&mut client) {
                Ok(t) => {
                    info!("Success: {}", t);
                    Ok(t)
                }
                Err(e) => {
                    error!("Error! {}", e);
                    Err(e)
                }
            },
        }
    }

    fn fetch_token(client: &mut HttpClient<EspHttpConnection>) -> Result<String> {
        let response = post_access_request(client)?;

        //loop until we complete
        loop {
            let response = get_access_request_status(client, response.href.as_str())?;

            match response.state {
                DeviceAccessState::PENDING => {
                    info!("{:?}", response);
                    std::thread::sleep(Duration::from_secs(5));
                    continue;
                },
                DeviceAccessState::COMPLETED => {
                    return match response.accessRequest {
                        Some(ar) => match ar.permission {
                                Permission::APPROVED => match ar.token {
                                    Some(t) => Ok(t),
                                    None => Err(anyhow!("Access request approved but no token. This is out of spec.")),
                                },
                                Permission::DENIED => Err(anyhow!("Access request DENIED by the server. This is an explicit action, does a fishing buddy hate you?")),
                            },
                        None => {
                            Err(anyhow!("Access request state is COMPLETED without accessRequest section.
                                This happens when a request is pending and the same ClientId resubmits a request,
                                rather than referencing by href. {:?}", &response))
                        }
                    }
                }
            }
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
}
