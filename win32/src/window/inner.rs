use crate::{errors::*, geom::Dimension2D, invoke::chk, types::*, window::WindowClass};

use ::std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
};
use ::tokio::sync::watch;
use ::tracing::{debug, trace};
use ::windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    System::LibraryLoader::GetModuleHandleA,
    UI::WindowsAndMessaging::{
        AdjustWindowRectEx, CreateWindowExA, DefWindowProcA, DestroyWindow, GetWindowLongPtrA,
        SetWindowLongPtrA, ShowWindow, CREATESTRUCTA, CW_USEDEFAULT, GWLP_USERDATA, GWLP_WNDPROC,
        SW_SHOWNORMAL, WINDOW_EX_STYLE, WM_CLOSE, WM_NCCREATE, WM_NCDESTROY, WS_OVERLAPPEDWINDOW,
    },
};

/// The next step to take when handling a window proc message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Forwarding {
    /// The message should be forwarded to the system's default implementation.
    ForwardToSystem,
    /// The message has been completely handled. Do not forward the message.
    None,
}

pub(super) struct WindowInner {
    /// A reference-counted handle to the Win32 window class registered for
    /// windows of this type. When the last `Window` instance is released, the
    /// corresponding Win32 window class will be de-registered.
    window_class: Arc<WindowClass>,
    /// A handle to our corresponding Win32 window. If zero, the window has been
    /// destroyed on the Win32 size.
    hwnd: Cell<isize>,
    /// Fixed dimensions for our window.
    dimension: Dimension2D<i32>,
    /// The Window's title, as it appears in the Windows title bar.
    title: String,

    close_sender: watch::Sender<()>,
    close_receiver: RefCell<watch::Receiver<()>>,
}

impl WindowInner {
    /// Construct and display a new window.
    pub(super) fn new(
        dimension: Dimension2D<i32>,
        title: &str,
        icon_id: Option<ResourceId>,
    ) -> Result<Rc<Self>> {
        debug!(wnd_title = %title, "Creating window inner");

        let (close_sender, close_receiver) = watch::channel(());
        let this = Rc::new(Self {
            title: title.to_string(),
            window_class: WindowClass::get_or_create("MainWindow", icon_id, Self::wnd_proc_setup)?,
            hwnd: Default::default(),
            dimension,
            close_sender,
            close_receiver: RefCell::new(close_receiver),
        });

        let hwnd = {
            let module = chk!(res; GetModuleHandleA(None))?;
            let mut rect = dimension.into();
            chk!(bool; AdjustWindowRectEx(
                &mut rect,
                WS_OVERLAPPEDWINDOW,
                false,
                WINDOW_EX_STYLE::default()
            ))?;
            let title = WinString::new(title).expect("Window name contained null byte");

            chk!(ptr; CreateWindowExA(
                    WINDOW_EX_STYLE::default(),
                    this.window_class.class_name(),
                    &title,
                    WS_OVERLAPPEDWINDOW,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    rect.right - rect.left,
                    rect.bottom - rect.top,
                    None,
                    None,
                    module,
                    Some(Rc::into_raw(this.clone()) as *const _)
                )
            )?
        };
        unsafe { ShowWindow(hwnd, SW_SHOWNORMAL) };

        // Note: We don't store `hwnd` in `this` here. Instead we store the
        // handle when if first appears in the window proc function.

        Ok(this)
    }

    /// The dimensions of the client area of our Win32 window. The window chrome
    /// is in addition to this dimension.
    pub(super) const fn dimension(&self) -> Dimension2D<i32> {
        self.dimension
    }

    pub(super) fn title(&self) -> &str {
        &self.title
    }

    /// Get a handle to the Win32 window's handle. This is often required when
    /// interacting with other APIs.
    ///
    /// If `None`, then the window has already been destroyed on the Win32 side.
    pub(super) fn hwnd(&self) -> Option<HWND> {
        let val = self.hwnd.get();
        if val == 0 { None } else { Some(HWND(val)) }
    }

    /// Returns whether the window has requested to close, and immediately
    /// clears this request. Window is not actually closed until it is
    /// dropped, so the close request can be ignored if needed.
    ///
    /// Safety: this can only be called by the `Window` class.
    pub(super) fn requested_close(&self) -> bool {
        let mut rec = self.close_receiver.borrow_mut();

        if rec.has_changed().unwrap_or(true) {
            let _ = rec.borrow_and_update();
            true
        } else {
            false
        }
    }

    pub(super) fn destroy(&self) -> Result<()> {
        if let Some(h) = self.hwnd() {
            chk!(bool; DestroyWindow(h))?;
        }
        Ok(())
    }

    // TODO: forward to the window. This shouldn't be implemented only on the
    // inner type.
    fn handle_message(&self, umsg: u32, _wparam: WPARAM, _lparam: LPARAM) -> Forwarding {
        trace!(msg = %crate::debug::msgs::DebugMsg::new(umsg, _wparam, _lparam));

        match umsg {
            WM_CLOSE => {
                self.close_sender.send_replace(());
                return Forwarding::None;
            }
            WM_NCDESTROY => {
                debug!(wnd_title = %self.title, "Destroying window inner");

                // Our window is being destroyed, so we must clean up our Rc'd
                // handle on the Win32 side.
                let self_ = chk!(last_err; SetWindowLongPtrA(self.hwnd(), GWLP_USERDATA, 0))
                    .unwrap() as *const Self;
                let _ = unsafe { Rc::from_raw(self_) };

                // Clear our window handle now that we're destroyed.
                self.hwnd.set(0);
            }
            _ => (),
        }

        Forwarding::ForwardToSystem
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
        // If we've received a create event, then we populate an `Rc`'ed
        // reference our rust window type in the user data section of the Win32
        // window.
        if umsg == WM_NCCREATE {
            let create_struct = lparam.0 as *const CREATESTRUCTA;
            // SAFETY: The `CREATESRUCTA` structure is guaranteed by the Win32
            // API to be valid if we've received an event of type `WM_NCCREATE`.
            let self_ = unsafe { (*create_struct).lpCreateParams } as *const Self;

            // SAFETY: `self` is within an Rc which we don't release until the
            // window is destroyed. We are still creating the window here and
            // our message loop is single threaded so no other window activity
            // could be happening.
            unsafe { (*self_).hwnd.set(hwnd.0) };

            chk!(last_err; SetWindowLongPtrA(hwnd, GWLP_USERDATA, self_ as _)).unwrap();
            chk!(last_err; SetWindowLongPtrA(hwnd, GWLP_WNDPROC, (Self::wnd_proc_thunk as usize) as isize))
                .unwrap();
        }

        // We _always_ pass our message through to the default window procedure.
        unsafe { DefWindowProcA(hwnd, umsg, wparam, lparam) }
    }

    /// A minimal shim which forwards Win32 window proc messages to our own
    /// type for handling.
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
                Rc::increment_strong_count(self_);
                if Rc::from_raw(self_).handle_message(umsg, wparam, lparam) == Forwarding::None {
                    return LRESULT(0);
                }
            }
        }

        unsafe { DefWindowProcA(hwnd, umsg, wparam, lparam) }
    }
}
