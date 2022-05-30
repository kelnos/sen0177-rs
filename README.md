# sen0177

[![crates.io][crates-shield]][crates-url]
[![Documentation][docs-shield]][docs-url]
[![Apache 2.0][license-shield]][license-url]
[![Build Status][build-shield]][build-url]

`sen0177` is a Rust library/crate that reads air quality data from the
SEN0177 air quality sensor.

## Prerequisites

* You've connected the sensor to a UART or I2C bus on your device, and
  your device has a crate implementing the applicable [`embedded_hal`]
  traits.
* For a UART-based sensor, you've configured the UART for 9600 baud, 8
  data bits, no parity, 1 stop bit, and no flow control.

## Setup

Include the following in your `Cargo.toml` file:

```toml
[dependencies]
sen0177 = "0.4"
```

If you are in a `no_std` environment, you may depend on this crate like so:

```toml
[dependencies]
sen0177 = { version = "0.4", default-features = false }
```

## Usage

This example shows how to use the sensor when connected to a Linux-
based serial device.

Note that this example currently does not work becuase this crate is
tracking the current embedded-hal 1.0.0 alpha, but linux-embedded-hal
is a little behind at the time of writing.  If you want to use my patched
version, add the following to your `Cargo.toml`:

```toml
[patch.crates-io]
linux-embedded-hal = { git = "https://github.com/kelnos/linux-embedded-hal.git", branch = "embedded-hal-1.0.0-alpha.8" }
```

```rust,no_run,ignore
use linux_embedded_hal::Serial;
use sen0177::{serial::Sen0177, Reading};
use serial::{core::prelude::*, BaudRate, CharSize, FlowControl, Parity, StopBits};
use std::{io, path::Path, time::Duration};

const SERIAL_PORT: &str = "/dev/ttyS0";
const BAUD_RATE: BaudRate = BaudRate::Baud9600;
const CHAR_SIZE: CharSize = CharSize::Bits8;
const PARITY: Parity = Parity::ParityNone;
const STOP_BITS: StopBits = StopBits::Stop1;
const FLOW_CONTROL: FlowControl = FlowControl::FlowNone;

pub fn main() -> std::io::Result<()> {
    let mut serial = Serial::open(&Path::new(SERIAL_PORT))?;
    serial.0.set_timeout(Duration::from_millis(1500))?;
    serial.0.reconfigure(&|settings| {
        settings.set_char_size(CHAR_SIZE);
        settings.set_parity(PARITY);
        settings.set_stop_bits(STOP_BITS);
        settings.set_flow_control(FLOW_CONTROL);
        settings.set_baud_rate(BAUD_RATE)
    })?;
    let mut sensor = Sen0177::new(serial);

    let reading = sensor.read().expect("Failed to read sensor data");
    println!("PM1: {}µg/m³, PM2.5: {}µg/m³, PM10: {}µg/m³",
             reading.pm1(), reading.pm2_5(), reading.pm10());
    Ok(())
}
```

Note that the serial device occasionally returns bad data.  If you
receive [`SensorError::BadMagic`] or [`SensorError::ChecksumMismatch`]
from the [`AirQualitySensor::read`] call, a second try will usually succeed.

## Gotchas

### Raspberry Pi

If you're using this with a Raspberry Pi, note that by default the
primary UART is set up as a Linux serial console.  You will need
to disable that (by editing `/boot/cmdline.txt`) before this will work.
Instead of using a specifiy TTY device node, you should use
`/dev/serial0`, which is a symlink to the proper device.

Alternatively, you can use the second UART, but you'll need to load an
overlay to assign it to GPIO pins.  See [UART
configuration](https://www.raspberrypi.org/documentation/configuration/uart.md)
and the [UART-related
overlays](https://www.raspberrypi.org/documentation/configuration/uart.md)
for more information.

[crates-shield]: https://img.shields.io/crates/v/sen0177.svg
[crates-url]: https://crates.io/crates/sen0177
[docs-shield]: https://docs.rs/sen0177/badge.svg
[docs-url]: https://docs.rs/sen0177
[license-shield]: https://img.shields.io/crates/l/sen0177.svg
[license-url]: https://github.com/kelnos/sen0177-rs/blob/maim/LICENSE
[build-shield]: https://img.shields.io/github/workflow/status/kelnos/sen0177-rs/CI
[build-url]: https://github.com/kelnos/sen0177-rs/actions
