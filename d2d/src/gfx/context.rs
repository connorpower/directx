//! Graphics context which is used for all concrete drawing operations within a
//! draw cycle.

use ::geom::d2::Point2D;
use ::win32::invoke::check_res;
use ::windows::{
    Foundation::Numerics::Matrix3x2,
    Win32::Graphics::Direct2D::{Common::D2D_RECT_F, ID2D1HwndRenderTarget, D2D1_BRUSH_PROPERTIES},
};

use super::{color::Color, target::RenderTarget};

// TODO: trait
// TODO: ensure !Send, !Sync

/// Drawing context for performing batched operations on an underlying render
/// target. Drawing may _only_ be performed via a `Context`.
///
/// ## Draw Lifecycle
///
/// A `BeginDraw` is automatically called when the context is created
/// and `EndDraw` is automatically called when the context is dropped. During
/// the time that the context is alive, an exclusive reference is held on the
/// renderer to prevent concurrent operations.
pub struct Context<'t> {
    render_target: &'t mut RenderTarget,
}

impl<'t> Context<'t> {
    /// Construct a new [Context] for batching draw calls for the current frame.
    pub(crate) fn new(render_target: &'t mut RenderTarget) -> Self {
        let ctx = Self { render_target };
        unsafe {
            ctx.tgt().BeginDraw();
        }
        ctx
    }

    /// Clears the entire screen, setting the color to `color`.
    pub fn clear(&self, color: Color) {
        unsafe {
            self.tgt().Clear(Some(color.as_d2d1_color()));
        }
    }

    /// TEMP/HACK
    /// Put a single pixel to the screen of `color` at `coord`.
    pub fn put_pixel(&self, coord: Point2D<f32>, color: Color) {
        // TODO: cache brushes

        let brush_props = D2D1_BRUSH_PROPERTIES {
            opacity: 1.0,
            transform: Matrix3x2::identity(),
        };
        let brush = check_res(
            || unsafe {
                self.tgt()
                    .CreateSolidColorBrush(color.as_d2d1_color(), Some(&brush_props as _))
            },
            "CreateSolidColorBrush",
        )
        .expect("failed to create brush for put_pixel");

        let rect = D2D_RECT_F {
            left: coord.x,
            top: coord.y,
            right: coord.x + 1.0,
            bottom: coord.y + 1.0,
        };
        unsafe {
            self.tgt().FillRectangle(&rect as _, &brush);
        }
    }

    /// Private syntactic sugar to retrieve the Direct2D render target.
    fn tgt(&self) -> &ID2D1HwndRenderTarget {
        self.render_target.target()
    }
}

impl<'t> Drop for Context<'t> {
    /// Drops the context, automatically committing the batched drawing
    /// commands.
    fn drop(&mut self) {
        check_res(|| unsafe { self.tgt().EndDraw(None, None) }, "EndDraw").unwrap();
    }
}