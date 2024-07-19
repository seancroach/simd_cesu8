#[cfg(all(feature = "nightly", not(feature = "std")))]
use core::error;
use core::fmt;
#[cfg(feature = "std")]
use std::error;

/// A zero-sized type that represents an error that occurred while decoding.
///
/// No information is provided where the error occurred or what the error was,
/// only that an error *did* occur.
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(clippy::module_name_repetitions)]
pub struct DecodingError(pub(crate) ());

impl fmt::Debug for DecodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DecodingError")
    }
}

impl fmt::Display for DecodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid input")
    }
}

#[cfg(any(feature = "nightly", feature = "std"))]
impl error::Error for DecodingError {}
