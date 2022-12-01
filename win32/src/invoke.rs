//! Utilities for invoking native Win32 API in a safe & ergonomic way.

use crate::errors::*;

use ::std::num::{NonZeroIsize, NonZeroU16};
use ::windows::{
    core::{Result as Win32Result, PCSTR, PCWSTR, PSTR, PWSTR},
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
/// - res
///
/// ### Usage
///
/// ```
/// use ::win32::invoke;
/// use ::windows::Win32::System::LibraryLoader::GetModuleHandleA;
///
/// let _module = invoke::chk!(res; GetModuleHandleA(None)).unwrap();
/// ```
#[macro_export]
macro_rules! chk {
    ($check:expr ; $fn:ident ( $( $param:expr),* ) ) => {
        ::paste::paste! {
            $crate::invoke:: [< check_ $check >] (
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
            #[doc = "Invokes a Win32 API which defines success by non-zero return"]
            #[doc = "codes. Returns a guaranteed `NonZero` integer or otherwise maps"]
            #[doc = "the result of `F` to a crate error complete with system error"]
            #[doc = "message context."]
            #[doc = ""]
            #[doc = "Can be used with [crate::chk] by specifying `nonzero_" $num "`"]
            #[doc = "as the type of check, e.g.: `chk!(nonzero_" $num "; ...)`"]
            pub fn [<check_nonzero_ $num>]<F>(f: F, f_name: &'static str) -> Result<$nonzero>
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
///
/// Can be used with [crate::chk] by specifying `last_err` as the type of check,
/// e.g.: `chk!(last_err; ...)`
pub fn check_last_err<F, R>(f: F, f_name: &'static str) -> Result<R>
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
///
/// Can be used with [crate::chk] by specifying `bool` as the type of check,
/// e.g.: `chk!(bool; ...)`
pub fn check_bool<F>(f: F, f_name: &'static str) -> Result<()>
where
    F: FnOnce() -> BOOL,
{
    f().ok().map_err(|_| get_last_err(f_name))
}

/// Invokes a Win32 API which defines success by Win32 results. Maps
/// the result of `F` to an error on failure.
///
/// Can be used with [crate::chk] by specifying `res` as the type of check,
/// e.g.: `chk!(res; ...)`
pub fn check_res<F, V>(f: F, f_name: &'static str) -> Result<V>
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
///
/// Can be used with [crate::chk] by specifying `ptr` as the type of check,
/// e.g.: `chk!(ptr; ...)`
pub fn check_ptr<F, P>(f: F, f_name: &'static str) -> Result<P>
where
    F: FnOnce() -> P,
    P: Win32Pointer,
{
    let ptr = f();

    if ptr.is_null() {
        Err(get_last_err(f_name))
    } else {
        Ok(ptr)
    }
}

/// A common trait implemented for Win32 pointer types.
pub trait Win32Pointer {
    /// Predicate method which indicates whether the pointer should be
    /// considered a null pointer.
    fn is_null(&self) -> bool;
}

macro_rules! impl_win32_ptr {
    ($type:ty; wrapped_is_null) => {
        impl Win32Pointer for $type {
            fn is_null(&self) -> bool {
                self.0.is_null()
            }
        }
    };
    ($type:ty; int_val) => {
        impl Win32Pointer for $type {
            fn is_null(&self) -> bool {
                self.0 == 0 as _
            }
        }
    };
}

impl_win32_ptr!(HWND; int_val);
impl_win32_ptr!(PSTR; wrapped_is_null);
impl_win32_ptr!(PWSTR; wrapped_is_null);
impl_win32_ptr!(PCSTR; wrapped_is_null);
impl_win32_ptr!(PCWSTR; wrapped_is_null);
