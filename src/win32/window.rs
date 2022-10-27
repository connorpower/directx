//! Win32 Window methods & types.

use crate::{
    geom::Dimension2D,
    win32::{debug::msgs::DebugMsg, errors::*, invoke::chk},
};

use ::std::{cell::Cell, ffi::CString, sync::Arc};
use ::windows::{
    core::{s, PCSTR},
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleA,
        UI::WindowsAndMessaging::{
            AdjustWindowRectEx, CreateWindowExA, DefWindowProcA, GetWindowLongPtrA, LoadCursorA,
            PostQuitMessage, RegisterClassExA, SetWindowLongPtrA, ShowWindow, CREATESTRUCTA,
            CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, GWLP_WNDPROC, IDC_ARROW,
            SW_SHOWNORMAL, WINDOW_EX_STYLE, WM_CLOSE, WM_NCCREATE, WM_NCDESTROY, WM_PAINT,
            WNDCLASSEXA, WS_OVERLAPPEDWINDOW,
        },
    },
};

/// A rusty wrapper around Win32 window class.
pub struct Window<P>
where
    P: Fn(),
{
    hwnd: Cell<HWND>,
    dimension: Dimension2D<i32>,
    on_paint: P,
}

// TODO; unregister class
// TODO: Arc'ed repo of window classes?
// TOOD: test multiple instances of same window class
// TODO: test multiple windows of different classes
impl<P> Window<P>
where
    P: Fn(),
{
    const fn class_name() -> PCSTR {
        s!("MainWindow")
    }

    /// Construct and display a new window.
    pub fn new(dimension: Dimension2D<i32>, name: &str, on_paint: P) -> Result<Arc<Self>> {
        let this = Arc::new(Window {
            hwnd: Cell::new(HWND::default()),
            dimension,
            on_paint,
        });

        let wnd_class = {
            let cursor = chk!(res;
                LoadCursorA(
                    HINSTANCE::default(),
                    PCSTR::from_raw(IDC_ARROW.as_ptr() as *const u8)
                )
            )?;
            WNDCLASSEXA {
                cbSize: ::std::mem::size_of::<WNDCLASSEXA>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::wnd_proc_setup),
                lpszClassName: Self::class_name(),
                hCursor: cursor,
                ..Default::default()
            }
        };

        let hwnd = {
            let module = chk!(res; GetModuleHandleA(PCSTR::null()))?;
            let _atom = chk!(nonzero_u16; RegisterClassExA(&wnd_class))?;
            let mut rect = dimension.into();
            chk!(bool; AdjustWindowRectEx(
                &mut rect,
                WS_OVERLAPPEDWINDOW,
                false,
                WINDOW_EX_STYLE::default()
            ))?;
            let name = CString::new(name).expect("Window name contained null byte");
            chk!(ptr; CreateWindowExA(
                    WINDOW_EX_STYLE::default(),
                    Self::class_name(),
                    PCSTR::from_raw(name.as_ptr() as *const u8),
                    WS_OVERLAPPEDWINDOW,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    rect.right - rect.left,
                    rect.bottom - rect.top,
                    None,
                    None,
                    module,
                    Some(Arc::into_raw(this.clone()) as *const _)
                )
            )?
        };

        unsafe { ShowWindow(hwnd, SW_SHOWNORMAL) };

        Ok(this)
    }

    /// The dimensions of the client area of our Win32 window. The window chrome
    /// is in addition to this dimension.
    pub const fn dimension(&self) -> Dimension2D<i32> {
        self.dimension
    }

    /// Get a handle to the Win32 window's handle. This is often required when
    /// interacting with other APIs.
    pub fn hwnd(&self) -> HWND {
        self.hwnd.get()
    }

    fn handle_message(&self, umsg: u32, wparam: WPARAM, lparam: LPARAM) {
        println!("{}", DebugMsg::new(umsg, wparam, lparam));

        match umsg {
            WM_PAINT => {
                (self.on_paint)();
            }
            WM_CLOSE => {
                unsafe { PostQuitMessage(0) };
            }
            WM_NCDESTROY => {
                // Our window is being destroyed, so we must clean up our Arc'd
                // handle on the Win32 side.
                let self_ = chk!(last_err; SetWindowLongPtrA(self.hwnd(), GWLP_USERDATA, 0))
                    .unwrap() as *const Self;
                let _ = unsafe { Arc::from_raw(self_) };
            }
            _ => (),
        }
    }

    /// C-function Win32 window procedure performs one-time setup of the
    /// structures on the Win32 side to associate our Rust object with the Win32
    /// object.
    extern "system" fn wnd_proc_setup(
        hwnd: HWND,
        umsg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        // If we've received a create event, then we populate an `Arc`'ed
        // reference our rust window type in the user data section of the Win32
        // window.
        if umsg == WM_NCCREATE {
            let create_struct = lparam.0 as *const CREATESTRUCTA;
            // SAFETY: The `CREATESRUCTA` structure is guaranteed by the Win32
            // API to be valid if we've received an event of type `WM_NCCREATE`.
            let self_ = unsafe { (*create_struct).lpCreateParams } as *const Self;

            // SAFETY: `self` is within an Arc which we don't release until the
            // window is destroyed. We are still creating the window here and
            // our message loop is single threaded so no other window activity
            // could be happening.
            unsafe { (*self_).hwnd.set(hwnd) };

            chk!(last_err; SetWindowLongPtrA(hwnd, GWLP_USERDATA, self_ as _)).unwrap();
            chk!(last_err; SetWindowLongPtrA(hwnd, GWLP_WNDPROC, Self::wnd_proc_thunk as _))
                .unwrap();
        }

        // We _always_ pass our message through to the default window procedure.
        unsafe { DefWindowProcA(hwnd, umsg, wparam, lparam) }
    }

    extern "system" fn wnd_proc_thunk(
        hwnd: HWND,
        umsg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if let Ok(ptr) = chk!(nonzero_isize; GetWindowLongPtrA(hwnd, GWLP_USERDATA)) {
            let self_ = ptr.get() as *const Self;

            unsafe {
                // Add extra retain for the duration of following call
                Arc::increment_strong_count(self_);
                Arc::from_raw(self_).handle_message(umsg, wparam, lparam);
            }
        }

        // We _always_ pass our message through to the default window procedure.
        unsafe { DefWindowProcA(hwnd, umsg, wparam, lparam) }
    }
}

impl<P> Drop for Window<P>
where
    P: Fn(),
{
    fn drop(&mut self) {
        println!("window dropped!")
    }
}
