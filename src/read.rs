use core::fmt;
use crate::{Reading, SensorError};

pub(crate) const MAGIC_BYTE_0: u8 = 0x42;
pub(crate) const MAGIC_BYTE_1: u8 = 0x4d;
pub(crate) const PAYLOAD_LEN: usize = 32;

pub(crate) fn parse_data<E: fmt::Debug>(buf: &[u8; PAYLOAD_LEN]) -> Result<Reading, SensorError<E>> {
    let sum = buf[0..PAYLOAD_LEN-2].iter().fold(0u16, |accum, next| accum + *next as u16);
    let expected_sum: u16 = ((buf[PAYLOAD_LEN-2] as u16) << 8) | (buf[PAYLOAD_LEN-1] as u16);
    if expected_sum == sum {
        Ok(Reading {
            pm1: as_u16(buf[4], buf[5]) as f32,
            pm2_5: as_u16(buf[6], buf[7]) as f32,
            pm10: as_u16(buf[8], buf[9]) as f32,
            env_pm1: as_u16(buf[10], buf[11]) as f32,
            env_pm2_5: as_u16(buf[12], buf[13]) as f32,
            env_pm10: as_u16(buf[14], buf[15]) as f32,
            particles_0_3: as_u16(buf[16], buf[17]),
            particles_0_5: as_u16(buf[18], buf[19]),
            particles_1: as_u16(buf[20], buf[21]),
            particles_2_5: as_u16(buf[22], buf[23]),
            particles_5: as_u16(buf[24], buf[25]),
            particles_10: as_u16(buf[26], buf[27]),
        })
    } else {
        Err(SensorError::ChecksumMismatch)
    }
}

fn as_u16(hi: u8, lo: u8) -> u16 {
    ((hi as u16) << 8) | (lo as u16)
}
