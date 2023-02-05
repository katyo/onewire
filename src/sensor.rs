use crate::{Device, Driver, Error, IoWire};
use embedded_hal::blocking::delay::DelayUs;

pub trait Sensor: Device {
    /// returns the milliseconds required to wait until the measurement finished
    fn start_measurement<W: IoWire>(
        &self,
        wire: &mut Driver<W>,
        delay: &mut impl DelayUs<u16>,
    ) -> Result<u16, Error<W::Error>>;

    /// returns the measured value
    fn read_measurement<W: IoWire>(
        &self,
        wire: &mut Driver<W>,
        delay: &mut impl DelayUs<u16>,
    ) -> Result<f32, Error<W::Error>>;

    fn read_measurement_raw<W: IoWire>(
        &self,
        wire: &mut Driver<W>,
        delay: &mut impl DelayUs<u16>,
    ) -> Result<u16, Error<W::Error>>;
}
