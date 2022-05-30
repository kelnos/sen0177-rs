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

#![doc = include_str!("../README.md")]
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
            BadMagic => f.write_str("Unable to find magic bytes at start of payload"),
            ChecksumMismatch => f.write_str("Data read was corrupt"),
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
