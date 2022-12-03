//! Graphics context which is used for all concrete drawing operations within a
//! draw cycle.

use ::win32::invoke::check_res;
use ::win_geom::d2::{Point2D, Rect2D, Size2D};
use ::windows::{
    Foundation::Numerics::Matrix3x2,
    Win32::Graphics::Direct2D::{ID2D1HwndRenderTarget, D2D1_BRUSH_PROPERTIES},
};

use crate::Color;

/// Drawing context for performing batched operations on an underlying render
/// target. Drawing may _only_ be performed via a `Context`.
///
/// # Example
///
/// ```no_run
/// # use ::win_geom::d2::Size2D;
/// # use ::windows::Win32::Foundation::HWND;
/// # use ::d2d::D2DFactory;
/// use ::win_geom::d2::Point2D;
/// use ::d2d::Color;
/// # let factory = D2DFactory::new().unwrap();
/// # let mut render_target = factory.make_render_target(
/// #     HWND(0),
/// #     Size2D { width: 100, height: 100 },
/// # );
/// let ctx = render_target.begin_draw();
/// ctx.clear(Color::blue());
/// ctx.put_pixel(Point2D { x: 10.0, y: 10.0 }, Color::red());
/// render_target.end_draw(ctx);
/// ```
pub struct Context {
    /// Cached reference to a created and usable HWND hardware render target.
    device_target: ID2D1HwndRenderTarget,
}

impl Context {
    /// Construct a new [Context] for batching draw calls for the current frame.
    pub(crate) fn new(device_target: ID2D1HwndRenderTarget) -> Self {
        Self { device_target }
    }

    /// Consumes the [`Context`] and gives back the inner
    /// `ID2D1HwndRenderTarget`, usually done in preparation for an
    /// [`Target::end_draw`] call.
    pub(crate) fn into_inner(self) -> ID2D1HwndRenderTarget {
        self.device_target
    }

    /// Clears the entire screen by filling with `color`.
    pub fn clear(&self, color: Color) {
        unsafe {
            self.device_target.Clear(Some(&color.into() as _));
        }
    }

    /// TEMP/HACK
    /// Put a single pixel to the screen of `color` at `coord`.
    pub fn put_pixel(&self, origin: Point2D<f32>, color: Color) {
        // TODO: cache brushes

        let brush_props = D2D1_BRUSH_PROPERTIES {
            opacity: 1.0,
            transform: Matrix3x2::identity(),
        };
        let brush = check_res(
            || unsafe {
                self.device_target
                    .CreateSolidColorBrush(&color.into() as _, Some(&brush_props as _))
            },
            "CreateSolidColorBrush",
        )
        .expect("failed to create brush for put_pixel");

        let rect = Rect2D::from_size_and_origin(Size2D::pixel(), origin);
        unsafe {
            self.device_target.FillRectangle(&rect.into() as _, &brush);
        }
    }
}
