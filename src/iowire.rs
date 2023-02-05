use core::fmt::Debug;
use embedded_hal::digital::v2::{InputPin, OutputPin};

pub trait IoWire {
    type Error: Sized + Debug;

    /// Is the input pin high?
    fn is_high(&self) -> Result<bool, Self::Error>;

    /// Is the input pin low?
    fn is_low(&self) -> Result<bool, Self::Error>;

    /// Drives the pin low
    ///
    /// *NOTE* the actual electrical state of the pin may not actually be low, e.g. due to external
    /// electrical sources
    fn set_low(&mut self) -> Result<(), Self::Error>;

    /// Drives the pin high
    ///
    /// *NOTE* the actual electrical state of the pin may not actually be high, e.g. due to external
    /// electrical sources
    fn set_high(&mut self) -> Result<(), Self::Error>;
}

/// Single line config wrapper
impl<E, IO> IoWire for (IO,)
where
    E: Debug,
    IO: OutputPin<Error = E> + InputPin<Error = E>,
{
    type Error = E;

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.0.is_high()
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        self.0.is_low()
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.0.set_low()
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.0.set_high()
    }
}

/// Dual line config wrapper
impl<E, I, O> IoWire for (I, O)
where
    E: Debug,
    I: InputPin<Error = E>,
    O: OutputPin<Error = E>,
{
    type Error = E;

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.0.is_high()
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        self.0.is_low()
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.1.set_low()
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.1.set_high()
    }
}

/// Inverted wire wrapper
pub struct Inverted<P>(pub P);

impl<I> InputPin for Inverted<I>
where
    I: InputPin,
{
    type Error = I::Error;

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.0.is_low()
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        self.0.is_high()
    }
}

impl<O> OutputPin for Inverted<O>
where
    O: OutputPin,
{
    type Error = O::Error;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.0.set_high()
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.0.set_low()
    }
}
