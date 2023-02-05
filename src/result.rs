use core::fmt::Debug;

/// Error type
#[derive(Debug)]
pub enum Error<E: Sized + Debug> {
    NotSupport,
    /// Wire not high
    WireFault,
    /// No presence on wire
    NoPresence,
    CrcMismatch(u8, u8),
    FamilyCodeMismatch(u8, u8),
    //Debug(Option<u8>),
    PortError(E),
}

impl<E: Sized + Debug> From<E> for Error<E> {
    fn from(e: E) -> Self {
        Error::PortError(e)
    }
}
