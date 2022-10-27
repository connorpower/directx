//! Utilities for invoking native Win32 API in a safe & ergonomic way.

use super::errors::*;

use ::std::num::{NonZeroIsize, NonZeroU16};

use ::windows::{
    core::Result as Win32Result,
    Win32::Foundation::{GetLastError, SetLastError, BOOL, HWND, WIN32_ERROR},
};

/// Invokes a Win32 function with the provided argument and checks the return
/// value for success, or creates a crate error with context.
///
/// The supported values for check are:
/// - nonzero_isize
/// - nonzero_u16
/// - last_err
/// - hwnd
/// - bool
///
/// ### Usage
///
/// ```
/// invoke::chk!(nonzero_isize; GetWindowLongPtrA(hwnd, GWLP_USERDATA))?;
/// ```
#[macro_export]
macro_rules! chk {
    ($check:expr ; $fn:ident ( $( $param:expr),* ) ) => {
        ::paste::paste! {
            $crate::win32::invoke:: [< check_ $check >] (
                || unsafe { [<$fn>]( $( $param, )* ) } ,
                ::std::stringify!([<$fn>])
            )
        }
    }
}

pub use chk;

macro_rules! impl_nonzero {
    ($num:ty => $nonzero:ty) => {
        ::paste::paste! {
            /// Invokes a Win32 API which defines success by non-zero return
            /// codes. Returns a guaranteed `NonZero` integer or otherwise maps
            /// the result of `F` to a crate error complete with system error
            /// message context.
            pub(crate) fn [<check_nonzero_ $num>]<F>(f: F, f_name: &'static str) -> Result<$nonzero>
            where
                F: FnOnce() -> $num,
            {
                <$nonzero>::new(f()).ok_or_else(|| get_last_err(f_name))
            }
        }
    };
}

impl_nonzero!(u16 => NonZeroU16);
impl_nonzero!(isize => NonZeroIsize);

/// Invokes a Win32 API which indicates failure by setting the last error code
/// and not by return type or output params. The last error is cleared
/// immediately before invoking the function.
pub(crate) fn check_last_err<F, R>(f: F, f_name: &'static str) -> Result<R>
where
    F: FnOnce() -> R,
{
    unsafe { SetLastError(WIN32_ERROR(0)) };
    let res = f();
    let last_err = unsafe { GetLastError() };

    if last_err.is_ok() {
        Ok(res)
    } else {
        Err(Error::Unexpected {
            function: f_name,
            context: last_err.to_hresult().into(),
        })
    }
}

/// Invokes a Win32 API which defines success by bool return values. Maps the
/// result of `F` to an error on failure.
pub(crate) fn check_bool<F>(f: F, f_name: &'static str) -> Result<()>
where
    F: FnOnce() -> BOOL,
{
    f().ok().map_err(|_| get_last_err(f_name))
}

pub(crate) trait Win32Pointer {
    fn is_invalid(&self) -> bool;
}

impl Win32Pointer for HWND {
    fn is_invalid(&self) -> bool {
        self.0 == 0
    }
}

/// Invokes a Win32 API which defines success by Win32 results. Maps
/// the result of `F` to an error on failure.
pub(crate) fn check_res<F, V>(f: F, f_name: &'static str) -> Result<V>
where
    F: FnOnce() -> Win32Result<V>,
{
    f().map_err(|e| Error::Unexpected {
        function: f_name,
        context: e.into(),
    })
}

/// Invokes a Win32 API which defines success by non-zero pointers. Maps
/// the result of `F` to an error on failure.
pub(crate) fn check_ptr<F, P>(f: F, f_name: &'static str) -> Result<P>
where
    F: FnOnce() -> P,
    P: Win32Pointer,
{
    let ptr = f();

    if ptr.is_invalid() {
        Err(get_last_err(f_name))
    } else {
        Ok(ptr)
    }
}
