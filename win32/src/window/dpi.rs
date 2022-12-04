use ::num_traits::{AsPrimitive, Num};
use ::std::fmt::{self, Debug, Display};
use ::win_geom::d2::{Rect2D, Size2D};
use ::windows::Win32::{Foundation::HWND, UI::HiDpi::GetDpiForWindow};

/// The DPI of a monitor or device, used to handle high-DPI rendering. DPI
/// stands for dots per inch, where a dot represents a physical device pixel.
#[derive(Clone, Copy, Debug)]
pub struct DPI(f32);

impl Display for DPI {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dpi = self.0 as u32;
        let pct = ((self.0 / 96.0) * 100.0).ceil() as u32;
        write!(f, "{dpi} dpi ({pct}%)")
    }
}

impl DPI {
    /// Returns the dots per inch (dpi) value for the specified window.
    pub fn detect(hwnd: HWND) -> Self {
        let val = unsafe { GetDpiForWindow(hwnd) };
        Self(val as _)
    }
}

impl From<f32> for DPI {
    fn from(val: f32) -> Self {
        Self(val)
    }
}

impl From<DPI> for f32 {
    fn from(val: DPI) -> Self {
        val.0 as _
    }
}

impl From<DPI> for u32 {
    fn from(val: DPI) -> Self {
        val.0 as _
    }
}

impl DPI {
    /// Scale a Device Independent Pixel (DIP) by the DPI to an equivalent raw
    /// pixel dimension.
    ///
    /// Normally, all rendering and co-ordinates are expressed in DIPs (Device
    /// Independent Pixels), which enables the application to scale
    /// automatically when the DPI setting changes. Some older Win32 APIs
    /// require parameters in the form of raw pixels, so a DIP must be scaled
    /// accordingly.
    pub fn scale_dip<T>(&self, dip: T) -> T
    where
        T: Num + Clone + Copy + Debug + AsPrimitive<f32> + 'static,
        f32: AsPrimitive<T>,
    {
        (dip.as_() * self.0 / 96.0).ceil().as_()
    }

    /// Scale a [`Size2D`] representing Device Independent Pixels (DIP) by the
    /// DPI to an equivalent [`Size2D`] in raw pixel dimensions.
    ///
    /// Normally, all rendering and co-ordinates are expressed in DIPs (Device
    /// Independent Pixels), which enables the application to scale
    /// automatically when the DPI setting changes. Some older Win32 APIs
    /// require parameters in the form of raw pixels, so a DIP must be scaled
    /// accordingly.
    pub fn scale_size<T>(&self, size: Size2D<T>) -> Size2D<T>
    where
        T: Num + Clone + Copy + Debug + AsPrimitive<f32> + 'static,
        f32: AsPrimitive<T>,
    {
        Size2D {
            width: self.scale_dip(size.width),
            height: self.scale_dip(size.height),
        }
    }

    /// Scale a [`Rect2D`] representing Device Independent Pixels (DIP) by the
    /// DPI to an equivalent [`Rect2D`] in raw pixel dimensions.
    ///
    /// Normally, all rendering and co-ordinates are expressed in DIPs (Device
    /// Independent Pixels), which enables the application to scale
    /// automatically when the DPI setting changes. Some older Win32 APIs
    /// require parameters in the form of raw pixels, so a DIP must be scaled
    /// accordingly.
    pub fn scale_rect<T>(&self, rect: Rect2D<T>) -> Rect2D<T>
    where
        T: Num + Clone + Copy + Debug + AsPrimitive<f32> + 'static,
        f32: AsPrimitive<T>,
    {
        Rect2D {
            left: self.scale_dip(rect.left),
            right: self.scale_dip(rect.right),
            top: self.scale_dip(rect.top),
            bottom: self.scale_dip(rect.bottom),
        }
    }
}
