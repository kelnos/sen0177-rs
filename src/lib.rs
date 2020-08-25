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
//! * You've connected the sensor to a UART on a Linux-based device, and
//!   that UART is enabled and available from the kernel as a TTY device
//!   node.
//! 
//! ## Installation
//! 
//! Include the following in your `Cargo.toml` file:
//! 
//! ```toml
//! [dependencies]
//! sen0177 = "0.1"
//! ```
//! 
//! ## Usage
//! 
//! ```rust,no_run
//! use sen0177::{Reading, Sen0177};
//! 
//! const SERIAL_PORT: &str = "/dev/ttyS0";
//! 
//! let mut sensor = Sen0177::open(SERIAL_PORT).expect("Failed to open device");
//! let reading = sensor.read().expect("Failed to read sensor data");
//! println!("PM1: {}µg/m³, PM2.5: {}µg/m³, PM10: {}µg/m³",
//!          reading.pm1(), reading.pm2_5(), reading.pm10());
//! ```
//! 
//! Note that the serial device occasionally returns bad data.  If you
//! recieve `Sen0177::InvalidData` or `Sen0177::ChecksumMismatch` from the
//! `read()` call, a second try will usually succeed.
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

use serial::{SystemPort, open as serial_open};
use serial_core::{BaudRate, CharSize, Error as SerialError, ErrorKind as SerialErrorKind, FlowControl, Parity, Result as SerialResult, SerialPort, StopBits};
use std::error::Error;
use std::fmt;
use std::io;
use std::io::Read;
use std::path::Path;
use std::time::Duration;

const BAUD_RATE: BaudRate = BaudRate::Baud9600;
const CHAR_SIZE: CharSize = CharSize::Bits8;
const PARITY: Parity = Parity::ParityNone;
const STOP_BITS: StopBits = StopBits::Stop1;
const FLOW_CONTROL: FlowControl = FlowControl::FlowNone;

const MAGIC_BYTE_0: u8 = 0x42;
const MAGIC_BYTE_1: u8 = 0x4d;
const PAYLOAD_LEN: usize = 30;  // 32 - magic bytes

/// A single air quality sensor reading
#[derive(Debug, Clone, Copy)]
pub struct Reading {
    pm1: f64,
    pm2_5: f64,
    pm10: f64,
}

impl Reading {
    /// Returns the PM1 concentration in µg/m³
    pub fn pm1(&self) -> f64 {
        self.pm1
    }

    /// Returns the PM2.5 concentration in µg/m³
    pub fn pm2_5(&self) -> f64 {
        self.pm2_5
    }

    /// Returns the PM10 concentration in µg/m³
    pub fn pm10(&self) -> f64 {
        self.pm10
    }
}

/// Describes errors returned by the SEN0177 sensor
#[derive(Debug)]
pub enum Sen0177Error {
    /// Device not found on the specified port
    DeviceNotFound,
    /// Device is in use or does not support required port configuration parameters
    DeviceUnavailable,
    /// Device returned invalid data
    InvalidData(String),
    /// The checksum provided in the sensor data did not match the checksum of the data itself
    ///
    /// Retrying the read will usually clear up the problem.
    ChecksumMismatch,
    /// An IO error occurred when communicating with the serial port
    IoError(io::Error),

}

impl fmt::Display for Sen0177Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Sen0177Error::*;
        match self {
            DeviceNotFound => write!(f, "Device not found"),
            DeviceUnavailable => write!(f, "Device unavailable or does not support required parameters"),
            InvalidData(reason) => write!(f, "Invalid data: {}", reason),
            ChecksumMismatch => write!(f, "Data read was corrupt"),
            IoError(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl Error for Sen0177Error {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Sen0177Error::IoError(ioerr) => Some(ioerr),
            _ => None,
        }
    }
}

impl From<SerialError> for Sen0177Error {
    fn from(err: SerialError) -> Self {
        use Sen0177Error::*;
        match err.kind() {
            SerialErrorKind::NoDevice => DeviceNotFound,
            SerialErrorKind::InvalidInput => DeviceUnavailable,
            SerialErrorKind::Io(kind) => IoError(io::Error::new(kind, format!("{:?}", kind))),
        }
    }
}

impl From<io::Error> for Sen0177Error {
    fn from(err: io::Error) -> Self {
        Sen0177Error::IoError(err)
    }
}

/// An instance of the SEN0177 air quality sensor
pub struct Sen0177 {
    serial_port: SystemPort,
}

impl Sen0177 {
    /// Opens the sensor on the specified port
    ///
    /// # Arguments
    ///
    /// * `serial_dev` - The serial device node the sensor is connected to
    pub fn open<P: AsRef<Path>>(serial_dev: P) -> Result<Sen0177, Sen0177Error> {
        let mut serial_port = serial_open(serial_dev.as_ref())?;
        Sen0177::configure_serial_port(&mut serial_port)?;
        Ok(Sen0177 {
            serial_port,
        })
    }

    /// Reads a single sensor measurement
    ///
    /// Note that this function will block until sufficient data is available.
    pub fn read(&mut self) -> Result<Reading, Sen0177Error> {
        let mut attempts_left = 10;
        let mut buf: [u8; 1] = [0; 1];
        while buf[0] != MAGIC_BYTE_1 && attempts_left > 0 && find_byte(&mut self.serial_port, MAGIC_BYTE_0, PAYLOAD_LEN as u32 * 4)? {
            self.serial_port.read_exact(&mut buf)?;
            attempts_left -= 1;
        }

        if buf[0] == MAGIC_BYTE_1 {
            let mut buf: [u8; PAYLOAD_LEN] = [0; PAYLOAD_LEN];
            self.serial_port.read_exact(&mut buf)?;
            validate_checksum(&buf)?;

            Ok(Reading {
                pm1: (((buf[2] as u16) << 8) | (buf[3] as u16)) as f64,
                pm2_5: (((buf[4] as u16) << 8) | (buf[5] as u16)) as f64,
                pm10: (((buf[6] as u16) << 8) | (buf[7] as u16)) as f64,
            })
        } else {
            Err(Sen0177Error::InvalidData(format!("Unable to find start magic 0x{:2x}{:2x}", MAGIC_BYTE_0, MAGIC_BYTE_1)))
        }
    }

    fn configure_serial_port(serial_port: &mut SystemPort) -> SerialResult<()> {
        serial_port.set_timeout(Duration::from_millis(1500))?;
        serial_port.reconfigure(&|settings| {
            settings.set_char_size(CHAR_SIZE);
            settings.set_parity(PARITY);
            settings.set_stop_bits(STOP_BITS);
            settings.set_flow_control(FLOW_CONTROL);
            settings.set_baud_rate(BAUD_RATE)
        })
    }
}

fn find_byte(serial_port: &mut SystemPort, byte: u8, attempts: u32) -> io::Result<bool> {
    let mut attempts_left = attempts;
    let mut buf: [u8; 1] = [0; 1];
    while buf[0] != byte && attempts_left > 0 {
        serial_port.read_exact(&mut buf)?;
        attempts_left -= 1;
    }
    Ok(buf[0] == byte)
}

fn validate_checksum(buf: &[u8; PAYLOAD_LEN]) -> Result<(), Sen0177Error> {
    let init: u16 = (MAGIC_BYTE_0 as u16) + (MAGIC_BYTE_1 as u16);
    let sum = buf[0..PAYLOAD_LEN-2].iter().fold(init, |accum, next| accum + *next as u16);
    let expected_sum: u16 = ((buf[PAYLOAD_LEN-2] as u16) << 8) | (buf[PAYLOAD_LEN-1] as u16);
    if expected_sum == sum {
        Ok(())
    } else {
        Err(Sen0177Error::ChecksumMismatch)
    }
}
