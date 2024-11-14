# SensESP-rs

Sensor library in Rust designed for use with SignalK and the Sailor Hat ESP32.  Built with heavy inspiration from [SensESP](https://github.com/signalk/sensesp).
While this library will strive to eventually reach feature parity with the C++ SensESP implementation, it will not necessarily
follow the same patterns.

## Getting Started
Lots of starter information can be found in [The Rust on ESP Book](https://docs.esp-rs.org/book/).  You will need to install 
a toolchain for the `xtensa-esp32-none-elf` target.  This is a `std` project, not `no-std`, so the full standard library is
at your disposal.

### WSL
To expose the serial port to WSL, `usbipd` must be used to forward the port.  See this [guide](https://developer.espressif.com/blog/espressif-devkits-with-wsl2/)


### Build and Run
Build all targets:
`cargo build`

Run the example on an ESP32: `cargo run --bin example`

Or simply (while only one binary exists): `cargo run`

Eventually, `cargo test` will also do something.  Today is not that day.