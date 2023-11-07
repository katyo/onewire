use byteorder::{ByteOrder, LittleEndian};
use embedded_hal::delay::DelayUs;

use crate::{Address, Device, Driver, Error, IoWire, OpCode, Sensor};
use core::fmt::Debug;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Command {
    Convert = 0x44,
    WriteScratchpad = 0x4e,
    ReadScratchpad = 0xBE,
    CopyScratchpad = 0x48,
    RecallE2 = 0xB8,
    ReadPowerSupply = 0xB4,
}

impl OpCode for Command {
    fn op_code(&self) -> u8 {
        *self as _
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum MeasureResolution {
    TC8 = 0b0001_1111,
    TC4 = 0b0011_1111,
    TC2 = 0b0101_1111,
    TC = 0b0111_1111,
}

impl MeasureResolution {
    pub fn time_ms(&self) -> u16 {
        match self {
            MeasureResolution::TC8 => 94,
            MeasureResolution::TC4 => 188,
            MeasureResolution::TC2 => 375,
            MeasureResolution::TC => 750,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ds18b20 {
    address: Address,
    resolution: MeasureResolution,
}

impl From<Ds18b20> for Address {
    fn from(device: Ds18b20) -> Self {
        device.address
    }
}

impl Ds18b20 {
    pub fn measure_temperature<W: IoWire>(
        &self,
        driver: &mut Driver<W>,
        delay: &mut impl DelayUs,
    ) -> Result<MeasureResolution, Error<W::Error>> {
        driver.reset_select_write_only(delay, &self.address, &[Command::Convert.op_code()])?;
        Ok(self.resolution)
    }

    pub fn read_temperature<W: IoWire>(
        &self,
        driver: &mut Driver<W>,
        delay: &mut impl DelayUs,
    ) -> Result<u16, Error<W::Error>> {
        let mut scratchpad = [0u8; 9];
        driver.reset_select_write_read(
            delay,
            &self.address,
            &[Command::ReadScratchpad.op_code()],
            &mut scratchpad[..],
        )?;
        self.address
            .ensure_correct_crc8(&scratchpad[..8], scratchpad[8])?;
        Ok(Self::read_temperature_from_scratchpad(&scratchpad))
    }

    fn read_temperature_from_scratchpad(scratchpad: &[u8]) -> u16 {
        LittleEndian::read_u16(&scratchpad[0..2])
    }
}

impl Device for Ds18b20 {
    const FAMILY_CODE: u8 = 0x28;

    fn address(&self) -> &Address {
        &self.address
    }

    unsafe fn from_address_unchecked(address: Address) -> Self {
        Self {
            address,
            resolution: MeasureResolution::TC,
        }
    }
}

impl Sensor for Ds18b20 {
    fn start_measurement<W: IoWire>(
        &self,
        driver: &mut Driver<W>,
        delay: &mut impl DelayUs,
    ) -> Result<u16, Error<W::Error>> {
        Ok(self.measure_temperature(driver, delay)?.time_ms())
    }

    fn read_measurement<W: IoWire>(
        &self,
        driver: &mut Driver<W>,
        delay: &mut impl DelayUs,
    ) -> Result<f32, Error<W::Error>> {
        self.read_temperature(driver, delay)
            .map(|t| t as i16 as f32 / 16_f32)
    }

    fn read_measurement_raw<W: IoWire>(
        &self,
        driver: &mut Driver<W>,
        delay: &mut impl DelayUs,
    ) -> Result<u16, Error<W::Error>> {
        self.read_temperature(driver, delay)
    }
}

/// Split raw u16 value to two parts: integer and fraction N
/// Original value may be calculated as: integer + fraction/10000
pub fn split_temp(temperature: u16) -> (i16, i16) {
    if temperature < 0x8000 {
        (temperature as i16 >> 4, (temperature as i16 & 0xF) * 625)
    } else {
        let abs = -(temperature as i16);
        (-(abs >> 4), -625 * (abs & 0xF))
    }
}

#[cfg(test)]
mod tests {
    use super::split_temp;
    #[test]
    fn test_temp_conv() {
        assert_eq!(split_temp(0x07d0), (125, 0));
        assert_eq!(split_temp(0x0550), (85, 0));
        assert_eq!(split_temp(0x0191), (25, 625)); // 25.0625
        assert_eq!(split_temp(0x00A2), (10, 1250)); // 10.125
        assert_eq!(split_temp(0x0008), (0, 5000)); // 0.5
        assert_eq!(split_temp(0x0000), (0, 0)); // 0
        assert_eq!(split_temp(0xfff8), (0, -5000)); // -0.5
        assert_eq!(split_temp(0xFF5E), (-10, -1250)); // -10.125
        assert_eq!(split_temp(0xFE6F), (-25, -625)); // -25.0625
        assert_eq!(split_temp(0xFC90), (-55, 0)); // -55
    }
}
