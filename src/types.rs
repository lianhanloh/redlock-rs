///! This mod specifies RedlockResult as an alias for Result and the various Error enum types

use std::result;

/// Redlock Error 
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Error {
    /// Failed to acquire lock
    CannotObtainLock,
    /// Error communicating with 1 or more Redis masters
    RedlockConn,
    /// Failed to connect to enough Redis masters
    NotEnoughMasters,
    /// Lock wasn't valid (may have expired)
    InvalidLock,
    /// Unlock instance failed
    UnlockFailed,
}

pub type RedlockResult<T> = result::Result<T, Error>;
