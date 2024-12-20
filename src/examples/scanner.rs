use anyhow::Result;
use embedded_hal::digital::OutputPin;
use embedded_hal::digital::PinState;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_svc::hal::prelude::Peripherals;
use toml_cfg::toml_config;

use esp_idf_svc::hal::i2c::config;
use esp_idf_svc::hal::i2c::I2cDriver;

type I2cDeviceInfo = (&'static str, &'static str, &'static [u8]);

const I2C_SCANNER_KNOWN_DEVICES: [I2cDeviceInfo; 220] = [
    ("47L04/47C04/47L16/47C16", "4K/16K I2C Serial EERAM - Control register", &[0x1c, 0x18, 0x1a, 0x1e]),
    ("47L04/47C04/47L16/47C16", "4K/16K I2C Serial EERAM - SRAM Memory with EEPROM backup", &[0x54, 0x50, 0x56, 0x52]),
    ("AD5243", "Dual, 256-Position, I2 C-Compatible Digital Potentiometer", &[0x2f]),
    ("AD5248", "Dual, 256-Position, I2 C-Compatible Digital Potentiometer", &[0x2c, 0x2e, 0x2f, 0x2d]),
    ("AD5251", "Dual 64-Position I2 C Nonvolatile Memory Digital Potentiometers", &[0x2c, 0x2e, 0x2f, 0x2d]),
    ("AD5252", "Dual 256-Position I2C Nonvolatile Memory Digital Potentiometers", &[0x2c, 0x2e, 0x2f, 0x2d]),
    ("ADS1015", "4-channel 12-bit ADC", &[0x49, 0x48, 0x4b, 0x4a]),
    ("ADS1115", "4-channel 16-bit ADC", &[0x49, 0x48, 0x4b, 0x4a]),
    ("ADS7828", "12-Bit, 8-Channel Sampling ANALOG-TO-DIGITAL CONVERTER", &[0x49, 0x48, 0x4b, 0x4a]),
    ("ADXL345", "3-axis accelerometer", &[0x53, 0x1d]),
    ("AHT10", "ASAIR Humidity and Temperature sensor", &[0x38]),
    ("AHT20", "Humidity and Temperature Sensor", &[0x38]),
    ("AK8975", "3-axis magnetometer", &[0xe, 0xd, 0xc, 0xf]),
    ("AM2315", "Humidity/Temp sensor", &[0x5c]),
    ("AMG8833", "IR Thermal Camera Breakout", &[0x68, 0x69]),
    ("APDS-9250", "Digital RGB, IR and Ambient Light Sensor", &[0x52]),
    ("APDS-9960", "IR/Color/Proximity Sensor", &[0x39]),
    ("AS7262", "6-channel visible spectral_ID device with electronic shutter and smart interface", &[0x49]),
    ("AT24C02N", "Two-wire Serial EEPROM 2K (256 x 8)", &[0x57, 0x54, 0x50, 0x56, 0x53, 0x55, 0x52, 0x51]),
    ("AT24C64", "2-Wire Serial EEPROM 64K (8192 x 8)", &[0x57, 0x54, 0x50, 0x56, 0x53, 0x55, 0x52, 0x51]),
    ("ATECC508A", "Crypto Element", &[0x60]),
    ("ATECC608A", "Microchip CryptoAuthentication™ Device", &[0x60]),
    ("BH1750FVI", "Digital 16bit Serial Output Type Ambient Light Sensor IC", &[0x5c, 0x23]),
    ("BMA150", "Digital triaxial acceleration sensor", &[0x38]),
    ("BMA180", "Accelerometer", &[0x77]),
    ("BME280", "Temp/Barometric/Humidity", &[0x77, 0x76]),
    ("BME680", "Low power gas, pressure, temperature & humidity sensor", &[0x77, 0x76]),
    ("BME688", "Digital low power gas, pressure, temperature and humidity sensor with AI", &[0x77, 0x76]),
    ("BMP085", "Temp/Barometric", &[0x77]),
    ("BMP180", "Temp/Barometric", &[0x77]),
    ("BMP280", "Temp/Barometric", &[0x77, 0x76]),
    ("BNO055", "Absolute Orientation Sensor", &[0x28, 0x29]),
    ("BQ32000", "Real-Time Clock (RTC)", &[0x68]),
    ("BU9796", "Low Duty LCD Segment Drivers", &[0x3e]),
    ("CAP1188", "8-channel Capacitive Touch", &[0x2c, 0x2a, 0x2b, 0x2d, 0x28, 0x29]),
    ("CAT24C512", "EEPROM - 512Kbit - 64KB", &[0x57, 0x54, 0x50, 0x56, 0x53, 0x55, 0x52, 0x51]),
    ("CAT5171", "256‐position I2C Compatible Digital Potentiometer", &[0x2c, 0x2d]),
    ("CCS811", "Volatile organics (VOC) and equivalent CO2 (eCO2) sensor", &[0x5b, 0x5a]),
    ("CCS811", "Ultra-Low Power Digital Gas Sensor for Monitoring Indoor Air Quality TVOC eCO2", &[0x5b, 0x5a]),
    ("Chirp!", "Water sensor", &[0x20]),
    ("COM-15093", "SparkFun Qwiic Single Relay", &[0x18, 0x19]),
    ("CS43L22", "Low Power Stereo DAC w/ Headphone & Speaker Amps", &[0x4a]),
    ("D7S", "D7S Vibration Sensor", &[0x55]),
    ("DRV2605", "Haptic Motor Driver", &[0x5a]),
    ("DS1307", "64 x 8 Serial Real-Time Clock", &[0x68]),
    ("DS1371", "I2C, 32-Bit Binary Counter Watchdog Clock", &[0x68]),
    ("DS1841", "Temperature-Controlled, NV, I2C, Logarithmic Resistor", &[0x2a, 0x2b, 0x28, 0x29]),
    ("DS1881", "Dual NV Audio Taper Digital Potentiometer", &[0x2c, 0x2a, 0x2e, 0x2f, 0x2b, 0x2d, 0x28, 0x29]),
    ("DS3231", "Extremely Accurate RTC/TCXO/Crystal", &[0x68]),
    ("DS3502", "High-Voltage, NV, I2C POT", &[0x2a, 0x2b, 0x28, 0x29]),
    ("EMC2101", "SMBus Fan Control with 1°C Accurate Temperature Monitoring", &[0x4c]),
    ("FS1015", "Air Velocity Sensor Module -- 0-5, 0-15m/sec", &[0x50]),
    ("FS3000", "Air Velocity Sensor Module - 3.3V - 0-7, 0-15m/sec", &[0x28]),
    ("FT6x06", "Capacitive Touch Driver", &[0x38]),
    ("FXAS21002", "3-axis gyroscope", &[0x21, 0x20]),
    ("FXOS8700", "6-axis sensor with integrated linear accelerometer and magnetometer", &[0x1c, 0x1d, 0x1f, 0x1e]),
    ("HDC1008", "Low Power, High Accuracy Digital Humidity Sensor with Temperature Sensor", &[0x43, 0x42]),
    ("HDC1080", "Low Power, High Accuracy Digital Humidity Sensor with Temperature Sensor", &[0x40]),
    ("HIH6130", "HumidIcon", &[0x27]),
    ("HMC5883", "3-Axis Digital Compass/Magnetometer IC", &[0x1e]),
    ("HT16K33", "LED Matrix Driver", &[0x71, 0x72, 0x77, 0x73, 0x70, 0x76, 0x75, 0x74]),
    ("HTS221", "Capacitive digital sensor for relative humidity and temperature", &[0x5f]),
    ("HTU21D-F", "Humidity/Temp Sensor", &[0x40]),
    ("HTU31D", "Digital Relative Humidity & Temperature Sensor", &[0x41, 0x40]),
    ("HW-061", "I2C Serial Interface LCD1602 Adapter", &[0x23, 0x24, 0x26, 0x22, 0x27, 0x21, 0x25, 0x20]),
    ("ICM-20948", "9-Axis Motion Tracking device", &[0x68, 0x69]),
    ("INA219", "26V Bi-Directional High-Side Current/Power/Voltage Monitor", &[0x46, 0x49, 0x47, 0x48, 0x4c, 0x4d, 0x43, 0x4b, 0x44, 0x41, 0x4a, 0x45, 0x42, 0x4f, 0x4e, 0x40]),
    ("INA260", "Precision Digital Current and Power Monitor With Low-Drift, Precision Integrated Shunt", &[0x46, 0x49, 0x47, 0x48, 0x4c, 0x4d, 0x43, 0x4b, 0x44, 0x41, 0x4a, 0x45, 0x42, 0x4f, 0x4e, 0x40]),
    ("IS31FL3731", "144-LED Audio Modulated Matrix LED Driver (CharliePlex)", &[0x77, 0x66]),
    ("ISL29125", "Digital Red, Green and Blue Color Light Sensor with IR Blocking Filter", &[0x44]),
    ("IST-8310", "Three-axis Magnetometer", &[0xe]),
    ("ITG3200", "Gyro", &[0x68, 0x69]),
    ("L3GD20H", "gyroscope", &[0x6b, 0x6a]),
    ("LC709203F", "Smart LiB Gauge Battery Fuel Gauge LSI For 1‐Cell Lithium‐ion/ Polymer (Li+)", &[0x11]),
    ("LIS3DH", "3-axis accelerometer", &[0x18, 0x19]),
    ("LM25066", "PMBus power management IC", &[0x46, 0x57, 0x11, 0x54, 0x47, 0x16, 0x50, 0x10, 0x13, 0x43, 0x56, 0x5a, 0x53, 0x14, 0x44, 0x59, 0x17, 0x55, 0x41, 0x45, 0x52, 0x42, 0x58, 0x12, 0x51, 0x15, 0x40]),
    ("LM75b", "Digital temperature sensor and thermal watchdog", &[0x49, 0x48, 0x4c, 0x4d, 0x4b, 0x4a, 0x4f, 0x4e]),
    ("LPS22HB", "MEMS nano pressure sensor", &[0x2e]),
    ("LSM303", "Triple-axis Accelerometer+Magnetometer (Compass)", &[0x19, 0x1e]),
    ("LSM303", "Triple-axis Accelerometer+Magnetometer (Compass)", &[0x18, 0x1e]),
    ("LTC4151", "High voltage (7-80V) current and voltage monitor", &[0x6c, 0x6e, 0x6b, 0x6a, 0x68, 0x6f, 0x6d, 0x66, 0x69, 0x67]),
    ("MA12070P", "Merus Multi level Class D Interated amplifier", &[0x23, 0x22, 0x21, 0x20]),
    ("MAG3110", "3-Axis Magnetometer", &[0xe]),
    ("MAX17048", "3μA 1-Cell/2-Cell Fuel Gauge with ModelGauge", &[0x36]),
    ("MAX17048", "3μA 1-Cell/2-Cell Fuel Gauge with ModelGauge", &[0x36]),
    ("MAX30101", "High-Sensitivity Pulse Oximeter and Heart-Rate Sensor for Wearable Health", &[0x55]),
    ("MAX3010x", "Pulse & Oximetry sensor", &[0x57]),
    ("MAX31341", "Low-Current, Real-Time Clock with I2C Interface and Power Management", &[0x69]),
    ("MAX44009", "Ambient Light Sensor with ADC", &[0x4b, 0x4a]),
    ("MB85RC", "Ferroelectric RAM", &[0x57, 0x54, 0x50, 0x56, 0x53, 0x55, 0x52, 0x51]),
    ("MCP23008", "8-Bit I/O Expander with Serial Interface I2C GPIO expander", &[0x23, 0x24, 0x26, 0x22, 0x27, 0x21, 0x25, 0x20]),
    ("MCP23017", "I2C GPIO expander", &[0x23, 0x24, 0x26, 0x22, 0x27, 0x21, 0x25, 0x20]),
    ("MCP3422", "18-Bit, Multi-Channel ΔΣ Analog-to-Digital Converter with I2CTM Interface and On-Board Reference", &[0x68]),
    ("MCP4532", "7/8-Bit Single/Dual I2C Digital POT with Volatile Memory", &[0x2c, 0x2a, 0x2e, 0x2f, 0x2b, 0x2d, 0x28, 0x29]),
    ("MCP4725A0", "12-bit DAC", &[0x60, 0x61]),
    ("MCP4725A1", "12-Bit Digital-to-Analog Converter with EEPROM Memory", &[0x64, 0x60, 0x61, 0x65, 0x63, 0x66, 0x62, 0x67]),
    ("MCP4725A2", "12-Bit Digital-to-Analog Converter with EEPROM Memory", &[0x64, 0x65]),
    ("MCP4725A3", "12-Bit Digital-to-Analog Converter with EEPROM Memory", &[0x66, 0x67]),
    ("MCP4728", "12-Bit 4-Channel Digital-to-Analog Converter (DAC) with EEPROM", &[0x64, 0x60, 0x61, 0x65, 0x63, 0x66, 0x62, 0x67]),
    ("MCP7940N", "Battery-Backed I2C Real-Time Clock/Calendar with SRAM", &[0x6f]),
    ("MCP9808", "±0.5°C Maximum Accuracy Digital Temperature Sensor", &[0x1c, 0x1d, 0x18, 0x1b, 0x1f, 0x1a, 0x19, 0x1e]),
    ("MLX90614", "IR temperature sensor", &[0x5a]),
    ("MLX90632", "FIR temperature sensor", &[0x3a]),
    ("MLX90640", "Far infrared thermal sensor array (32x24 RES)", &[0x33]),
    ("MMA845x", "3-axis, 14-bit/8-bit digital accelerometer", &[0x1c, 0x1d]),
    ("MPL115A2", "Miniature I2C digital barometer, 50 to 115 kPa", &[0x60]),
    ("MPL3115A2", "Barometric Pressure", &[0x60]),
    ("MPR121", "12-point capacitive touch sensor", &[0x5c, 0x5b, 0x5a, 0x5d]),
    ("MPU6050", "Six-Axis (Gyro + Accelerometer) MEMS MotionTracking™ Devices", &[0x68, 0x69]),
    ("MPU-9250", "9-DoF IMU Gyroscope, Accelerometer and Magnetometer", &[0x68, 0x69]),
    ("MPU-9250", "3-Axis Gyroscope and Accelerometer", &[0x68]),
    ("MS5607", "Barometric Pressure", &[0x77, 0x76]),
    ("MS5611", "Barometric Pressure", &[0x77, 0x76]),
    ("NE5751", "Audio processor for IV communication", &[0x41, 0x40]),
    ("Nunchuck controller", "Nintendo", &[0x52]),
    ("PCA1070", "Multistandard programmable analog CMOS speech transmission IC", &[0x22]),
    ("PCA6408A", "Low-voltage, 8-bit I2C-bus and SMBus I/O expander", &[0x21, 0x20]),
    ("PCA9536", "4-bit 2.3- to 5.5-V I2C/SMBus I/O expander with config registers", &[0x41]),
    ("PCA9539", "16-bit I/O expander with interrupt and reset", &[0x77, 0x76, 0x75, 0x74]),
    ("PCA9541", "2-1 I2C bus arbiter", &[0x71, 0x72, 0x77, 0x73, 0x70, 0x76, 0x75, 0x74]),
    ("PCA9685", "16-channel PWM driver default address", &[0x5e, 0x64, 0x6c, 0x60, 0x6e, 0x46, 0x6b, 0x57, 0x49, 0x6a, 0x5c, 0x5b, 0x54, 0x47, 0x48, 0x4c, 0x50, 0x4d, 0x68, 0x43, 0x61, 0x56, 0x4b, 0x5a, 0x53, 0x65, 0x44, 0x7f, 0x71, 0x72, 0x78, 0x7a, 0x77, 0x59, 0x55, 0x41, 0x4a, 0x73, 0x63, 0x6f, 0x6d, 0x66, 0x5d, 0x45, 0x5f, 0x7b, 0x70, 0x69, 0x76, 0x52, 0x42, 0x4f, 0x58, 0x7e, 0x75, 0x74, 0x4e, 0x79, 0x62, 0x51, 0x67, 0x7d, 0x7c, 0x40]),
    ("PCD3311C", "DTMF/modem/musical tone generator", &[0x24, 0x25]),
    ("PCD3312C", "DTMF/modem/musical-tone generator", &[0x24, 0x25]),
    ("PCF8523", "RTC", &[0x68]),
    ("PCF8563", "Real-time clock/calendar", &[0x51]),
    ("PCF8569", "LCD column driver for dot matrix displays", &[0x3c, 0x3b]),
    ("PCF8573", "Clock/calendar with Power Fail Detector", &[0x6b, 0x6a, 0x68, 0x69]),
    ("PCF8574", "Remote 8-Bit I/O Expander", &[0x46, 0x49, 0x47, 0x48, 0x4c, 0x4d, 0x43, 0x4b, 0x44, 0x41, 0x4a, 0x45, 0x42, 0x4f, 0x4e, 0x40]),
    ("PCF8574", "Remote 8-Bit I/O Expander for I2C Bus", &[0x23, 0x24, 0x26, 0x22, 0x27, 0x21, 0x25, 0x20]),
    ("PCF8574AP", "I²C-bus to parallel port expander", &[0x38, 0x3c, 0x3b, 0x3e, 0x3f, 0x39, 0x3a, 0x3d]),
    ("PCF8575", "Remote16-BIT I2C AND SMBus I/O Expander withInterrupt Output", &[0x23, 0x24, 0x26, 0x22, 0x27, 0x21, 0x25, 0x20]),
    ("PCF8577C", "32/64-segment LCD display driver", &[0x3a]),
    ("PCF8578", "Row/column LCD dot matrix driver/display", &[0x3c, 0x3d]),
    ("PM2008", "Laser particle sensor", &[0x28]),
    ("PMSA003I", "Digital universal partical concentration sensor", &[0x12]),
    ("PN532", "NFC/RFID reader", &[0x48]),
    ("SAA1064", "4-digit LED driver", &[0x38, 0x3b, 0x39, 0x3a]),
    ("SAA2502", "MPEG audio source decoder", &[0x30, 0x31]),
    ("SAA4700", "VPS Dataline Processor", &[0x23, 0x21]),
    ("SAA5243P/E", "Computer controlled teletext circuit", &[0x11]),
    ("SAA5243P/H", "Computer controlled teletext circuit", &[0x11]),
    ("SAA5243P/K", "Computer controlled teletext circuit", &[0x11]),
    ("SAA5243P/L", "Computer controlled teletext circuit", &[0x11]),
    ("SAA5246", "Integrated VIP and teletext", &[0x11]),
    ("SAA7706H", "Car radio Digital Signal Processor (DSP)", &[0x1c]),
    ("SAB3035", "Digital tuning circuit for computer-controlled TV", &[0x60, 0x61, 0x63, 0x62]),
    ("SAB3037", "Digital tuning circuit for computer-controlled TV", &[0x60, 0x61, 0x63, 0x62]),
    ("SCD30", "CO2, humidity, and temperature sensor", &[0x61]),
    ("SCD40", "CO2 sensor - 2000ppm", &[0x62]),
    ("SCD40-D-R2", "Miniaturized CO2 Sensor", &[0x62]),
    ("SCD41", "CO2 sensor", &[0x62]),
    ("SEN-15892", "Zio Qwiic Loudness Sensor", &[0x38]),
    ("SEN-17374", "Sparkfun EKMC4607112K PIR", &[0x13, 0x12]),
    ("SFA30", "Formaldehyde Sensor Module for HVAC and Indoor Air Quality Applications", &[0x5d]),
    ("SGP30", "Gas Sensor", &[0x58]),
    ("SGP40", "Indoor Air Quality Sensor for VOC Measurements", &[0x59]),
    ("SH1106", "132 X 64 Dot Matrix OLED/PLED  Preliminary Segment/Common Driver with Controller", &[0x3c, 0x3d]),
    ("SHT31", "Humidity/Temp sensor", &[0x44, 0x45]),
    ("SHTC3", "Humidity & Temperature Sensor", &[0x70]),
    ("SI1132", "UV Index and Ambient Light Sensor", &[0x60]),
    ("SI1133", "UV Index and Ambient Light Sensor", &[0x55, 0x52]),
    ("Si1145", "Proximity/UV/Ambient Light Sensor IC With I2C Interface", &[0x60]),
    ("Si4713", "FM Radio Transmitter with Receive Power Scan", &[0x11, 0x63]),
    ("Si5351A", "Clock Generator", &[0x60, 0x61]),
    ("Si7021", "Humidity/Temp sensor", &[0x40]),
    ("SPL06-007", "Digital Temperature/Pressure Sensor", &[0x77, 0x76]),
    ("SPS30", "Particulate Matter Sensor for Air Quality Monitoring and Control", &[0x69]),
    ("SSD1305", "132 x 64 Dot Matrix OLED/PLED Segment/Common Driver with Controller", &[0x3c, 0x3d]),
    ("SSD1306", "128 x 64 Dot Matrix Monochrome OLED/PLED Segment/Common Driver with Controller", &[0x3c, 0x3d]),
    ("ST25DV16K", "Dynamic NFC/RFID tag IC with 4-, 16-, or 64-Kbit EEPROM, and fast transfer mode capability", &[0x57, 0x53, 0x2d]),
    ("STDS75", "STDS75 temperature sensor", &[0x49, 0x48, 0x4c, 0x4d, 0x4b, 0x4a, 0x4f, 0x4e]),
    ("STMPE610", "Resistive Touch controller", &[0x44, 0x41]),
    ("STMPE811", "Resistive touchscreen controller", &[0x44, 0x41]),
    ("TCA9548", "1-to-8 I2C Multiplexer", &[0x71, 0x72, 0x77, 0x73, 0x70, 0x76, 0x75, 0x74]),
    ("TCA9548A", "Low-Voltage8-Channel I2CSwitchwithReset", &[0x71, 0x72, 0x77, 0x73, 0x70, 0x76, 0x75, 0x74]),
    ("TCA9554", "4 Low Voltage 8-Bit I 2C and SMBus Low-Power I/O Expander With Interrupt Output and Configuration Registers", &[0x23, 0x24, 0x26, 0x22, 0x27, 0x21, 0x25, 0x20]),
    ("TCS34725", "Color sensor", &[0x29]),
    ("TDA4670", "Picture signal improvement circuit", &[0x44]),
    ("TDA4671", "Picture signal improvement circuit", &[0x44]),
    ("TDA4672", "Picture signal improvement (PSI) circuit", &[0x44]),
    ("TDA4680", "Video processor", &[0x44]),
    ("TDA4687", "Video processor", &[0x44]),
    ("TDA4688", "Video processor", &[0x44]),
    ("TDA4780", "Video control with gamma control", &[0x44]),
    ("TDA7433", "Basic function audio processor", &[0x45]),
    ("TDA8370", "High/medium perf. sync. processor", &[0x46]),
    ("TDA8376", "One-chip multistandard video", &[0x45]),
    ("TDA8415", "TVNCR stereo/dual sound processor", &[0x42]),
    ("TDA8417", "TVNCR stereo/dual sound processor", &[0x42]),
    ("TDA8421", "Audio processor with loudspeaker and headphone channel", &[0x41, 0x40]),
    ("TDA8424", "Audio processor with loudspeaker channel", &[0x41]),
    ("TDA8425", "Audio processor with loudspeaker channel", &[0x41]),
    ("TDA8426", "Hi-fi stereo audio processor", &[0x41]),
    ("TDA8442", "Interface for colour decoder", &[0x44]),
    ("TDA9150", "Deflection processor", &[0x46]),
    ("TDA9860", "Hi-fi audio processor", &[0x41, 0x40]),
    ("TEA5767", "Radio receiver", &[0x60]),
    ("TEA6100", "FM/IF for computer-controlled radio", &[0x61]),
    ("TEA6300", "Sound fader control and preamplifier/source selector", &[0x40]),
    ("TEA6320", "4-input tone/volume controller with fader control", &[0x40]),
    ("TEA6330", "Sound fader control circuit for car radios", &[0x40]),
    ("TMP006", "Infrared Thermopile Sensor in Chip-Scale Package", &[0x46, 0x47, 0x43, 0x44, 0x41, 0x45, 0x42, 0x40]),
    ("TMP007", "IR Temperature sensor", &[0x46, 0x47, 0x43, 0x44, 0x41, 0x45, 0x42, 0x40]),
    ("TMP102", "Temperature sensor", &[0x49, 0x48, 0x4b, 0x4a]),
    ("TPA2016", "2.8-W/Ch Stereo Class-D Audio Amplifier With Dynamic Range Compression and Automatic Gain Control", &[0x58]),
    ("TSA5511", "1.3 GHz PLL frequency synthesizer for TV", &[0x60, 0x61, 0x63, 0x62]),
    ("TSL2561", "Light sensor", &[0x49, 0x39]),
    ("TSL2591", "Light sensor", &[0x29]),
    ("UMA1014T", "Low-power frequency synthesizer for mobile radio communications", &[0x63, 0x62]),
    ("VCNL40x0", "Proximity sensor", &[0x13]),
    ("VCNL4200", "High Sensitivity Long Distance Proximity and Ambient Light Sensor With I2C Interface", &[0x51]),
    ("VEML6070", "UVA Light Sensor with I2C Interface", &[0x38, 0x39]),
    ("VEML6075", "UVA and UVB Light Sensor", &[0x10]),
    ("VEML7700", "High Accuracy Ambient Light Sensor", &[0x10]),
    ("VL53L0x", "Time Of Flight distance sensor", &[0x29]),
    ("VL6180X", "Time Of Flight distance sensor", &[0x29]),
    ("VML6075", "UVA and UVB Light Sensor with I2C Interface", &[0x10]),
    ("WITTY PI 3", "WITTY PI 3 (Mini) - REALTIME CLOCK (DS3231SN) AND POWER MANAGEMENT FOR RASPBERRY PI", &[0x68, 0x69]),
    ("XD8574", "I²C 8-Bit I/O Expander", &[0x71, 0x72, 0x77, 0x73, 0x70, 0x76, 0x75, 0x74]),
    ("XD8574A", "I²C 8-Bit I/O Expander", &[0x23, 0x24, 0x26, 0x22, 0x27, 0x21, 0x25, 0x20]),
];

fn lookup(addr: u8) {
    for i in 0..I2C_SCANNER_KNOWN_DEVICES.len() {
        let addresses = I2C_SCANNER_KNOWN_DEVICES[i].2;
        for j in 0..addresses.len() {
            if addr == addresses[j] {
                println!(
                    "  {}:  {}",
                    I2C_SCANNER_KNOWN_DEVICES[i].0, I2C_SCANNER_KNOWN_DEVICES[i].1
                );
            }
        }
    }
}

#[derive(Debug)]
#[toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

fn main() -> Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let mut power_pin = PinDriver::output(peripherals.pins.gpio4)?;
    let mut led = PinDriver::output(peripherals.pins.gpio2)?;
    power_pin.set_high()?;
    led.set_high()?;

    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio22;

    log::info!("Preparing to initialize...");
    led.set_low()?;

    let config = config::Config::default();
    log::info!("{:?}", &config);

    // Initialize I2C driver
    let mut i2c = I2cDriver::new(peripherals.i2c1, sda, scl, &config)?;

    for addr in 0..=127 {
        let mut buf = [0; 32];
        match led.set_state(match led.is_set_high() {
            true => PinState::Low,
            false => PinState::High,
        }) {
            Ok(_) => (),
            Err(e) => log::error!("Failed to set LED pin! {:?}", e),
        };

        // Scan Address
        // in the copied implementation there were two different scans happening here,
        // with one being SMBus.  SMBus is not implemented for esp-idf-hal at this time,
        // so we do what we can
        if (addr >= 0x30 && addr <= 0x37) || (addr >= 0x50 && addr <= 0x57) {
            match i2c.write_read(addr, &[0], &mut buf, 100) {
                Ok(_) => {
                    println!("Found Address {:#02x}", addr as u8);
                    lookup(addr as u8);
                }
                Err(_e) => {
                    //log::error!("Error on scan! Addr: {:?} Error: {:?}", &addr, e);
                    continue;
                }
            }
        } else {
            match i2c.write_read(addr, &[0], &mut buf, 100) {
                Ok(_) => {
                    println!("Found Address {:#02x}", addr as u8);
                    lookup(addr as u8);
                }
                Err(_e) => {
                    //log::error!("Error on scan! Addr: {:?} Error: {:?}", &addr, e);
                    continue;
                }
            }
        }
    }

    return Result::Ok(());
}
