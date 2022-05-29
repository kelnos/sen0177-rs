use embedded_hal::i2c::{
    blocking::I2c,
    AddressMode,
    Error as I2cError
};
use crate::{
    read::*,
    AirQualitySensor,
    Reading,
    SensorError,
};

/// A SEN0177 device connected via I2C
pub struct Sen0177<A, I2C, E>
where
    A: AddressMode + Copy,
    I2C: I2c<A, Error = E>,
    E: I2cError,
{
    i2c_bus: I2C,
    address: A,
}

impl<A, I2C, E> Sen0177<A, I2C, E>
where
    A: AddressMode + Copy,
    I2C: I2c<A, Error = E>,
    E: I2cError,
{
    /// Creates a new sensor instance connected to I2C bus `i2c_bus` at address `address`
    pub fn new(i2c_bus: I2C, address: A) -> Self {
        Self {
            i2c_bus,
            address,
        }
    }
}

impl<A, I2C, E> AirQualitySensor<E> for Sen0177<A, I2C, E>
where
    A: AddressMode + Copy,
    I2C: I2c<A, Error = E>,
    E: I2cError,
{
    fn read(&mut self) -> Result<Reading, SensorError<E>> {
        let mut buf: [u8; PAYLOAD_LEN] = [0; PAYLOAD_LEN];
        if let Err(err) = self.i2c_bus.read(self.address, &mut buf) {
            Err(err.into())
        } else {
            if buf[0] != MAGIC_BYTE_0 || buf[1] != MAGIC_BYTE_1 {
                Err(SensorError::BadMagic)
            } else {
                parse_data(&buf)
            }
        }
    }
}
