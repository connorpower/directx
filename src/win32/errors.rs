use ::std::fmt::{self, Display};
use ::windows::{
    core::{Error as Win32Error, HRESULT},
    Win32::Foundation::GetLastError,
};

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(::thiserror::Error, Debug)]
pub enum Error {
    #[error("unexpected win32 error in {function}. {context}")]
    Unexpected {
        function: &'static str,
        context: Context,
    },
}

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

pub(crate) fn get_last_err(f_name: &'static str) -> Error {
    let hresult = unsafe { GetLastError() }.to_hresult();
    Error::Unexpected {
        function: f_name,
        context: hresult.into(),
    }
}
