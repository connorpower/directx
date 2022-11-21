//! Top-level rust Window object which abstracts the underlying Win32 API.

use crate::{errors::*, geom::Dimension2D, types::*, window::WindowInner};

use ::std::rc::Rc;
use ::tracing::{debug, error};
use ::windows::Win32::Foundation::HWND;

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

    /// Returns whether the window has requested to close, and immediately
    /// clears this request. Window is not actually closed until it is
    /// dropped, so the close request can be ignored if needed.
    pub fn clear_close_request(&mut self) -> bool {
        self.inner.clear_close_request()
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
