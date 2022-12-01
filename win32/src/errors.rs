//! Crate-specific error and result types, plus common conversions.

use ::windows::core::Error as Win32Error;

/// Result type returned by functions that call into Win32 API.
pub type Result<T> = ::std::result::Result<T, Error>;

/// Error type for functions that call into Win32 API. The error attempts to
/// pro-actively capture as much context as possible (error codes, system error
/// message strings, etc).
#[derive(::thiserror::Error, Debug)]
pub enum Error {
    /// An unexpected error occurred and was not handled internally.
    #[error("unexpected win32 error in {function}. {context}")]
    Unexpected {
        /// The name of the function which failed. Typically provided to
        /// [`crate::chk`].
        function: &'static str,
        /// Inner error context. Implements [`Display`](std::fmt::Display) to
        /// conveniently print any Win32 error codes or system error messages
        /// which were gathered at the point of the error.
        context: Win32Error,
    },
}

/// Gets the last Win32 error (the Win32 equivalent of `errno`).
pub(crate) fn get_last_err(f_name: &'static str) -> Error {
    Error::Unexpected {
        function: f_name,
        context: Win32Error::from_win32(),
    }
}
