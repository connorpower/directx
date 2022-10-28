//! Win32 Window methods & types.
//!
//! ## Interop Relationships
//!
//! This module exposes a top level [Window] which is little more than a wrapper
//! around an Arc'ed [InnerWindow]. The [InnerWindow] is held onto by both the
//! [Window] on the rust side, and also by user data in the Win32 side. This
//! ensures the inner is retained so long as any references exist to it.
//! Destroying a window requires that both the Win32 side and our [Window]
//! relinquish ownership.
//!
//! A separate class registry tracks the registered window classes (here "class"
//! is the Win32 parlance for window definitions). Multiple windows might exist,
//! which share a common definition. Provided any window is still alive, the
//! window class definitions should remain registered with the system. When the
//! last displayed window of a given class is destroyed, we de-register our
//! window class definitions in the system. This all happens transparently to
//! the user.
//!
//! ```text
//!                         ┌────────┐
//!                         │ Window │
//!                         └────┬───┘
//!                              │
//!   App (Rust)                 │
//! ─────────────────────────────┼─────────────────────────────────────────────
//!   Library (Rust)             │
//!                              │                      ┌────────────────┐
//!                              │                      │ Class Registry │
//!                              │strong                └────────┬───────┘
//!                              │                               │
//!                              │                               │weak
//!                              │                               │
//!                     ┌────────▼────────┐ strong     ┌─────────▼────────┐
//!                     │ Rc<InnerWindow> ├────────────► Arc<WindowClass> │
//!                     └────────▲────────┘            └──────────────────┘
//!    Library (Rust)            │
//!  ────────────────────────────┼─────────────────────────────────────────────
//!    System (Win32)            │
//!                              │strong
//!                              │
//!                      ┌───────┴────────┐
//!                      │ HWND User Data │
//!                      └────────────────┘
//!```
//!
//! ## Release Procedure
//!
//! Our window might close in one of two ways. The application might drop the
//! rust window handle first, in which case we're responsible for destroying the
//! resources on the win32 side.
//!
//! Alternatively, the user might close the window using native windows methods
//! (window chrome close button, etc.) in which case the native Win32 side
//! will be the initiator of the process.
//!
//! #### Rust Window Dropped
//!
//! ```text
//!       ┌────────┐       ┌──────────────────┐         ┌───────┐
//!       │ Window │       │ Arc<InnerWindow> │         │ Win32 │
//!       └────┬───┘       └─────────┬────────┘         └───┬───┘
//! drop       │                     │                      │
//! ─────────►┌┴┐ destroy            │                      │
//!           │ ├──────────────────►┌┴┐                     │
//!           │ │                   │ │DestroyWindow()      │
//!           │ │                   │ ├───────────────────►┌┴┐
//!           │ │                   └┬┘                    │ │
//!           │ │                    │                     │ │
//!           │ │                    │     WM_NCDESTROY    │ │
//!           │ │                   ┌┴┐◄───────────────────┤ │
//!           │ │                   │ │                    └┬┘
//!           │ │                   │ │                     │
//!           │ │                   │ │   erase user data   │
//!           │ │                   │ ├───────────────────► X
//!           │ │                   │ │
//!           │ │                   │ │
//!           │ │                   │ │release ARC
//!           │ │                   │ ├──────┐
//!           │ │                   │ │      │
//!           │ │                   │ │◄─────┘
//!           │ │                   └┬┘
//!           │ │                    │
//!           │ │release ARC         │
//!           │ ├──────────────────► X
//!           └┬┘
//!            │
//!            X
//! ```
//! #### Win32 Window Close Button
//!
//! If the window's close button is clicked, then the Win32 system initiates
//! the close operation. Our goal is to communicate this back to the top of our
//! application so our RAII [Window] type can be dropped and initiate the window
//! destruction process.
//!
//! ```text
//! ┌────────────┐   ┌────────┐    ┌──────────────────┐   ┌───────┐
//! │ Tokio/Main │   │ Window │    │ Arc<InnerWindow> │   │ Win32 │
//! └──────┬─────┘   └────┬───┘    └─────────┬────────┘   └───┬───┘
//!        │              │                  │                │
//!        │              │                  │               ┌┴┐ close clicked
//!        │              │                 ┌┴┐     WM_CLOSE │ │◄──────────────
//!        │              │                 │ │◄─────────────┤ │
//!        │              │                 │ │              └┬┘
//!       ┌┴┐             │  CloseRequested │ │               │
//!       │ │◄────────────┼─────────────────┤ │               │
//!       │ │             │                 └┬┘               │
//!       │ │             │                  │                │
//!       │ │drop         │                  │                │
//!       │ ├───────────►┌┴──────────────────┴────────────────┴─────────┐
//!       └┬┘            │                                              │
//!        │             │                                              │
//!                      │      ... See Drop() Sequence Above ...       │
//!                      │                                              │
//!                      │                                              │
//!                      └──────────────────────────────────────────────┘
//! ```
use crate::{
    geom::Dimension2D,
    win32::{errors::*, invoke::chk, window::inner::WindowClass},
};

use ::std::{
    ffi::CString,
    rc::Rc,
    cell::Cell,
    sync::{
        Arc, Weak as SyncWeak,
    },
};
use ::tokio::sync::watch;
use ::windows::{
    core::PCSTR,
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleA,
        UI::WindowsAndMessaging::{
            AdjustWindowRectEx, CreateWindowExA, DefWindowProcA, DestroyWindow, GetWindowLongPtrA,
            LoadCursorA, RegisterClassExA, SetWindowLongPtrA, ShowWindow, UnregisterClassA,
            CREATESTRUCTA, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, GWLP_WNDPROC,
            IDC_ARROW, SW_SHOWNORMAL, WINDOW_EX_STYLE, WM_CLOSE, WM_NCCREATE, WM_NCDESTROY,
            WM_PAINT, WNDCLASSEXA, WS_OVERLAPPEDWINDOW,
        },
    },
};

type WndProc = extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;

/// The state of a window. If the state is `CloseRequested`, the corresponding
/// [Window] should be dropped to action the close.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    /// Window is active and running normally.
    Active,
    /// The window has requested to close - typically because the user clicked
    /// the window's close button.
    CloseRequested,
}

/// The next step to take when handling a window proc message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NextMessageAction {
    /// The message should be forwarded to the system's default implementation.
    ForwardToSystem,
    /// The message has been completely handled. Do not forward the message.
    DontForward,
}

/// A rusty wrapper around Win32 window class.
///
/// A [Window] is `!Sync + !Send` as Win32 windows must be controlled by the
/// same thread on which they were created.
pub struct Window<P>
where
    P: Fn(),
{
    /// The inner refcounted window object. A clone of this object is held on
    /// the win32 API side and should be released when the window is destroyed.
    inner: Rc<WindowInner<P>>,
}

impl<P> Window<P>
where
    P: Fn(),
{
    /// Construct and display a new window.
    pub fn new(dimension: Dimension2D<i32>, title: &str, on_paint: P) -> Result<Self> {
        println!("Creating window: {title}");
        WindowInner::new(dimension, title, on_paint).map(|inner| Self { inner })
    }

    /// The dimensions of the client area of our Win32 window. The window chrome
    /// is in addition to this dimension.
    pub fn dimension(&self) -> Dimension2D<i32> {
        self.inner.dimension()
    }

    /// Get a handle to the Win32 window's handle. This is often required when
    /// interacting with other APIs.
    pub fn hwnd(&self) -> Option<HWND> {
        self.inner.hwnd()
    }

    /// Returns a receiver that can be awaited to monitor the window state.
    /// When the window requests to close, the [Window] should be dropped.
    pub fn close_receiver(&self) -> watch::Receiver<WindowState> {
        self.inner.close_receiver()
    }
}

impl<P> Drop for Window<P>
where
    P: Fn(),
{
    fn drop(&mut self) {
        println!("Dropping window: {title}", title = &self.inner.title);
        if let Err(e) = self.inner.destroy() {
            eprintln!("Failed to destroy window: {}", e);
        }
    }
}

struct WindowInner<P>
where
    P: Fn(),
{
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
    /// A closure invoked when the system requests the window be painted.
    on_paint: P,

    close_sender: watch::Sender<WindowState>,
    close_receiver: watch::Receiver<WindowState>,
}

impl<P> WindowInner<P>
where
    P: Fn(),
{
    /// Construct and display a new window.
    pub fn new(dimension: Dimension2D<i32>, title: &str, on_paint: P) -> Result<Rc<Self>> {
        println!("Creating window inner: {title}");

        let (close_sender, close_receiver) = watch::channel(WindowState::Active);
        let this = Rc::new(Self {
            title: title.to_string(),
            window_class: WindowClass::get_or_create("MainWindow", Self::wnd_proc_setup)?,
            hwnd: Default::default(),
            dimension,
            on_paint,
            close_sender,
            close_receiver,
        });

        let hwnd = {
            let module = chk!(res; GetModuleHandleA(PCSTR::null()))?;
            let mut rect = dimension.into();
            chk!(bool; AdjustWindowRectEx(
                &mut rect,
                WS_OVERLAPPEDWINDOW,
                false,
                WINDOW_EX_STYLE::default()
            ))?;
            let name = CString::new(title).expect("Window name contained null byte");
            chk!(ptr; CreateWindowExA(
                    WINDOW_EX_STYLE::default(),
                    PCSTR::from_raw(this.window_class.class_name().as_ptr() as *const u8),
                    PCSTR::from_raw(name.as_ptr() as *const u8),
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
    pub const fn dimension(&self) -> Dimension2D<i32> {
        self.dimension
    }

    /// Get a handle to the Win32 window's handle. This is often required when
    /// interacting with other APIs.
    ///
    /// If `None`, then the window has already been destroyed on the Win32 side.
    pub fn hwnd(&self) -> Option<HWND> {
        let val = self.hwnd.get();
        if val == 0 {
            None
        } else {
            Some(HWND(val))
        }
    }
    /// Returns a receiver that can be awaited to monitor the window state.
    /// When the window requests to close, the [Window] should be dropped.
    pub fn close_receiver(&self) -> watch::Receiver<WindowState> {
        self.close_receiver.clone()
    }

    fn destroy(&self) -> Result<()> {
        if let Some(h) = self.hwnd() {
            chk!(bool; DestroyWindow(h))?;
        }
        Ok(())
    }

    fn handle_message(&self, umsg: u32, _wparam: WPARAM, _lparam: LPARAM) -> NextMessageAction {
        //println!("{}", crate::win32::debug::msgs::DebugMsg::new(umsg, _wparam, _lparam));

        match umsg {
            WM_PAINT => {
                (self.on_paint)();
            }
            WM_CLOSE => {
                self.close_sender.send_replace(WindowState::CloseRequested);

                return NextMessageAction::DontForward;
            }
            WM_NCDESTROY => {
                println!("Destroying window inner: {title}", title = self.title);

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

        NextMessageAction::ForwardToSystem
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
                if Rc::from_raw(self_).handle_message(umsg, wparam, lparam)
                    == NextMessageAction::DontForward
                {
                    return LRESULT(0);
                }
            }
        }

        unsafe { DefWindowProcA(hwnd, umsg, wparam, lparam) }
    }
}

/// An inner module which hides visibility of certain properties even from
/// types in this current module.
mod inner {
    use super::*;

    use ::lazy_static::lazy_static;
    use ::parking_lot::Mutex;
    use ::std::{
        collections::{hash_map::Entry, HashMap},
        ffi::CStr,
    };

    lazy_static! {
        static ref WINDOW_REGISTRATIONS: Mutex<HashMap<String, SyncWeak<WindowClass>>> =
            Default::default();
    }

    pub(super) struct WindowClass {
        class_name: CString,
    }

    impl WindowClass {
        /// Private constructor.
        fn new(class_name: &str, wnd_proc_setup: WndProc) -> Result<Arc<Self>> {
            let class = Arc::new(Self {
                class_name: CString::new(class_name).expect("Window ClassName contained null byte"),
            });
            class.register(wnd_proc_setup)?;
            Ok(class)
        }

        /// Gets a handle to an existing window class registration, or registers
        /// the window class for the first time.
        pub(super) fn get_or_create(
            class_name: &str,
            wnd_proc_setup: WndProc,
        ) -> Result<Arc<Self>> {
            let mut registry = WINDOW_REGISTRATIONS.lock();
            let key = class_name.to_string();

            match registry.entry(key) {
                Entry::Vacant(entry) => {
                    let class = Self::new(class_name, wnd_proc_setup)?;
                    entry.insert(Arc::downgrade(&class));
                    Ok(class)
                }
                Entry::Occupied(mut entry) => {
                    if let Some(strong_ref) = entry.get().upgrade() {
                        Ok(strong_ref)
                    } else {
                        let class = Self::new(class_name, wnd_proc_setup)?;
                        entry.insert(Arc::downgrade(&class));
                        Ok(class)
                    }
                }
            }
        }

        pub(super) fn class_name(&self) -> &CStr {
            &self.class_name
        }

        fn register(&self, wnd_proc_setup: WndProc) -> Result<()> {
            println!("Registering window class: {:?}", self.class_name());
            let cursor = chk!(res;
                LoadCursorA(
                    HINSTANCE::default(),
                    PCSTR::from_raw(IDC_ARROW.as_ptr() as *const u8)
                )
            )?;

            let wnd_class = WNDCLASSEXA {
                cbSize: ::std::mem::size_of::<WNDCLASSEXA>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wnd_proc_setup),
                lpszClassName: PCSTR::from_raw(self.class_name.as_ptr() as *const u8),
                hCursor: cursor,
                ..Default::default()
            };
            let _atom = chk!(nonzero_u16; RegisterClassExA(&wnd_class))?;

            Ok(())
        }

        fn unregister(&self) -> Result<()> {
            println!("Unregistering window class: {:?}", self.class_name());
            let module = chk!(res; GetModuleHandleA(PCSTR::null()))?;
            chk!(bool; UnregisterClassA(PCSTR::from_raw(self.class_name.as_ptr() as *const u8), module))?;
            Ok(())
        }
    }

    impl Drop for WindowClass {
        fn drop(&mut self) {
            if let Err(e) = self.unregister() {
                eprintln!("{e}");
            }
        }
    }
}
