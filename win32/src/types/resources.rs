//! Type definitions for Win32 resources (icons, cursors, etc.)

use ::std::{
    convert::From,
    fmt::{self, Display},
};
use ::windows::core::{PCSTR, PCWSTR};

/// An reference for a compiled windows resource (icons, cursors, etc).
#[derive(Clone, Copy, Debug, Default)]
pub struct ResourceId(isize);

impl ResourceId {
    /// Strongly typed conversion into a `PCSTR` type. Useful for Win32 APIs
    /// that take a resource identifier as a `T: Into<PCSTR>` and therefore
    /// suffer from type ambiguity when using `id.into()`.
    pub fn into_pcstr(self) -> PCSTR {
        self.into()
    }

    /// Strongly typed conversion into a `PCWSTR` type. Useful for Win32 APIs
    /// that take a resource identifier as a `T: Into<PCWSTR>` and therefore
    /// suffer from type ambiguity when using `id.into()`.
    pub fn into_pcwstr(self) -> PCWSTR {
        self.into()
    }
}

impl Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<isize> for ResourceId {
    fn from(val: isize) -> Self {
        Self(val)
    }
}

impl From<ResourceId> for PCSTR {
    fn from(resource: ResourceId) -> Self {
        Self(resource.0 as _)
    }
}

impl From<ResourceId> for PCWSTR {
    fn from(resource: ResourceId) -> Self {
        Self(resource.0 as _)
    }
}
