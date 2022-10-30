//! Fundamental types when interacting with Win32 APIs

use ::std::{
    ffi::{CString, NulError},
    fmt::{self, Display},
};
use ::windows::core::PCSTR;

/// An integer reference for a compiled windows resource (icons, cursors, etc).
pub type ResourceId = isize;

pub use winstr::*;
mod winstr {
    use super::*;

    /// The Win32 API often encodes plain integer values in a PCSTR, so we
    /// allow for this in our inner type.
    #[derive(Clone, Debug)]
    enum Inner {
        String(CString),
        Int(isize),
    }

    /// A convenience type which can be constructed from `&str` or `String` and
    /// which can convert itself into a `PCSTR`.
    #[derive(Clone, Debug)]
    pub struct WinString(Inner);

    impl WinString {
        pub fn from_resource_id(i: ResourceId) -> Self {
            Self(Inner::Int(i))
        }

        pub fn new<S: AsRef<str>>(s: S) -> Result<Self, NulError> {
            Self::try_from(s.as_ref())
        }
    }

    impl Display for WinString {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match &self.0 {
                Inner::String(s) => write!(f, "{}", s.to_string_lossy()),
                Inner::Int(i) => write!(f, "{}_isize", i),
            }
        }
    }

    impl TryFrom<&str> for WinString {
        type Error = NulError;

        fn try_from(s: &str) -> Result<Self, Self::Error> {
            CString::new(s).map(|s| WinString(Inner::String(s)))
        }
    }

    impl From<&WinString> for PCSTR {
        fn from(s: &WinString) -> Self {
            match &s.0 {
                Inner::String(s) => Self::from_raw(s.as_ptr() as *const u8),
                Inner::Int(i) => Self(*i as _),
            }
        }
    }
}
