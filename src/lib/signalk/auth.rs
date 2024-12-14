use core::time::Duration;

use anyhow::{anyhow, Result};
use embedded_svc::{http::client::Client as HttpClient, io::Write, utils::io};
use esp_idf_svc::http::client::EspHttpConnection;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use signalk::definitions::V1DateTime;

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
pub struct AccessRequest {
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

fn post_access_request(
    client: &mut HttpClient<EspHttpConnection>,
    server_root: &str,
) -> Result<DeviceAccessResponse> {
    let payload: DeviceAccessRequest = DeviceAccessRequest {
        clientId: "31337-400432-6317832".to_string(),
        description: "Basic SensESP-rs Sensor example".to_string(),
    };

    let json = match serde_json::to_string(&payload) {
        Ok(s) => s,
        Err(e) => return Err(anyhow!(e)),
    };

    let url = format!("http://{}/signalk/v1/access/requests", server_root);

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
    server_root: &str,
    href: &str,
) -> Result<DeviceAccessResponse> {
    let url = format!("http://{}{}", server_root, href);

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
    server_root: &str,
    token: &str,
) -> Result<DeviceAccessResponse> {
    let url = format!("http://{}/signalk/v1/auth/validate", server_root);

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

pub(crate) fn get_token(server_root: &str) -> Result<String> {
    let nvs = match EspDefaultNvsPartition::take() {
        Ok(n) => match EspNvs::new(n, "token", true) {
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

    let mut buf = [0u8; 1024];
    let token = match nvs {
        Some(ref n) => match n.get_blob("token", &mut buf) {
            Ok(b) => match b {
                Some(u) => match String::from_utf8(u.to_vec()) {
                    Ok(s) => Some(s),
                    Err(e) => {
                        error!("Error parsing token: {}", e);
                        None
                    }
                },
                None => None,
            },
            Err(e) => {
                error!("Error pulling token from NVS: {}", e);
                None
            }
        },
        None => None,
    };
    info!("Token: {:?}", token);

    let do_validation = false;

    let mut client = HttpClient::wrap(EspHttpConnection::new(&Default::default())?);

    let token = match token {
        Some(t) => match do_validation {
            false => Some(t),
            true => match validate_token(&mut client, server_root, &t.clone()) {
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
        None => match fetch_token(&mut client, server_root) {
            Ok(t) => {
                info!("Success: {}", t);
                match nvs {
                    Some(mut n) => match n.set_blob("token", t.as_bytes()) {
                        Ok(()) => {
                            info!("Successfully stored token");
                        }
                        Err(e) => {
                            error!("Error storing token to NVS: {}", e);
                        }
                    },
                    None => {
                        warn!("No NVS - unable to store token");
                    }
                };
                Ok(t)
            }
            Err(e) => {
                error!("Error! {}", e);
                Err(e)
            }
        },
    }
}

fn fetch_token(client: &mut HttpClient<EspHttpConnection>, server_root: &str) -> Result<String> {
    let response = post_access_request(client, server_root)?;

    //loop until we complete
    loop {
        let response = get_access_request_status(client, server_root, response.href.as_str())?;

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
