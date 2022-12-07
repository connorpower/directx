//! Top-level rust Window object which abstracts the underlying Win32 API.

use crate::{
    errors::*,
    input::keyboard::Keyboard,
    invoke::chk,
    types::*,
    window::{Theme, WindowInner, DPI},
};

use ::std::{ops::DerefMut, rc::Rc};
use ::tracing::{debug, error};
use ::widestring::U16CString;
use ::win_geom::d2::Size2D;
use ::windows::{
    core::PCWSTR,
    Win32::{Foundation::HWND, UI::WindowsAndMessaging::SetWindowTextW},
};

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
        size: Size2D<i32>,
        title: &str,
        icon_id: Option<ResourceId>,
        theme: Theme,
    ) -> Result<Self> {
        debug!(wnd_title = %title, "Creating window");
        WindowInner::new(size, title, icon_id, theme).map(|inner| Self { inner })
    }

    /// The size of the client area of our Win32 window. The window chrome
    /// is in addition to this size.
    pub fn size(&self) -> Size2D<i32> {
        self.inner.size()
    }

    /// Get a handle to the Win32 window's handle. This is often required when
    /// interacting with other APIs.
    pub fn hwnd(&self) -> HWND {
        self.inner.hwnd()
    }

    /// Sets the window's system theme. This currently only controls the color
    /// of the title bar.
    pub fn current_theme(&self) -> Theme {
        self.inner.current_theme()
    }

    /// Sets the window's title bar to match the given theme.
    pub fn set_theme(&self, theme: Theme) {
        self.inner.set_theme(theme)
    }

    /// Returns the dots per inch (dpi) value for the window.
    pub fn dpi(&self) -> DPI {
        DPI::detect(self.hwnd())
    }

    /// Returns whether the window has requested to close, and immediately
    /// clears this request. Window is not actually closed until it is
    /// dropped, so the close request can be ignored if needed.
    pub fn clear_close_request(&mut self) -> bool {
        self.inner.clear_close_request()
    }

    /// Returns whether the window has requested to redraw, and immediately
    /// clears this request. Window is not actually redrawn until it is painted
    /// by external higher level code, so the close request can be ignored if
    /// needed.
    pub fn clear_redraw_request(&mut self) -> bool {
        self.inner.clear_redraw_request()
    }

    /// Reads the keyboard state. A read lock is held during this process, so
    /// the reference must be dropped for further keyboard input to be handled.
    pub fn keyboard(&self) -> impl DerefMut<Target = Keyboard> + '_ {
        self.inner.keyboard()
    }

    /// Set the window title.
    pub fn set_title(&self, title: &str) -> Result<()> {
        let string = U16CString::from_str_truncate(title);
        chk!(bool; SetWindowTextW(self.hwnd(), PCWSTR::from_raw(string.as_ptr()))).map(|_| ())
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        debug!(wnd_title = %&self.inner.title(), "Dropping window");
        if let Err(e) = self.inner.destroy() {
            error!("Failed to destroy window: {}", e);
        }
    }
}
