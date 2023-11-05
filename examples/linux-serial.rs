use linux_embedded_hal::{
    serialport::{self, DataBits, FlowControl, Parity, StopBits},
    Serial,
};
use sen0177::{serial::Sen0177, AirQualitySensor};
use std::time::Duration;

const SERIAL_PORT: &str = "/dev/ttyS0";
const BAUD_RATE: u32 = 9600;
const DATA_BITS: DataBits = DataBits::Eight;
const PARITY: Parity = Parity::None;
const STOP_BITS: StopBits = StopBits::One;
const FLOW_CONTROL: FlowControl = FlowControl::None;

pub fn main() -> anyhow::Result<()> {
    let builder = serialport::new(SERIAL_PORT, BAUD_RATE)
        .data_bits(DATA_BITS)
        .flow_control(FLOW_CONTROL)
        .parity(PARITY)
        .stop_bits(STOP_BITS)
        .timeout(Duration::from_millis(1500));
    let serial = Serial::open_from_builder(builder)?;
    let mut sensor = Sen0177::new(serial);

    loop {
        match sensor.read() {
            Ok(reading) => {
                println!(
                    "PM1: {}µg/m³, PM2.5: {}µg/m³, PM10: {}µg/m³",
                    reading.pm1(),
                    reading.pm2_5(),
                    reading.pm10()
                );
            }
            Err(err) => eprintln!("Error: {:?}", err),
        }
    }
}
