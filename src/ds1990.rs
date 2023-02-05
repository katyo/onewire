use crate::{Address, Device};
use core::fmt::Debug;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Ds1990 {
    address: Address,
}

impl From<Ds1990> for Address {
    fn from(device: Ds1990) -> Self {
        device.address
    }
}

impl Default for Ds1990 {
    fn default() -> Self {
        let address = Address::from([Self::FAMILY_CODE, 0, 0, 0, 0, 0, 0, Self::FAMILY_CODE]);

        Self { address }
    }
}

impl Device for Ds1990 {
    const FAMILY_CODE: u8 = 0x01;

    fn address(&self) -> &Address {
        &self.address
    }

    unsafe fn from_address_unchecked(address: Address) -> Self {
        Self { address }
    }
}
