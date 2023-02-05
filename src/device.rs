use crate::{Address, Driver, Error, IoWire};
use core::fmt::Debug;
use embedded_hal::blocking::delay::DelayUs;

/// Generic device interface
pub trait Device: Sized {
    /// Device family code
    const FAMILY_CODE: u8;

    /// Get device address
    fn address(&self) -> &Address;

    /// Instantiate device using address without checks
    ///
    /// # Safety
    ///
    /// This is marked as unsafe because it does not check whether the given address
    /// is compatible with a specific device. It assumes so.
    unsafe fn from_address_unchecked(address: Address) -> Self;

    /// Instantiate device from address
    fn from_address<E: Sized + Debug>(address: Address) -> Result<Self, Error<E>> {
        if address.family_code() != Self::FAMILY_CODE {
            Err(Error::FamilyCodeMismatch(
                Self::FAMILY_CODE,
                address.family_code(),
            ))
        } else {
            Ok(unsafe { Self::from_address_unchecked(address) })
        }
    }

    fn search_first<W: IoWire>(
        driver: &mut Driver<W>,
        delay: &mut impl DelayUs<u16>,
    ) -> Result<Option<Self>, Error<W::Error>> {
        Address::search_first(driver, delay, Self::FAMILY_CODE)
            .map(|res| res.map(|address| unsafe { Self::from_address_unchecked(address) }))
    }

    fn get_single<W: IoWire>(
        driver: &mut Driver<W>,
        delay: &mut impl DelayUs<u16>,
    ) -> Result<Self, Error<W::Error>> {
        let address = Address::get_single(driver, delay)?;
        Self::from_address(address)
    }
}
