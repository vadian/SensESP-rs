use esp_idf_hal::modem::Modem;

pub enum SignalKConnection {
    ExistingWifi,
    WifiPsk {
        ssid: String,
        password: String,
        modem: Modem,
    },
    AccessPointConfigurable, //todo
}

pub struct SignalKServerDetails {
    pub hostname: String,
    pub sensor_name: Option<String>,
}
