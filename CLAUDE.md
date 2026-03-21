# SensESP-rs

Rust implementation of SensESP for ESP32 microcontrollers using the esp-idf-hal ecosystem.

## Build

```bash
cargo build
```

Target chip and features are configured in `.cargo/config.toml` and `rust-toolchain.toml`.

## Library Modules

The shared library is located in `src/lib/` and provides:

### sensor (`src/lib/sensor.rs`)

Core sensor abstractions using the observer pattern via the `eyeball` crate.

- **`SensESPSensor`** - Trait for sensors with a `tick()` method called in the main loop
- **`NamedSensor`** - Trait for sensors with a SignalK path name
- **`Attachable<T>`** - Trait providing `attach()` to get a reactive `Subscriber<T>`
- **`ConstantSensor<T>`** - Emits a constant value at fixed intervals
- **`TimedSensor<T, F>`** - Calls a closure at fixed intervals and emits the result

Subscribers use `eyeball::Subscriber` which supports async `.next().await` for reactive notifications.

### application (`src/lib/application.rs`)

Simple sensor registry for standalone use without SignalK.

- **`Application`** - Registers sensors and calls `tick()` on each in a loop

### signalk (`src/lib/signalk/`)

SignalK server integration with typestate pattern for compile-time state guarantees.

**States:** `New` → `Initialized` → `Running`

- **`new()`** - Creates server, optionally connects WiFi
- **`init()`** - Authenticates and connects WebSocket
- **`attach()`** - Registers sensors with reactive async handlers
- **`run()`** - Enters infinite loop, ticking sensors and sending deltas

#### signalk/auth (`src/lib/signalk/auth.rs`)

Device access token management following SignalK security spec.

- Requests device access via HTTP POST to `/signalk/v1/access/requests`
- Polls for approval, stores token in NVS (non-volatile storage)
- Token persists across reboots

#### signalk/config (`src/lib/signalk/config.rs`)

Connection configuration enums:

- **`SignalKConnection`** - WiFi config (existing, PSK, or AP mode)
- **`SignalKServerDetails`** - Server hostname and optional sensor name

### wifi (`src/lib/wifi.rs`)

WiFi connection helper using `BlockingWifi`.

- Scans for configured SSID
- Connects with WPA2 or open auth
- Waits for DHCP lease

### i2c (`src/lib/i2c.rs`)

I2C display interface for OLED displays (SSD1306, SH1106).

- **`I2CDisplayInterface`** - Factory for creating display interfaces
- **`I2CInterface`** - Implements `WriteOnlyDataCommand` for display drivers

### rgbled (`src/lib/rgbled.rs`)

WS2812 RGB LED driver using the ESP32 RMT peripheral.

- **`WS2812RMT`** - Drives addressable LEDs with precise timing
- Uses `FixedLengthSignal` for bit-banging the WS2812 protocol

## Examples

Examples are in `src/examples/`:

- **signalk** - Full SignalK integration with WiFi and WebSocket
- **constant_sensor** - Demonstrates reactive sensor subscriptions
- **scanner** - I2C bus scanner with device identification
- **ssd1306-oled** / **ssd1306-oled-bus** - OLED display examples

## Configuration

WiFi and server settings are configured via `cfg.toml` (see `cfg.toml.example`).
