use crate::{Address, Command, Driver, Error, IoWire};
use core::fmt::Debug;
use embedded_hal::delay::DelayUs;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum SearchState {
    #[default]
    Initialized,
    DeviceFound,
    End,
}

#[derive(Clone, Default)]
pub struct DeviceSearch {
    address: [u8; 8],
    discrepancies: [u8; 8],
    state: SearchState,
}

impl DeviceSearch {
    pub fn new() -> DeviceSearch {
        DeviceSearch::default()
    }

    pub fn new_for_family(family: u8) -> DeviceSearch {
        let mut search = DeviceSearch::new();
        search.address[0] = family;
        search
    }

    fn is_bit_set_in_address(&self, bit: u8) -> bool {
        DeviceSearch::is_bit_set(&self.address, bit)
    }

    fn set_bit_in_address(&mut self, bit: u8) {
        DeviceSearch::set_bit(&mut self.address, bit);
    }

    fn reset_bit_in_address(&mut self, bit: u8) {
        DeviceSearch::reset_bit(&mut self.address, bit);
    }

    fn write_bit_in_address(&mut self, bit: u8, value: bool) {
        if value {
            self.set_bit_in_address(bit);
        } else {
            self.reset_bit_in_address(bit);
        }
    }

    fn is_bit_set_in_discrepancies(&self, bit: u8) -> bool {
        DeviceSearch::is_bit_set(&self.discrepancies, bit)
    }

    fn set_bit_in_discrepancy(&mut self, bit: u8) {
        DeviceSearch::set_bit(&mut self.discrepancies, bit);
    }

    fn reset_bit_in_discrepancy(&mut self, bit: u8) {
        DeviceSearch::reset_bit(&mut self.discrepancies, bit);
    }

    #[allow(unused)] // useful method anyway?
    fn write_bit_in_discrepancy(&mut self, bit: u8, value: bool) {
        if value {
            self.set_bit_in_discrepancy(bit);
        } else {
            self.reset_bit_in_discrepancy(bit);
        }
    }

    fn is_bit_set(array: &[u8], bit: u8) -> bool {
        if bit / 8 >= array.len() as u8 {
            return false;
        }
        let index = bit / 8;
        let offset = bit % 8;
        array[index as usize] & (0x01 << offset) != 0x00
    }

    fn set_bit(array: &mut [u8], bit: u8) {
        if bit / 8 >= array.len() as u8 {
            return;
        }
        let index = bit / 8;
        let offset = bit % 8;
        array[index as usize] |= 0x01 << offset
    }

    fn reset_bit(array: &mut [u8], bit: u8) {
        if bit / 8 >= array.len() as u8 {
            return;
        }
        let index = bit / 8;
        let offset = bit % 8;
        array[index as usize] &= !(0x01 << offset)
    }

    pub fn last_discrepancy(&self) -> Option<u8> {
        let mut result = None;
        for i in 0..Address::BITS {
            if self.is_bit_set_in_discrepancies(i) {
                result = Some(i);
            }
        }
        result
    }

    pub fn into_iter<'a, W: IoWire>(
        self,
        wire: &'a mut Driver<W>,
        delay: &'a mut impl DelayUs,
    ) -> DeviceSearchIter<'a, W, impl DelayUs> {
        DeviceSearchIter {
            search: Some(self),
            wire,
            delay,
        }
    }
}

pub struct DeviceSearchIter<'a, W: IoWire, Delay: DelayUs> {
    search: Option<DeviceSearch>,
    wire: &'a mut Driver<W>,
    delay: &'a mut Delay,
}

impl<'a, W: IoWire, Delay: DelayUs> Iterator for DeviceSearchIter<'a, W, Delay> {
    type Item = Result<Address, Error<W::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut search = self.search.take()?;
        let result = self
            .wire
            .search_next(&mut search, &mut *self.delay)
            .transpose()?;
        self.search = Some(search);
        Some(result)
    }
}

impl<E: Debug, W: IoWire<Error = E>> Driver<W> {
    /// Heavily inspired by https://github.com/ntruchsess/arduino-Driver/blob/85d1aae63ea4919c64151e03f7e24c2efbc40198/Driver.cpp#L362
    pub(crate) fn search(
        &mut self,
        rom: &mut DeviceSearch,
        delay: &mut impl DelayUs,
        cmd: Command,
    ) -> Result<Option<Address>, Error<E>> {
        if SearchState::End == rom.state {
            return Ok(None);
        }

        let mut discrepancy_found = false;
        let last_discrepancy = rom.last_discrepancy();

        if !self.reset_presence(delay)? {
            return Ok(None);
        }

        self.write_byte(delay, cmd as u8, false)?;

        if let Some(last_discrepancy) = last_discrepancy {
            // walk previous path
            for i in 0..last_discrepancy {
                let bit0 = self.read_bit(delay)?;
                let bit1 = self.read_bit(delay)?;

                if bit0 && bit1 {
                    // no device responded
                    return Ok(None);
                } else {
                    let bit = rom.is_bit_set_in_address(i);
                    // rom.write_bit_in_address(i, bit0);
                    // rom.write_bit_in_discrepancy(i, bit);
                    self.write_bit(delay, bit)?;
                }
            }
        } else {
            // no discrepancy and device found, meaning the one found is the only one
            if rom.state == SearchState::DeviceFound {
                rom.state = SearchState::End;
                return Ok(None);
            }
        }

        for i in last_discrepancy.unwrap_or(0)..Address::BITS {
            let bit0 = self.read_bit(delay)?; // normal bit
            let bit1 = self.read_bit(delay)?; // complementar bit

            if last_discrepancy.eq(&Some(i)) {
                // be sure to go different path from before (go second path, thus writing 1)
                rom.reset_bit_in_discrepancy(i);
                rom.set_bit_in_address(i);
                self.write_bit(delay, true)?;
            } else {
                if bit0 && bit1 {
                    // no response received
                    return Ok(None);
                }

                if !bit0 && !bit1 {
                    // addresses with 0 and 1
                    // found new path, go first path by default (thus writing 0)
                    discrepancy_found |= true;
                    rom.set_bit_in_discrepancy(i);
                    rom.reset_bit_in_address(i);
                    self.write_bit(delay, false)?;
                } else {
                    // addresses only with bit0
                    rom.write_bit_in_address(i, bit0);
                    self.write_bit(delay, bit0)?;
                }
            }
        }

        if !discrepancy_found && rom.last_discrepancy().is_none() {
            rom.state = SearchState::End;
        } else {
            rom.state = SearchState::DeviceFound;
        }
        Ok(Some(Address::from(rom.address)))
    }
}
