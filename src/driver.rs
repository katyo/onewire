use crate::{Address, Command, DeviceSearch, Error, IoWire, OpCode};
use core::fmt::Debug;
use embedded_hal::delay::DelayUs;

pub struct Driver<W: IoWire> {
    io_wire: W,
    pub(crate) parasite_mode: bool,
}

impl<E: Debug, W: IoWire<Error = E>> Driver<W> {
    pub fn new(io_wire: W, parasite_mode: bool) -> Self {
        Driver {
            io_wire,
            parasite_mode,
        }
    }

    pub fn reset_write_read(
        &mut self,
        delay: &mut impl DelayUs,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Error<E>> {
        self.reset(delay)?;
        self.write_bytes(delay, write)?;
        self.read_bytes(delay, read)?;
        Ok(())
    }

    pub fn reset_read_only(
        &mut self,
        delay: &mut impl DelayUs,
        read: &mut [u8],
    ) -> Result<(), Error<E>> {
        self.reset(delay)?;
        self.read_bytes(delay, read)?;
        Ok(())
    }

    pub fn reset_write_only(
        &mut self,
        delay: &mut impl DelayUs,
        write: &[u8],
    ) -> Result<(), Error<E>> {
        self.reset(delay)?;
        self.write_bytes(delay, write)?;
        Ok(())
    }

    pub fn reset_select_write_read(
        &mut self,
        delay: &mut impl DelayUs,
        addr: &Address,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Error<E>> {
        self.reset(delay)?;
        self.select(delay, addr)?;
        self.write_bytes(delay, write)?;
        self.read_bytes(delay, read)?;
        Ok(())
    }

    pub fn reset_select_read_only(
        &mut self,
        delay: &mut impl DelayUs,
        addr: &Address,
        read: &mut [u8],
    ) -> Result<(), Error<E>> {
        self.reset(delay)?;
        self.select(delay, addr)?;
        self.select(delay, addr)?;
        self.read_bytes(delay, read)?;
        Ok(())
    }

    pub fn reset_select_write_only(
        &mut self,
        delay: &mut impl DelayUs,
        addr: &Address,
        write: &[u8],
    ) -> Result<(), Error<E>> {
        self.reset(delay)?;
        self.select(delay, addr)?;
        self.select(delay, addr)?;
        self.write_bytes(delay, write)?;
        Ok(())
    }

    pub fn reset_skip_write_read(
        &mut self,
        delay: &mut impl DelayUs,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Error<E>> {
        self.reset(delay)?;
        self.skip(delay)?;
        self.write_bytes(delay, write)?;
        self.read_bytes(delay, read)?;
        Ok(())
    }

    pub fn reset_skip_read_only(
        &mut self,
        delay: &mut impl DelayUs,
        read: &mut [u8],
    ) -> Result<(), Error<E>> {
        self.reset(delay)?;
        self.skip(delay)?;
        self.read_bytes(delay, read)?;
        Ok(())
    }

    pub fn reset_skip_write_only(
        &mut self,
        delay: &mut impl DelayUs,
        write: &[u8],
    ) -> Result<(), Error<E>> {
        self.reset(delay)?;
        self.skip(delay)?;
        self.write_bytes(delay, write)?;
        Ok(())
    }

    pub fn skip(&mut self, delay: &mut impl DelayUs) -> Result<(), Error<E>> {
        let parasite_mode = self.parasite_mode;
        self.write_command(delay, Command::SkipRom, parasite_mode)?; // skip
        Ok(())
    }

    pub fn select(&mut self, delay: &mut impl DelayUs, addr: &Address) -> Result<(), Error<E>> {
        let parasite_mode = self.parasite_mode;
        self.write_command(delay, Command::MatchRom, parasite_mode)?; // select
        for i in 0..Address::BYTES {
            let last = i == Address::BYTES - 1;
            self.write_byte(delay, addr[i as usize], parasite_mode && last)?;
        }
        Ok(())
    }

    pub fn search_next(
        &mut self,
        search: &mut DeviceSearch,
        delay: &mut impl DelayUs,
    ) -> Result<Option<Address>, Error<E>> {
        self.search(search, delay, Command::SearchRom)
    }

    pub fn search_next_alarmed(
        &mut self,
        search: &mut DeviceSearch,
        delay: &mut impl DelayUs,
    ) -> Result<Option<Address>, Error<E>> {
        self.search(search, delay, Command::SearchRomAlarmed)
    }

    /// Performs a reset and listens for a presence pulse
    /// Returns Err(WireFault) if the wire seems to be shortened,
    /// Ok(true) if presence pulse has been received and Ok(false)
    /// if no other device was detected but the wire seems to be ok
    pub fn reset(&mut self, delay: &mut impl DelayUs) -> Result<(), Error<E>> {
        // let mut cli = DisableInterrupts::new();
        self.set_high()?;
        // drop(cli);

        self.ensure_wire_high(delay)?;
        // cli = DisableInterrupts::new();
        self.set_low()?;

        // drop(cli);
        delay.delay_us(480);
        // cli = DisableInterrupts::new();
        self.set_high()?;

        let mut presence = false;
        for _ in 0..7 {
            delay.delay_us(10);
            presence |= self.is_low()?;
        }
        // drop(cli);
        delay.delay_us(410);
        if presence {
            Ok(())
        } else {
            Err(Error::NoPresence)
        }
    }

    pub fn reset_presence(&mut self, delay: &mut impl DelayUs) -> Result<bool, Error<E>> {
        self.reset(delay).map(|_| true).or_else(|error| {
            if matches!(error, Error::NoPresence) {
                Ok(false)
            } else {
                Err(error)
            }
        })
    }

    fn ensure_wire_high(&mut self, delay: &mut impl DelayUs) -> Result<(), Error<E>> {
        for _ in 0..125 {
            if self.is_high()? {
                return Ok(());
            }
            delay.delay_us(2);
        }
        Err(Error::WireFault)
    }

    pub fn read_bytes(&mut self, delay: &mut impl DelayUs, dst: &mut [u8]) -> Result<(), E> {
        for d in dst {
            *d = self.read_byte(delay)?;
        }
        Ok(())
    }

    pub(crate) fn read_byte(&mut self, delay: &mut impl DelayUs) -> Result<u8, E> {
        let mut byte = 0_u8;
        for _ in 0..8 {
            byte >>= 1;
            if self.read_bit(delay)? {
                byte |= 0x80;
            }
        }
        Ok(byte)
    }

    pub(crate) fn read_bit(&mut self, delay: &mut impl DelayUs) -> Result<bool, E> {
        // let cli = DisableInterrupts::new();
        self.set_low()?;
        delay.delay_us(3);
        self.set_high()?;
        delay.delay_us(2); // was 10
        let val = self.is_high();
        // drop(cli);
        delay.delay_us(61); // was 53
        val
    }

    pub fn write_command(
        &mut self,
        delay: &mut impl DelayUs,
        cmd: impl OpCode,
        parasite_mode: bool,
    ) -> Result<(), E> {
        self.write_byte(delay, cmd.op_code(), parasite_mode)
    }

    pub fn write_bytes(&mut self, delay: &mut impl DelayUs, bytes: &[u8]) -> Result<(), E> {
        for b in bytes {
            self.write_byte(delay, *b, false)?;
        }
        self.disable_parasite_mode(self.parasite_mode)?;
        Ok(())
    }

    pub(crate) fn write_byte(
        &mut self,
        delay: &mut impl DelayUs,
        byte: u8,
        parasite_mode: bool,
    ) -> Result<(), E> {
        let mut byte = byte;
        for _ in 0..8 {
            self.write_bit(delay, (byte & 0x01) == 0x01)?;
            byte >>= 1;
        }
        self.disable_parasite_mode(parasite_mode)?;
        Ok(())
    }

    pub(crate) fn write_bit(&mut self, delay: &mut impl DelayUs, high: bool) -> Result<(), E> {
        // let cli = DisableInterrupts::new();
        self.set_low()?;
        delay.delay_us(if high { 10 } else { 65 });
        self.set_high()?;
        // drop(cli);
        delay.delay_us(if high { 55 } else { 5 });
        Ok(())
    }

    #[inline(always)]
    pub(crate) fn disable_parasite_mode(&mut self, parasite_mode: bool) -> Result<(), E> {
        if !parasite_mode {
            // let cli = DisableInterrupts::new();
            self.set_low()?;
        }
        Ok(())
    }

    #[inline(always)]
    pub(crate) fn set_high(&mut self) -> Result<(), E> {
        self.io_wire.set_high()
    }

    #[inline(always)]
    pub(crate) fn set_low(&mut self) -> Result<(), E> {
        self.io_wire.set_low()
    }

    #[inline(always)]
    pub(crate) fn is_high(&self) -> Result<bool, E> {
        self.io_wire.is_high()
    }

    #[inline(always)]
    pub(crate) fn is_low(&self) -> Result<bool, E> {
        self.io_wire.is_low()
    }
}
