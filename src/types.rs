use std::result;

/// Redlock Error 
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Error {
    /// Failed to acquire lock
    CannotObtainLock,
    /// Error communicating with 1 or more Redis masters
    MultipleRedlock,
}

pub type RedlockResult<T> = result::Result<T, Error>;
