use crate::{ds1990::Ds1990, Address, Device, Driver, Error, IoWire, OpCode};
use core::fmt::Debug;
use embedded_hal::delay::DelayUs;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Ds1990Type {
    Ds1990,
    Rw1990p1,
    Rw1990p2,
    Tm01,
    Tm2004,
    Cyfral,
    Metacom,
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum CommandTm2004 {
    ReadStatus = 0xAA,
    WriteRom = 0x3C,
    BlockRom = 0x35,
}

impl OpCode for CommandTm2004 {
    fn op_code(&self) -> u8 {
        *self as _
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum CommandRw1990 {
    WriteLockSet1 = 0xD1,
    WriteLockGet1 = 0xB1,
    WriteLockSet2 = 0x1D,
    WriteLockGet2 = 0x1E,
    WriteLockSet3 = 0xC1,
    WriteRom = 0xD5,
}

impl OpCode for CommandRw1990 {
    fn op_code(&self) -> u8 {
        *self as _
    }
}

impl Ds1990 {
    fn set_write_lock<W: IoWire>(
        driver: &mut Driver<W>,
        delay: &mut impl DelayUs,
        type_: Ds1990Type,
        lock: bool,
    ) -> Result<bool, Error<W::Error>> {
        let (set_cmd, get_cmd, lock) = match type_ {
            Ds1990Type::Rw1990p1 => (
                CommandRw1990::WriteLockSet1.op_code(),
                CommandRw1990::WriteLockGet1.op_code(),
                !lock,
            ),
            Ds1990Type::Rw1990p2 => (
                CommandRw1990::WriteLockSet2.op_code(),
                CommandRw1990::WriteLockGet2.op_code(),
                lock,
            ),
            Ds1990Type::Tm01 => (
                CommandRw1990::WriteLockSet3.op_code(),
                CommandRw1990::WriteLockGet1.op_code(),
                lock,
            ),
            _ => return Err(Error::NotSupport),
        };

        driver.reset_write_only(delay, &[set_cmd])?;
        driver.write_bit(delay, lock)?;
        delay.delay_us(10000);

        let mut state = [0u8];
        driver.reset_write_read(delay, &[get_cmd], &mut state)?;

        Ok(state[0] == 0xFE)
    }

    pub fn detect_type<W: IoWire>(
        driver: &mut Driver<W>,
        delay: &mut impl DelayUs,
    ) -> Result<Ds1990Type, Error<W::Error>> {
        Ok(
            if Self::set_write_lock(driver, delay, Ds1990Type::Rw1990p1, true)? {
                Self::set_write_lock(driver, delay, Ds1990Type::Rw1990p1, false)?;
                Ds1990Type::Rw1990p1
            } else if Self::set_write_lock(driver, delay, Ds1990Type::Rw1990p2, true)? {
                Self::set_write_lock(driver, delay, Ds1990Type::Rw1990p2, false)?;
                Ds1990Type::Rw1990p2
            } else {
                let data_wr = [CommandTm2004::ReadStatus.op_code(), 0x00, 0x00];
                let mut data_rd = [0u8; 2];
                driver.reset_write_read(delay, &data_wr, &mut data_rd)?;
                driver.reset(delay)?;

                if crate::compute_partial_crc8(0, &data_wr) == data_rd[0] {
                    Ds1990Type::Tm2004
                } else {
                    Ds1990Type::Tm01
                }
            },
        )
    }

    pub fn write_address<W: IoWire>(
        &self,
        driver: &mut Driver<W>,
        delay: &mut impl DelayUs,
        type_: Ds1990Type,
    ) -> Result<(), Error<W::Error>> {
        match type_ {
            Ds1990Type::Rw1990p1 | Ds1990Type::Rw1990p2 | Ds1990Type::Tm01 => {
                self.write_address_rw1990(driver, delay, type_)
            }
            Ds1990Type::Tm2004 => self.write_address_tm2004(driver, delay),
            _ => Err(Error::NotSupport),
        }
    }

    pub fn write_address_rw1990<W: IoWire>(
        &self,
        driver: &mut Driver<W>,
        delay: &mut impl DelayUs,
        type_: Ds1990Type,
    ) -> Result<(), Error<W::Error>> {
        Self::set_write_lock(driver, delay, type_, false)?;

        driver.reset_write_only(delay, &[CommandRw1990::WriteRom.op_code()])?;
        driver.write_bytes_rw(
            delay,
            self.address().as_ref(),
            !matches!(type_, Ds1990Type::Rw1990p2),
        )?;

        Self::set_write_lock(driver, delay, type_, true)?;

        Ok(())
    }

    pub fn write_address_tm2004<W: IoWire>(
        &self,
        driver: &mut Driver<W>,
        delay: &mut impl DelayUs,
    ) -> Result<(), Error<W::Error>> {
        let mut crc_read = [0u8; 1];

        for i in 0..Address::BYTES {
            let cmd_write = [
                CommandTm2004::WriteRom.op_code(),
                i,
                0,
                self.address()[i as usize],
            ];

            driver.reset_write_read(delay, &cmd_write, &mut crc_read)?;

            let crc_write = crate::compute_partial_crc8(0, &cmd_write);

            if crc_write != crc_read[0] {
                return Err(Error::CrcMismatch(crc_write, crc_read[0]));
            }
        }

        driver.program_pulse(delay)?;

        Ok(())
    }
}

impl<E: Debug, W: IoWire<Error = E>> Driver<W> {
    pub fn write_bytes_rw(
        &mut self,
        delay: &mut impl DelayUs,
        bytes: &[u8],
        invert: bool,
    ) -> Result<(), E> {
        for b in bytes {
            self.write_byte_rw(delay, *b, invert)?;
        }
        Ok(())
    }

    pub(crate) fn write_byte_rw(
        &mut self,
        delay: &mut impl DelayUs,
        byte: u8,
        invert: bool,
    ) -> Result<(), E> {
        let mut byte = byte;
        for _ in 0..8 {
            let bit = (byte & 0x01) == 0x01;
            self.write_bit_rw(delay, if invert { !bit } else { bit })?;
            byte >>= 1;
        }
        Ok(())
    }

    pub(crate) fn write_bit_rw(&mut self, delay: &mut impl DelayUs, high: bool) -> Result<(), E> {
        // let cli = DisableInterrupts::new();
        self.set_low()?;
        delay.delay_us(if high { 6 } else { 60 });
        self.set_high()?;
        // drop(cli);
        delay.delay_us(10000);
        Ok(())
    }

    pub(crate) fn program_pulse(&mut self, delay: &mut impl DelayUs) -> Result<(), E> {
        // let cli = DisableInterrupts::new();
        self.set_high()?;
        delay.delay_us(600);
        self.set_low()?;
        delay.delay_us(6);
        self.set_high()?;
        // drop(cli);
        delay.delay_us(50000);
        Ok(())
    }
}
