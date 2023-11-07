use embedded_hal::digital::{Error, ErrorType, InputPin, OutputPin};

pub trait IoWire {
    type Error: Error;

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
impl<IO> IoWire for (IO,)
where
    IO: ErrorType + OutputPin + InputPin,
{
    type Error = IO::Error;

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
    E: Error,
    I: ErrorType<Error = E> + InputPin,
    O: ErrorType<Error = E> + OutputPin,
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

impl<I: ErrorType> ErrorType for Inverted<I> {
    type Error = I::Error;
}

impl<I> InputPin for Inverted<I>
where
    I: InputPin,
{
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
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.0.set_high()
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.0.set_low()
    }
}
