use ::geom::d2::Dimension2D;
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

pub struct Graphics {
    d2d_factory: ID2D1Factory,
    // TODO: abstract HWND or DXGISurfaceTarget behind common trait
    render_target: ID2D1HwndRenderTarget,
}

impl Graphics {
    pub fn new_with_hwnd(hwnd: HWND, dimension: Dimension2D<i32>) -> Result<Self> {
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
            pixelSize: dimension
                .map::<u32>()
                .expect("Window dimension not representable by u32")
                .into(),
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
            d2d_factory,
            render_target,
        })
    }
}
