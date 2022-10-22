//! Utilities for invoking native Win32 API in a safe & ergonomic way.

use super::errors::*;

use ::paste::paste;
use ::std::num::{NonZeroIsize, NonZeroU16};
use ::windows::Win32::Foundation::{GetLastError, SetLastError, BOOL, HWND, WIN32_ERROR};

pub(crate) fn last_err(f_name: &'static str) -> Error {
    let hresult = unsafe { GetLastError() }.to_hresult();
    Error::Unexpected {
        function: f_name,
        context: hresult.into(),
    }
}

/// Returns an error only if `GetLastError` returns a non-zero value.
pub(crate) fn maybe_last_err(f_name: &'static str) -> Result<()> {
    let last_err = unsafe { GetLastError() };

    if last_err.is_ok() {
        Ok(())
    } else {
        Err(Error::Unexpected {
            function: f_name,
            context: last_err.to_hresult().into(),
        })
    }
}

macro_rules! impl_nonzero {
    ($num:ty => $nonzero:ty) => {
        paste! {
            /// Invokes a Win32 API which defines success by non-zero return
            /// codes. Returns a guaranteed `NonZero` integer or otherwise maps
            /// the result of `F` to a crate error complete with system error
            /// message context.
            pub(crate) fn [<win32_invoke_ $num>]<F>(f: F, f_name: &'static str) -> Result<$nonzero>
            where
                F: FnOnce() -> $num,
            {
                <$nonzero>::new(f()).ok_or_else(|| last_err(f_name))
            }
        }
    };
}

impl_nonzero!(u16 => NonZeroU16);
impl_nonzero!(isize => NonZeroIsize);

/// Invokes a Win32 API which indicates failure by setting the last error code
/// and not by return type or output params. The last error is cleared
/// immediately before invoking the function.
pub(crate) fn win32_invoke_and_check_err<F, R>(f: F, f_name: &'static str) -> Result<R>
where
    F: FnOnce() -> R,
{
    unsafe { SetLastError(WIN32_ERROR(0)) };
    let res = f();
    maybe_last_err(f_name).map(|_| res)
}

/// Invokes a Win32 API which defines success by bool return values. Maps the
/// result of `F` to a crate error complete with system error message context in
/// the event of failure.
pub(crate) fn win32_invoke_bool<F>(f: F, f_name: &'static str) -> Result<()>
where
    F: FnOnce() -> BOOL,
{
    f().ok().map_err(|_| last_err(f_name))
}

// TODO: this is sad. Ideally we could use HWND(NonZeroISize) or similar
pub(crate) fn win32_invoke_hwnd<F>(f: F, f_name: &'static str) -> Result<HWND>
where
    F: FnOnce() -> HWND,
{
    let hwnd = f();

    if hwnd.0 == 0 {
        Err(last_err(f_name))
    } else {
        Ok(hwnd)
    }
}
