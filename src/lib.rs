// Copyright 2020 Brian J. Tarricone <brian@tarricone.org>
// 
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// 
//     http://www.apache.org/licenses/LICENSE-2.0
// 
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! `sen0177` is a Rust library/crate that reads air quality data from the
//! SEN0177 air quality sensor.
//! 
//! ## Prerequisites
//! 
//! * You've connected the sensor to a UART or I2C bus on your device, and
//!   your device has a crate implementing the applicable [`embedded_hal`]
//!   traits.
//! * For a UART-based sensor, you've configured the UART for 9600 baud, 8
//!   data bits, no parity, 1 stop bit, and no flow control.
//! 
//! ## Setup
//! 
//! Include the following in your `Cargo.toml` file:
//! 
//! ```toml
//! [dependencies]
//! sen0177 = "0.4"
//! ```
//!
//! If you are in a `no_std` environment, you may depend on this crate like so:
//!
//! ```toml
//! [dependencies]
//! sen0177 = { version = "0.4", default-features = false }
//! ```
//! 
//! ## Usage
//!
//! This example shows how to use the sensor when connected to a Linux-
//! based serial device.
//!
//! Note that this example currently does not work becuase this crate is
//! tracking the current embedded-hal 1.0.0 alpha, but linux-embedded-hal
//! is a little behind at the time of writing.  If you want to use my patched
//! version, add the following to your `Cargo.toml`:
//!
//! ```toml
//! [patch.crates-io]
//! linux-embedded-hal = { git = "https://github.com/kelnos/linux-embedded-hal.git", branch = "embedded-hal-1.0.0-alpha.8" }
//! ```
//! 
//! ```rust,no_run,ignore
//! use linux_embedded_hal::Serial;
//! use sen0177::{serial::Sen0177, Reading};
//! use serial::{core::prelude::*, BaudRate, CharSize, FlowControl, Parity, StopBits};
//! use std::{io, path::Path, time::Duration};
//!
//! const SERIAL_PORT: &str = "/dev/ttyS0";
//! const BAUD_RATE: BaudRate = BaudRate::Baud9600;
//! const CHAR_SIZE: CharSize = CharSize::Bits8;
//! const PARITY: Parity = Parity::ParityNone;
//! const STOP_BITS: StopBits = StopBits::Stop1;
//! const FLOW_CONTROL: FlowControl = FlowControl::FlowNone;
//!
//! pub fn main() -> std::io::Result<()> {
//!     let mut serial = Serial::open(&Path::new(SERIAL_PORT))?;
//!     serial.0.set_timeout(Duration::from_millis(1500))?;
//!     serial.0.reconfigure(&|settings| {
//!         settings.set_char_size(CHAR_SIZE);
//!         settings.set_parity(PARITY);
//!         settings.set_stop_bits(STOP_BITS);
//!         settings.set_flow_control(FLOW_CONTROL);
//!         settings.set_baud_rate(BAUD_RATE)
//!     })?;
//!     let mut sensor = Sen0177::new(serial);
//!
//!     let reading = sensor.read().expect("Failed to read sensor data");
//!     println!("PM1: {}µg/m³, PM2.5: {}µg/m³, PM10: {}µg/m³",
//!              reading.pm1(), reading.pm2_5(), reading.pm10());
//!     Ok(())
//! }
//! ```
//! 
//! Note that the serial device occasionally returns bad data.  If you
//! receive [`SensorError::BadMagic`] or [`SensorError::ChecksumMismatch`]
//! from the [`AirQualitySensor::read`] call, a second try will usually succeed.
//! 
//! ## Gotchas
//! 
//! ### Raspberry Pi
//! 
//! If you're using this with a Raspberry Pi, note that by default the
//! primary UART is set up as a Linux serial console.  You will need
//! to disable that (by editing `/boot/cmdline.txt`) before this will work.
//! Instead of using a specifiy TTY device node, you should use
//! `/dev/serial0`, which is a symlink to the proper device.
//! 
//! Alternatively, you can use the second UART, but you'll need to load an
//! overlay to assign it to GPIO pins.  See [UART
//! configuration](https://www.raspberrypi.org/documentation/configuration/uart.md)
//! and the [UART-related
//! overlays](https://www.raspberrypi.org/documentation/configuration/uart.md)
//! for more information.

#![cfg_attr(not(feature = "std"), no_std)]

/// Sensors connected to the I2C bus
pub mod i2c;
pub(crate) mod read;
/// Sensors connected to a serial UART
pub mod serial;

use core::fmt;

/// Trait representing a bus-agnostic air quality sensor
pub trait AirQualitySensor<E: fmt::Debug> {
    /// Reads a single sensor measurement
    ///
    /// This function will block until sufficient data is available.
    fn read(&mut self) -> Result<Reading, SensorError<E>>;
}

/// A single air quality sensor reading
#[derive(Debug, Clone, Copy)]
pub struct Reading {
    pm1: u16,
    pm2_5: u16,
    pm10: u16,
    env_pm1: u16,
    env_pm2_5: u16,
    env_pm10: u16,
    particles_0_3: u16, 
    particles_0_5: u16,      
    particles_1: u16,      
    particles_2_5: u16,      
    particles_5: u16,      
    particles_10: u16,     
}

impl Reading {
    /// Returns the standard PM1 concentration in µg/m³
    pub fn pm1(&self) -> u16 {
        self.pm1
    }

    /// Returns the standard PM2.5 concentration in µg/m³
    pub fn pm2_5(&self) -> u16 {
        self.pm2_5
    }

    /// Returns the standard PM10 concentration in µg/m³
    pub fn pm10(&self) -> u16 {
        self.pm10
    }

    /// Returns the environmental PM1 concentration in µg/m³
    ///
    /// Note that some devices do not support this reading and will
    /// return garbage data for this value.
    pub fn env_pm1(&self) -> u16 {
        self.env_pm1
    }

    /// Returns the environmental PM2.5 concentration in µg/m³
    ///
    /// Note that some devices do not support this reading and will
    /// return garbage data for this value.
    pub fn env_pm2_5(&self) -> u16 {
        self.env_pm2_5
    }

    /// Returns the environmental PM10 concentration in µg/m³
    ///
    /// Note that some devices do not support this reading and will
    /// return garbage data for this value.
    pub fn env_pm10(&self) -> u16 {
        self.env_pm10
    }

    /// Returns count of particles smaller than 0.3µm
    pub fn particles_0_3(&self) -> u16 {
        self.particles_0_3
    }

    /// Returns count of particles smaller than 0.5µm
    pub fn particles_0_5(&self) -> u16 {
        self.particles_0_5
    }

    /// Returns count of particles smaller than 1µm
    pub fn particles_1(&self) -> u16 {
        self.particles_1
    }

    /// Returns count of particles smaller than 2.5µm
    pub fn particles_2_5(&self) -> u16 {
        self.particles_2_5
    }

    /// Returns count of particles smaller than 5µm
    pub fn particles_5(&self) -> u16 {
        self.particles_5
    }

    /// Returns count of particles smaller than 10µm
    pub fn particles_10(&self) -> u16 {
        self.particles_10
    }
}

/// Describes errors returned by the air quality sensor
#[derive(Debug)]
pub enum SensorError<E: fmt::Debug> {
    /// Couldn't find the "magic" bytes that indicate the start of a data frame
    ///
    /// This likely means that you've set an incorrect baud rate, or there is something
    /// noisy about your connection to the device.
    BadMagic,
    /// The checksum provided in the sensor data did not match the checksum of the data itself
    ///
    /// Retrying the read will usually clear up the problem.
    ChecksumMismatch,
    /// Read error from the serial device or I2C bus
    ReadError(E),
}

impl<E: fmt::Debug> fmt::Display for SensorError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use SensorError::*;
        match self {
            BadMagic => write!(f, "Unable to find magic bytes at start of payload"),
            ChecksumMismatch => write!(f, "Data read was corrupt"),
            ReadError(error) => write!(f, "Read error: {:?}", error),
        }
    }
}

#[cfg(feature = "std")]
impl<E: fmt::Debug> std::error::Error for SensorError<E> {}

impl<E: fmt::Debug> From<E> for SensorError<E> {
    fn from(error: E) -> Self {
        SensorError::ReadError(error)
    }
}
