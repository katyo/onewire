#![no_std]
#![doc = include_str!("../README.md")]

mod address;
mod command;
mod device;
mod driver;
#[cfg(feature = "ds18b20")]
pub mod ds18b20;
#[cfg(feature = "ds1990")]
pub mod ds1990;
mod iowire;
mod result;
#[cfg(feature = "rw1990")]
pub mod rw1990;
mod search;
mod sensor;

pub use address::Address;
pub use command::{Command, OpCode};
pub use device::Device;
pub use driver::Driver;
pub use iowire::{Inverted, IoWire};
pub use result::Error;
pub use search::{DeviceSearch, DeviceSearchIter};
pub use sensor::Sensor;

pub fn compute_partial_crc8(crc: u8, data: &[u8]) -> u8 {
    let mut crc = crc;
    for byte in data.iter() {
        let mut byte = *byte;
        for _ in 0..8 {
            let mix = (crc ^ byte) & 0x01;
            crc >>= 1;
            if mix != 0x00 {
                crc ^= 0x8C;
            }
            byte >>= 1;
        }
    }
    crc
}
