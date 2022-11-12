//! Rust wrappers around Win32 Window types.
//!
//! ## Interop Relationships
//!
//! This module exposes a top level [Window] which is little more than a wrapper
//! around an Arc'ed `InnerWindow`. The `InnerWindow` is held onto by both the
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
//! ```
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

pub(crate) mod class;
pub(crate) mod inner;

use class::*;
use inner::*;

use crate::{errors::*, geom::Dimension2D, types::*, window::WindowInner};

use ::std::rc::Rc;
use ::tracing::{debug, error};
use ::windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};

/// Typedef for the Win32 windows procedure function - the primary entry piont
/// for the Windows message pump.
type WndProc = extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;

/// A rusty wrapper around Win32 window class.
///
/// A [Window] is `!Sync + !Send` as Win32 windows must be controlled by the
/// same thread on which they were created.
pub struct Window {
    /// The inner refcounted window object. A clone of this object is held on
    /// the win32 API side and should be released when the window is destroyed.
    inner: Rc<WindowInner>,
}

impl Window {
    /// Construct and display a new window.
    // TODO: parameterize with a class rather than individual components, (e.g.
    // icon_id)...
    pub fn new(
        dimension: Dimension2D<i32>,
        title: &str,
        icon_id: Option<ResourceId>,
    ) -> Result<Self> {
        debug!(wnd_title = %title, "Creating window");
        WindowInner::new(dimension, title, icon_id).map(|inner| Self { inner })
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

    /// Indicates whether a window is requesting to close (usually due to a
    /// user clicking the close button). The window is not immediately closed
    /// and remains usable until it is explicitly dropped.
    pub fn requested_close(&mut self) -> bool {
        self.inner.requested_close()
    }

    // TODO: should allow registration of an handle_message callback with strong
    // message types.
}

impl Drop for Window {
    fn drop(&mut self) {
        debug!(wnd_title = %&self.inner.title(), "Dropping window");
        if let Err(e) = self.inner.destroy() {
            error!("Failed to destroy window: {}", e);
        }
    }
}
