//! Direct2D factory - the starting point for using Direct2D and creating other
//! resources.

use crate::RenderTarget;
use ::tracing::debug;
use ::win32::{
    errors::Result,
    invoke::{check_res, chk},
};
use ::win_geom::d2::Size2D;
use ::windows::Win32::{
    Foundation::HWND,
    Graphics::Direct2D::{
        D2D1CreateFactory, ID2D1Factory, D2D1_FACTORY_OPTIONS, D2D1_FACTORY_TYPE_SINGLE_THREADED,
        D2D1_HWND_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_PROPERTIES,
    },
};

/// A Direct2D factory - the starting point for using Direct2D and creating
/// Direct2D resources.
pub struct D2DFactory {
    inner: ID2D1Factory,
}

impl D2DFactory {
    /// Create a new factory from which all the other Direct2D resources can be
    /// created.
    ///
    /// Only one factory should exist, and it should exist for the lifetime of
    /// the process, but it _must_ be dropped _before_ the process exists in
    /// order to cleanly free up resources.
    pub fn new() -> Result<Self> {
        let options = D2D1_FACTORY_OPTIONS {
            debugLevel: if cfg!(debug_assertions) {
                ::windows::Win32::Graphics::Direct2D::D2D1_DEBUG_LEVEL_INFORMATION
            } else {
                ::windows::Win32::Graphics::Direct2D::D2D1_DEBUG_LEVEL_NONE
            },
        };

        debug!("Creating ID2D1Factory with {options:?}");

        let factory: ID2D1Factory =
            chk!(res; D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, Some(&options as _)))?;

        Ok(Self { inner: factory })
    }

    /// Makes a new Direct2D render target which targets a Win32 window.
    ///
    /// # Example
    ///
    /// TODO...
    pub fn make_render_target(&self, hwnd: HWND, size: Size2D<i32>) -> Result<RenderTarget> {
        let render_props = D2D1_RENDER_TARGET_PROPERTIES::default();
        let hwnd_target_props = D2D1_HWND_RENDER_TARGET_PROPERTIES {
            hwnd,
            pixelSize: size.cast::<u32>().into(),
            ..Default::default()
        };

        // TODO: macro parsing for field access, not just free functions
        let target = check_res(
            || unsafe {
                self.inner
                    .CreateHwndRenderTarget(&render_props as _, &hwnd_target_props as _)
            },
            "CreateHwndRenderTarget",
        )?;

        Ok(RenderTarget::new(target))
    }
}
