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
mod wnd;

use class::*;
use inner::*;
pub use wnd::*;
