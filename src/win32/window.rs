//! Win32 Window methods & types.

use crate::{
    geom::Dimension2D,
    win32::{debug::msgs::DebugMsg, errors::*, invoke::chk},
};

use ::std::{ffi::CString, sync::Arc};
use ::windows::{
    core::{s, PCSTR},
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleA,
        UI::WindowsAndMessaging::{
            AdjustWindowRectEx, CreateWindowExA, DefWindowProcA, GetWindowLongPtrA, LoadCursorA,
            PostQuitMessage, RegisterClassExA, SetWindowLongPtrA, ShowWindow, CREATESTRUCTA,
            CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, IDC_ARROW, SW_SHOWNORMAL,
            WINDOW_EX_STYLE, WM_CLOSE, WM_NCCREATE, WM_NCDESTROY, WM_PAINT, WNDCLASSEXA,
            WS_OVERLAPPEDWINDOW,
        },
    },
};

/// A rusty wrapper around Win32 window class.
pub struct Window<P>
where
    P: Fn(),
{
    dimension: Dimension2D<i32>,
    on_paint: P,
}

// TODO: remove HWND param from handle_msg
// TOOD: use separate setup/thunk window proc
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
                lpfnWndProc: Some(Self::wnd_proc_fn),
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

    /// C-function Win32 window procedure which acts as shim and delegates to
    /// `Self::wnd_proc`.
    extern "system" fn wnd_proc_fn(
        hwnd: HWND,
        umsg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        println!("{}", DebugMsg::new(umsg, wparam, lparam));

        match umsg {
            // If we've received a create event, then we populate an `Arc`'ed
            // reference our rust window type in the user data section of the Win32
            // window.
            WM_NCCREATE => {
                let create_struct = lparam.0 as *const CREATESTRUCTA;
                let self_ = unsafe { (*create_struct).lpCreateParams } as *const Self;

                chk!(last_err; SetWindowLongPtrA(hwnd, GWLP_USERDATA, self_ as _)).unwrap();
            }
            // Our window is being destroyed, so we must clean up our Arc'd data.
            WM_NCDESTROY => {
                let self_ = chk!(last_err; SetWindowLongPtrA(hwnd, GWLP_USERDATA, 0)).unwrap()
                    as *const Self;

                // Consume and drop our Arc which was held inside the Win32
                // window.
                let _ = unsafe { Arc::from_raw(self_) };
            }
            // We are neither creating, nor destroying a window, so we must find
            // the `wnd_proc` method on our rust window and pass the message along.
            _ => {
                match chk!(nonzero_isize; GetWindowLongPtrA(hwnd, GWLP_USERDATA)) {
                    // We've received a window event but we haven't yet received
                    // the create window event and populated the user data with
                    // a pointer to our rust window. We cannot handle this message
                    // yet.
                    Err(_) => (),
                    Ok(ptr) => {
                        let self_ = ptr.get() as *const Self;

                        unsafe {
                            // Add extra retain for the duration of following call
                            Arc::increment_strong_count(self_);
                            Arc::from_raw(self_).wnd_proc(hwnd, umsg, wparam, lparam);
                        }
                    }
                }
            }
        }

        // We _always_ pass our message through to the default window procedure.
        unsafe { DefWindowProcA(hwnd, umsg, wparam, lparam) }
    }

    fn wnd_proc(&self, _hwnd: HWND, umsg: u32, _wparam: WPARAM, _lparam: LPARAM) {
        match umsg {
            WM_PAINT => {
                (self.on_paint)();
            }
            WM_CLOSE => {
                unsafe { PostQuitMessage(0) };
            }
            _ => (),
        }
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
