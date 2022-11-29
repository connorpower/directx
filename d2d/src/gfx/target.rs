use ::geom::d2::Size2D;
use ::win32::{
    invoke::{check_res, chk},
    Result,
};
use ::windows::Win32::{
    Foundation::HWND,
    Graphics::Direct2D::{
        D2D1CreateFactory, ID2D1Factory, ID2D1HwndRenderTarget, D2D1_FACTORY_OPTIONS,
        D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_HWND_RENDER_TARGET_PROPERTIES,
        D2D1_RENDER_TARGET_PROPERTIES,
    },
};

use super::context::Context;

pub struct RenderTarget {
    _d2d_factory: ID2D1Factory,
    // TODO: abstract HWND or DXGISurfaceTarget behind common trait
    render_target: ID2D1HwndRenderTarget,
}

impl RenderTarget {
    pub fn new_with_hwnd(hwnd: HWND, size: Size2D<i32>) -> Result<Self> {
        // TODO: info-level tracing calls

        let options = D2D1_FACTORY_OPTIONS {
            debugLevel: if cfg!(debug_assertions) {
                ::windows::Win32::Graphics::Direct2D::D2D1_DEBUG_LEVEL_INFORMATION
            } else {
                ::windows::Win32::Graphics::Direct2D::D2D1_DEBUG_LEVEL_NONE
            },
        };

        let d2d_factory: ID2D1Factory =
            chk!(res; D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, Some(&options as _)))?;

        let render_props = D2D1_RENDER_TARGET_PROPERTIES::default();
        let hwnd_target_props = D2D1_HWND_RENDER_TARGET_PROPERTIES {
            hwnd,
            pixelSize: size.cast::<u32>().into(),
            ..Default::default()
        };

        // TODO: macro parsing for field access, not just free functions
        let render_target = check_res(
            || unsafe {
                d2d_factory.CreateHwndRenderTarget(&render_props as _, &hwnd_target_props as _)
            },
            "CreateHwndRenderTarget",
        )?;

        Ok(Self {
            _d2d_factory: d2d_factory,
            render_target,
        })
    }

    /// Make a new drawing [Context] for drawing the next frame. `BeginDraw` and
    /// `EndDraw` will be automatically called and tied to the lifetime of the
    /// `Context`. Drawing can _only_ be achieved via a [Context]. A new
    /// [Context] should be created for each frame.
    pub fn make_context(&mut self) -> Context<'_> {
        Context::new(self)
    }

    /// Returns a crate-private reference to the underlying Direct2D render
    /// target.
    pub(crate) fn target(&self) -> &ID2D1HwndRenderTarget {
        &self.render_target
    }
}
