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
//! * You've connected the sensor to a UART on your device, and your device
//!   has a crate implementing the [`embedded_hal::serial::Read`] trait.
//! * You've configured the UART for 9600 baud, 8 data bits, no parity, 1
//!   stop bit, and no flow control.
//! 
//! ## Setup
//! 
//! Include the following in your `Cargo.toml` file:
//! 
//! ```toml
//! [dependencies]
//! sen0177 = "0.2"
//! ```
//!
//! If you are in a `no_std` environment, you may depend on this crate like so:
//!
//! ```toml
//! [dependencies]
//! sen0177 = { version = "0.2", default-features = false }
//! ```
//! 
//! ## Usage
//!
//! This example shows how to use the sensor when connected to a Linux-
//! based serial device.
//! 
//! ```rust,no_run
//! use linux_embedded_hal::Serial;
//! use sen0177::Reading;
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
//!
//!     let reading = sen0177::read(&mut serial).expect("Failed to read sensor data");
//!     println!("PM1: {}µg/m³, PM2.5: {}µg/m³, PM10: {}µg/m³",
//!              reading.pm1(), reading.pm2_5(), reading.pm10());
//!     Ok(())
//! }
//! ```
//! 
//! Note that the serial device occasionally returns bad data.  If you
//! receive [`Sen0177Error::InvalidData`] or [`Sen0177Error::ChecksumMismatch`]
//! from the [`read`] call, a second try will usually succeed.
//! 
//! ## Gotchas
//! 
//! ### Raspberry Pi
//! 
//! If you're using this with a Raspberry Pi, note that by default the
//! primary GPIO pins are set up as a Linux serial console.  You will need
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

#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[macro_use(block)]
extern crate nb;

use core::fmt;
use embedded_hal::serial::Read;

const MAGIC_BYTE_0: u8 = 0x42;
const MAGIC_BYTE_1: u8 = 0x4d;
const PAYLOAD_LEN: usize = 30;  // 32 - magic bytes

/// A single air quality sensor reading
#[derive(Debug, Clone, Copy)]
pub struct Reading {
    pm1: f32,
    pm2_5: f32,
    pm10: f32,
}

impl Reading {
    /// Returns the PM1 concentration in µg/m³
    pub fn pm1(&self) -> f32 {
        self.pm1
    }

    /// Returns the PM2.5 concentration in µg/m³
    pub fn pm2_5(&self) -> f32 {
        self.pm2_5
    }

    /// Returns the PM10 concentration in µg/m³
    pub fn pm10(&self) -> f32 {
        self.pm10
    }
}

/// Describes errors returned by the SEN0177 sensor
#[derive(Debug)]
pub enum Sen0177Error<E> {
    /// Device returned invalid data
    InvalidData(&'static str),
    /// The checksum provided in the sensor data did not match the checksum of the data itself
    ///
    /// Retrying the read will usually clear up the problem.
    ChecksumMismatch,
    /// Read error from the serial device
    ReadError(E),
}

impl<E: fmt::Debug> fmt::Display for Sen0177Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Sen0177Error::*;
        match self {
            InvalidData(reason) => write!(f, "Invalid data: {}", reason),
            ChecksumMismatch => write!(f, "Data read was corrupt"),
            ReadError(error) => write!(f, "Read error: {:?}", error),
        }
    }
}

#[cfg(feature = "std")]
impl<E: fmt::Debug + fmt::Display> std::error::Error for Sen0177Error<E> {}

impl<E> From<E> for Sen0177Error<E> {
    fn from(error: E) -> Self {
        Sen0177Error::ReadError(error)
    }
}

#[doc(hidden)]
pub trait SerialReader<E>: Read<u8, Error = E> {}

impl<E, T> SerialReader<E> for T
where
    T: Read<u8, Error = E>
{}

/// Reads a single sensor measurement
///
/// This function will block until sufficient data is available.
///
/// # Arguments
///
/// * `serial_port` - a struct implementing the [`embedded_hal::serial::Read<u8>`] trait
pub fn read<E, R>(serial_port: &mut R) -> Result<Reading, Sen0177Error<E>>
where
    R: SerialReader<E>
{
    let mut attempts_left = 10;
    let mut byte_read = 0u8;
    while byte_read != MAGIC_BYTE_1 && attempts_left > 0 && find_byte(serial_port, MAGIC_BYTE_0, PAYLOAD_LEN as u32 * 4)? {
        byte_read = block!(serial_port.read())?;
        attempts_left -= 1;
    }

    if byte_read == MAGIC_BYTE_1 {
        let mut buf: [u8; PAYLOAD_LEN] = [0; PAYLOAD_LEN];
        for buf_slot in buf.iter_mut() {
            *buf_slot = block!(serial_port.read())?;
        }
        validate_checksum(&buf)?;

        Ok(Reading {
            pm1: (((buf[2] as u16) << 8) | (buf[3] as u16)) as f32,
            pm2_5: (((buf[4] as u16) << 8) | (buf[5] as u16)) as f32,
            pm10: (((buf[6] as u16) << 8) | (buf[7] as u16)) as f32,
        })
    } else {
        Err(Sen0177Error::InvalidData("Unable to find magic bytes at start of payload"))
    }
}

fn find_byte<R, E>(serial_port: &mut R, byte: u8, attempts: u32) -> Result<bool, Sen0177Error<E>>
where
    R: SerialReader<E>
{
    let mut attempts_left = attempts;
    let mut byte_read = 0u8;
    while byte_read != byte && attempts_left > 0 {
        byte_read = block!(serial_port.read())?;
        attempts_left -= 1;
    }
    Ok(byte_read == byte)
}

fn validate_checksum<E>(buf: &[u8; PAYLOAD_LEN]) -> Result<(), Sen0177Error<E>> {
    let init: u16 = (MAGIC_BYTE_0 as u16) + (MAGIC_BYTE_1 as u16);
    let sum = buf[0..PAYLOAD_LEN-2].iter().fold(init, |accum, next| accum + *next as u16);
    let expected_sum: u16 = ((buf[PAYLOAD_LEN-2] as u16) << 8) | (buf[PAYLOAD_LEN-1] as u16);
    if expected_sum == sum {
        Ok(())
    } else {
        Err(Sen0177Error::ChecksumMismatch)
    }
}
