use embedded_hal::{
    nb::block,
    serial::{
        Error as SerialError,
        nb::Read,
    },
};
use crate::{
    read::*,
    AirQualitySensor,
    Reading,
    SensorError,
};

/// A SEN0177 device connected via serial UART
pub struct Sen0177<R, E>
where
    R: Read<u8, Error = E>,
    E: SerialError,
{
    serial_port: R,
}

impl<R, E> Sen0177<R, E>
where
    R: Read<u8, Error = E>,
    E: SerialError,
{
    /// Creates a new sensor instance connected to UART `serial_port`
    pub fn new(serial_port: R) -> Self {
        Self {
            serial_port,
        }
    }

    fn find_byte(&mut self, byte: u8, attempts: u32) -> Result<bool, SensorError<E>> {
        let mut attempts_left = attempts;
        let mut byte_read = 0u8;
        while byte_read != byte && attempts_left > 0 {
            byte_read = block!(self.serial_port.read())?;
            attempts_left -= 1;
        }
        Ok(byte_read == byte)
    }
}

impl<R, E> AirQualitySensor<E> for Sen0177<R, E>
where
    R: Read<u8, Error = E>,
    E: SerialError,
{
    fn read(&mut self) -> Result<Reading, SensorError<E>> {
        let mut attempts_left = 10;
        let mut byte_read = 0u8;
        while byte_read != MAGIC_BYTE_1 && attempts_left > 0 && self.find_byte(MAGIC_BYTE_0, PAYLOAD_LEN as u32 * 4)? {
            byte_read = block!(self.serial_port.read())?;
            attempts_left -= 1;
        }

        if byte_read == MAGIC_BYTE_1 {
            let mut buf: [u8; PAYLOAD_LEN] = [0; PAYLOAD_LEN];
            buf[0] = MAGIC_BYTE_0;
            buf[1] = MAGIC_BYTE_1;
            for buf_slot in buf[2..PAYLOAD_LEN].iter_mut() {
                *buf_slot = block!(self.serial_port.read())?;
            }

            parse_data(&buf)
        } else {
            Err(SensorError::InvalidData("Unable to find magic bytes at start of payload"))
        }
    }
}
