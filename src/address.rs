use crate::{Command, DeviceSearch, Driver, Error, IoWire, OpCode};
use core::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    ops::{Deref, DerefMut},
    str::FromStr,
};
use embedded_hal::delay::DelayUs;

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
#[repr(transparent)]
pub struct Address {
    raw: [u8; Self::BYTES as usize],
}

impl Default for Address {
    fn default() -> Self {
        Self::from([0; Self::BYTES as usize])
    }
}

impl From<[u8; Self::BYTES as usize]> for Address {
    fn from(raw: [u8; Self::BYTES as usize]) -> Self {
        Address { raw }
    }
}

impl From<Address> for [u8; Address::BYTES as usize] {
    fn from(addr: Address) -> [u8; Address::BYTES as usize] {
        addr.raw
    }
}

impl Deref for Address {
    type Target = [u8; Self::BYTES as usize];

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl DerefMut for Address {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.raw
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        self.deref() as _
    }
}

impl AsMut<[u8]> for Address {
    fn as_mut(&mut self) -> &mut [u8] {
        self.deref_mut() as _
    }
}

impl Address {
    /// The length of device address in bytes
    pub const BYTES: u8 = 8;

    /// The length of device address in bits
    pub const BITS: u8 = Self::BYTES * 8;

    pub fn family_code(&self) -> u8 {
        self[0]
    }

    pub fn ensure_correct_crc8<E: Debug>(&self, data: &[u8], crc8: u8) -> Result<(), Error<E>> {
        let computed = self.compute_crc8(data);
        if computed != crc8 {
            Err(Error::CrcMismatch(computed, crc8))
        } else {
            Ok(())
        }
    }

    pub fn compute_crc8(&self, data: &[u8]) -> u8 {
        let crc = super::compute_partial_crc8(0u8, self.as_ref());
        super::compute_partial_crc8(crc, data)
    }
}

/// Error type
#[derive(Debug)]
pub enum AddressError {
    NotEnough,
    Invalid,
}

fn hex_to_u8(c: char) -> Option<u8> {
    //let b = c as u32;
    if c.is_ascii_digit() {
        Some((c as u32 - '0' as u32) as _)
    } else if ('a'..='f').contains(&c) {
        Some((c as u32 - 'a' as u32 + 10) as _)
    } else if ('A'..='F').contains(&c) {
        Some((c as u32 - 'A' as u32 + 10) as _)
    } else {
        None
    }
}

impl FromStr for Address {
    type Err = AddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut addr = Address::default();
        let mut chars = s.chars().filter(|c| !c.is_whitespace() && *c != ':');

        for i in 0..Self::BYTES as usize {
            match (chars.next(), chars.next()) {
                (Some(h), Some(l)) => match (hex_to_u8(h), hex_to_u8(l)) {
                    (Some(h), Some(l)) => {
                        addr[i] = (h << 4) | l;
                    }
                    _ => return Err(AddressError::Invalid),
                },
                _ => return Err(AddressError::NotEnough),
            }
        }

        Ok(addr)
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self[0], self[1], self[2], self[3], self[4], self[5], self[6], self[7],
        )
    }
}

impl Address {
    pub fn read_single<W: IoWire>(
        &mut self,
        driver: &mut Driver<W>,
        delay: &mut impl DelayUs,
    ) -> Result<(), Error<W::Error>> {
        driver.reset_write_read(delay, &[Command::ReadRom.op_code()], self.as_mut())?;
        Ok(())
    }

    pub fn get_single<W: IoWire>(
        driver: &mut Driver<W>,
        delay: &mut impl DelayUs,
    ) -> Result<Self, Error<W::Error>> {
        let mut address = Self::default();
        address.read_single(driver, delay)?;
        Ok(address)
    }

    pub fn search_first<W: IoWire>(
        driver: &mut Driver<W>,
        delay: &mut impl DelayUs,
        family_code: u8,
    ) -> Result<Option<Self>, Error<W::Error>> {
        let mut search = DeviceSearch::new();
        while let Some(address) = driver.search_next(&mut search, delay)? {
            if family_code == address.family_code() {
                return Ok(Some(address));
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod test {
    use super::Address;

    #[test]
    fn parse_address() {
        let addr: Address = "01228ff908000168".parse().unwrap();

        assert_eq!(
            addr,
            Address::from([0x01, 0x22, 0x8f, 0xf9, 0x08, 0x00, 0x01, 0x68])
        );
    }

    #[test]
    fn parse_address_space_separated() {
        let addr: Address = "01 22 8f f9 08 00 01 68".parse().unwrap();

        assert_eq!(
            addr,
            Address::from([0x01, 0x22, 0x8f, 0xf9, 0x08, 0x00, 0x01, 0x68])
        );
    }

    #[test]
    fn parse_address_colon_separated() {
        let addr: Address = "01:22:8f:f9:08:00:01:68".parse().unwrap();

        assert_eq!(
            addr,
            Address::from([0x01, 0x22, 0x8f, 0xf9, 0x08, 0x00, 0x01, 0x68])
        );
    }
}
