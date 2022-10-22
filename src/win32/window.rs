//! Win32 Window methods & types

use super::{errors::*, invoke};
use ::std::sync::Arc;
use ::windows::{
    core::{s, PCSTR},
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
        System::LibraryLoader::GetModuleHandleA,
        UI::WindowsAndMessaging::{
            AdjustWindowRectEx, CreateWindowExA, DefWindowProcA, GetWindowLongPtrA,
            PostQuitMessage, RegisterClassExA, SetWindowLongPtrA, ShowWindow, CREATESTRUCTA,
            CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, SW_SHOWNORMAL, WINDOW_EX_STYLE,
            WM_CLOSE, WM_NCCREATE, WM_NCDESTROY, WM_PAINT, WNDCLASSEXA, WS_OVERLAPPEDWINDOW,
        },
    },
};

pub trait Window<P>
where
    Self: Sized,
    P: Fn(),
{
    fn new(on_paint: P) -> Result<Arc<Self>>;
}

pub struct MainWindow<P>
where
    P: Fn(),
{
    on_paint: P,
}

impl<P> MainWindow<P>
where
    P: Fn(),
{
    const fn class_name() -> PCSTR {
        s!("MainWindow")
    }

    /// C-function Win32 window procedure which acts as shim and delegates to
    /// `Self::wnd_proc`.
    extern "system" fn wnd_proc_fn(
        hwnd: HWND,
        umsg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match umsg {
            // If we've received a create event, then we populate an `Arc`'ed
            // reference our rust window type in the user data section of the Win32
            // window.
            WM_NCCREATE => {
                let create_struct = lparam.0 as *const CREATESTRUCTA;
                let self_ = unsafe { (*create_struct).lpCreateParams } as *const Self;

                invoke::chk!(last_err; SetWindowLongPtrA(hwnd, GWLP_USERDATA, self_ as _)).unwrap();
            }
            // Our window is being destroyed, so we must clean up our Arc'd data.
            WM_NCDESTROY => {
                let self_ = invoke::chk!(last_err; SetWindowLongPtrA(hwnd, GWLP_USERDATA, 0))
                    .unwrap() as *const Self;

                // Consume and drop our Arc which was held inside the Win32
                // window.
                let _ = unsafe { Arc::from_raw(self_) };
            }
            // We are neither creating, nor destroying a window, so we must find
            // the `wnd_proc` method on our rust window and pass the message along.
            _ => {
                match invoke::chk!(nonzero_isize; GetWindowLongPtrA(hwnd, GWLP_USERDATA)) {
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

impl<P> Window<P> for MainWindow<P>
where
    P: Fn(),
{
    fn new(p: P) -> Result<Arc<Self>> {
        let module = unsafe { GetModuleHandleA(PCSTR::null()) }.map_err(|e| Error::Unexpected {
            function: "GetModuleHandleA",
            context: e.into(),
        })?;

        let wnd_class = WNDCLASSEXA {
            cbSize: ::std::mem::size_of::<WNDCLASSEXA>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(Self::wnd_proc_fn),
            lpszClassName: Self::class_name(),
            ..Default::default()
        };

        // TODO: macro_rules to perform all the following, including
        // string-ifying the function name
        let _atom = invoke::chk!(nonzero_u16; RegisterClassExA(&wnd_class))?;

        let mut rect = RECT {
            left: 0,
            right: 800,
            top: 0,
            bottom: 600,
        };

        invoke::chk!(bool; AdjustWindowRectEx(
            &mut rect,
            WS_OVERLAPPEDWINDOW,
            false,
            WINDOW_EX_STYLE::default()
        ))?;

        let wnd = Arc::new(MainWindow { on_paint: p });

        let hwnd = invoke::chk!(hwnd; CreateWindowExA(
                WINDOW_EX_STYLE::default(),
                Self::class_name(),
                s!("Hello, DirectX!"),
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                rect.right - rect.left,
                rect.bottom - rect.top,
                None,
                None,
                module,
                Some(Arc::into_raw(wnd.clone()) as *const _)
            )
        )?;

        unsafe { ShowWindow(hwnd, SW_SHOWNORMAL) };

        Ok(wnd)
    }
}

impl<P> Drop for MainWindow<P>
where
    P: Fn(),
{
    fn drop(&mut self) {
        println!("window dropped!")
    }
}
