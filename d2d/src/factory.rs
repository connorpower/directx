//! Direct2D factory - the starting point for using Direct2D and creating other
//! resources.

use crate::RenderTarget;
use ::std::{cell::UnsafeCell, marker::PhantomData, rc::Rc};
use ::tracing::debug;
use ::win32::{
    errors::Result,
    invoke::{check_res, chk},
    window::DPI,
};
use ::win_geom::d2::Size2D;
use ::windows::Win32::{
    Foundation::HWND,
    Graphics::Direct2D::{
        D2D1CreateFactory, ID2D1Factory, ID2D1HwndRenderTarget, D2D1_ANTIALIAS_MODE_PER_PRIMITIVE,
        D2D1_FACTORY_OPTIONS, D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_FEATURE_LEVEL_10,
        D2D1_HWND_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_PROPERTIES,
        D2D1_RENDER_TARGET_TYPE_HARDWARE,
    },
};

/// A Direct2D factory - the starting point for using Direct2D and creating
/// Direct2D resources.
pub struct D2DFactory {
    inner: ID2D1Factory,

    /// Force !Send & !Sync, as our `factory` can only be used by the thread on
    /// which it was created.
    phantom: PhantomData<UnsafeCell<()>>,
}

impl D2DFactory {
    /// Create a new factory from which all the other Direct2D resources can be
    /// created.
    ///
    /// Only one factory should exist per thread, and it should exist for the
    /// lifetime of the thread, but it _must_ be dropped _before_ the thread
    /// exists in order to cleanly free up resources.
    pub fn new() -> Result<Rc<Self>> {
        let options = D2D1_FACTORY_OPTIONS {
            debugLevel: if cfg!(debug_assertions) {
                debug!("Creating ID2D1Factory with debug level: info");
                ::windows::Win32::Graphics::Direct2D::D2D1_DEBUG_LEVEL_INFORMATION
            } else {
                debug!("Creating ID2D1Factory with no debug level: non");
                ::windows::Win32::Graphics::Direct2D::D2D1_DEBUG_LEVEL_NONE
            },
        };

        let factory: ID2D1Factory =
            chk!(res; D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, Some(&options as _)))?;

        Ok(Rc::new(Self {
            phantom: Default::default(),
            inner: factory,
        }))
    }

    /// Makes a new Direct2D render target which targets a Win32 window.
    ///
    /// # Example
    ///
    /// ```
    /// use ::d2d::D2DFactory;
    /// let factory = D2DFactory::new().unwrap();
    /// ```
    pub fn make_render_target(self: &Rc<Self>, hwnd: HWND, size: Size2D<i32>) -> RenderTarget {
        RenderTarget::new(self, hwnd, size)
    }

    /// (Re-)creates the device render target. Called once on initialization and
    /// anytime that Direct2D reports a hardware error that requires
    /// device-specific resources to be re-created.
    pub(crate) fn make_device_render_target(
        &self,
        hwnd: HWND,
        size: Size2D<i32>,
    ) -> Result<ID2D1HwndRenderTarget> {
        let dpi = DPI::detect(hwnd);

        let render_props = D2D1_RENDER_TARGET_PROPERTIES {
            r#type: D2D1_RENDER_TARGET_TYPE_HARDWARE,
            dpiX: dpi.into(),
            dpiY: dpi.into(),
            minLevel: D2D1_FEATURE_LEVEL_10, // Require DirectX 10
            ..Default::default()
        };

        let pixel_size = dpi.scale_size(size);
        ::tracing::warn!("dpi.scale(size) = {pixel_size:?}");

        let hwnd_target_props = D2D1_HWND_RENDER_TARGET_PROPERTIES {
            hwnd,
            pixelSize: pixel_size.cast::<u32>().into(),
            ..Default::default()
        };

        // TODO: macro parsing for field access, not just free functions
        let render_target = check_res(
            || unsafe {
                self.inner
                    .CreateHwndRenderTarget(&render_props as _, &hwnd_target_props as _)
            },
            "CreateHwndRenderTarget",
        )?;

        unsafe {
            render_target.SetAntialiasMode(D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);
        }

        Ok(render_target)
    }
}
