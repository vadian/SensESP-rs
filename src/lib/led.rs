//! Unified LED interface supporting both simple GPIO LEDs and WS2812 RGB LEDs.
//!
//! Select your board with Cargo features:
//! - `led-simple` (default): Simple GPIO LED (ESP32 DevKit v1.0 on GPIO2)
//! - `led-ws2812`: WS2812 RGB LED (FireBeetle ESP32-E on GPIO5)

use anyhow::Result;

/// LED color representation
#[derive(Clone, Copy, Debug)]
pub struct LedColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl LedColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const fn off() -> Self {
        Self { r: 0, g: 0, b: 0 }
    }

    /// Check if color is "on" (any component > 0)
    pub fn is_on(&self) -> bool {
        self.r > 0 || self.g > 0 || self.b > 0
    }

    // Common colors
    pub const RED: Self = Self::new(50, 0, 0);
    pub const GREEN: Self = Self::new(0, 50, 0);
    pub const BLUE: Self = Self::new(0, 0, 50);
    pub const YELLOW: Self = Self::new(50, 50, 0);
    pub const WHITE: Self = Self::new(50, 50, 50);
}

/// Unified LED trait
pub trait Led {
    fn set_color(&mut self, color: LedColor) -> Result<()>;

    fn on(&mut self) -> Result<()> {
        self.set_color(LedColor::WHITE)
    }

    fn off(&mut self) -> Result<()> {
        self.set_color(LedColor::off())
    }
}

// ============================================================================
// Simple GPIO LED (ESP32 DevKit v1.0)
// ============================================================================

#[cfg(feature = "led-simple")]
mod simple {
    use super::*;
    use esp_idf_hal::gpio::{AnyOutputPin, Output, PinDriver};

    pub struct SimpleLed<'d> {
        pin: PinDriver<'d, Output>,
    }

    impl<'d> SimpleLed<'d> {
        pub fn new(pin: AnyOutputPin<'d>) -> Result<Self> {
            let mut pin = PinDriver::output(pin)?;
            pin.set_low()?;
            Ok(Self { pin })
        }
    }

    impl Led for SimpleLed<'_> {
        fn set_color(&mut self, color: LedColor) -> Result<()> {
            if color.is_on() {
                self.pin.set_high()?;
            } else {
                self.pin.set_low()?;
            }
            Ok(())
        }
    }
}

#[cfg(feature = "led-simple")]
pub use simple::SimpleLed;

// ============================================================================
// WS2812 RGB LED (FireBeetle ESP32-E)
// ============================================================================

#[cfg(feature = "led-ws2812")]
mod ws2812 {
    use super::*;
    use crate::rgbled::{RGB8, WS2812RMT};
    use esp_idf_hal::gpio::AnyOutputPin;
    use esp_idf_hal::rmt::RmtChannel;

    pub struct Ws2812Led<'d> {
        driver: WS2812RMT<'d>,
    }

    impl<'d> Ws2812Led<'d> {
        pub fn new<C: RmtChannel + 'd>(pin: AnyOutputPin<'d>, channel: C) -> Result<Self> {
            let driver = WS2812RMT::new(pin, channel)?;
            Ok(Self { driver })
        }
    }

    impl Led for Ws2812Led<'_> {
        fn set_color(&mut self, color: LedColor) -> Result<()> {
            self.driver
                .set_pixel(RGB8::new(color.r, color.g, color.b))?;
            Ok(())
        }
    }
}

#[cfg(feature = "led-ws2812")]
pub use ws2812::Ws2812Led;
