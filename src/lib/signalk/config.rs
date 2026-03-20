pub enum SignalKConnection {
    ExistingWifi,
    WifiPsk { ssid: String, password: String },
    AccessPointConfigurable, //todo
}

pub struct SignalKServerDetails {
    pub hostname: String,
    pub sensor_name: Option<String>,
}
