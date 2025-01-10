use esp_idf_hal::modem::Modem;

pub enum SignalKConnection<'a> {
    ExistingWifi,
    WifiPsk {
        ssid: &'a str,
        password: &'a str,
        modem: Modem,
    },
    AccessPointConfigurable, //todo
}

pub struct SignalKServerDetails<'a> {
    hostname: &'a str,
    sensor_name: &'a str,
}
