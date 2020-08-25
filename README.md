# sen0177

`sen0177` is a Rust library/crate that reads air quality data from the
SEN0177 air quality sensor.

[![crates.io][crates-shield]][crates-url]
[![Documentation][docs-shield]][docs-url]
[![Apache 2.0][license-shield]][license-url]
[![Build Status][build-shield]][build-url]

## Prerequisites

* You've connected the sensor to a UART on a Linux-based device, and
  that UART is enabled and available from the kernel as a TTY device
  node.

## Installation

Include the following in your `Cargo.toml` file:

```toml
[dependencies]
sen0177 = "0.1"
```

## Usage

```rust,no_run
use sen0177::{Reading, Sen0177};

const SERIAL_PORT: &str = "/dev/ttyS0";

let mut sensor = Sen0177::open(SERIAL_PORT).expect("Failed to open device");
let reading = sensor.read().expect("Failed to read sensor data");
println!("PM1: {}µg/m³, PM2.5: {}µg/m³, PM10: {}µg/m³",
         reading.pm1(), reading.pm2_5(), reading.pm10());
```

Note that the serial device occasionally returns bad data.  If you
recieve `Sen0177::InvalidData` or `Sen0177::ChecksumMismatch` from the
`read()` call, a second try will usually succeed.

## Gotchas

### Raspberry Pi

If you're using this with a Raspberry Pi, note that by default the
primary GPIO pins are set up as a Linux serial console.  You will need
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
[license-url]: https://github.com/kelnos/sen0177/blob/master/LICENSE
[build-shield]: https://img.shields.io/github/workflow/status/kelnos/sen0177-rs/CI
[build-url]: https://github.com/kelnos/sen0177-rs/actions
