//! Crate-specific error and result types, plus common conversions.

use ::std::fmt::{self, Display};
use ::windows::{
    core::{Error as Win32Error, HRESULT},
    Win32::Foundation::GetLastError,
};

/// Result type returned by functions that call into Win32 API.
pub type Result<T> = ::std::result::Result<T, Error>;

/// Error type for functions that call into Win32 API. The error attempts to
/// pro-actively capture as much context as possible (error codes, system error
/// message strings, etc).
#[derive(::thiserror::Error, Debug)]
pub enum Error {
    #[error("unexpected win32 error in {function}. {context}")]
    Unexpected {
        /// The name of the function which failed. Typically provided to [chk].
        function: &'static str,
        /// Inner context which can be formatted with `Display`
        context: Context,
    },
}

/// Inner error context. Implements `Display` to conveniently print any Win32
/// error codes or system error messages which were gathered at the point of the
/// error.
#[derive(Debug)]
pub enum Context {
    Win32Error(Win32Error),
    Hresult(HRESULT),
}

impl From<Win32Error> for Context {
    fn from(e: Win32Error) -> Self {
        Self::Win32Error(e)
    }
}

impl From<HRESULT> for Context {
    fn from(hres: HRESULT) -> Self {
        Self::Hresult(hres)
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Context::*;
        match self {
            Win32Error(e) => write!(f, "{e}"),
            Hresult(hres) => write!(f, "{code}: {msg}", code = hres.0, msg = hres.message()),
        }
    }
}

/// Gets the last Win32 error (the Win32 equivalent of `errno`).
pub(crate) fn get_last_err(f_name: &'static str) -> Error {
    let hresult = unsafe { GetLastError() }.to_hresult();
    Error::Unexpected {
        function: f_name,
        context: hresult.into(),
    }
}
