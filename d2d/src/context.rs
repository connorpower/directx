//! Graphics context which is used for all concrete drawing operations within a
//! draw cycle.

use ::std::rc::Rc;
use ::win_geom::d2::{Ellipse2D, Point2D, Rect2D, RoundedRect2D, Size2D};
use ::windows::{core::InParam, Win32::Graphics::Direct2D::ID2D1HwndRenderTarget};

use crate::{
    brushes::{Brush, SolidColorBrush},
    Color, DeviceResource, RenderTarget,
};

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
///
/// # let factory = D2DFactory::new().unwrap();
/// # let mut render_target = factory.make_render_target(
/// #     HWND(0),
/// #     Size2D { width: 100, height: 100 },
/// # );
/// let ctx = render_target.begin_draw();
/// ctx.clear(Color::blue());
/// ctx.put_pixel(Point2D { x: 10.0, y: 10.0 }, Color::red());
/// ctx.end_draw();
/// ```
pub struct Context<'t> {
    /// Exclusive reference to the [`RenderTarget`] into which this context is
    /// drawing.
    render_target: &'t mut RenderTarget,
    /// Cached reference to a created and usable HWND hardware render target.
    device_target: Rc<ID2D1HwndRenderTarget>,
}

impl<'t> Context<'t> {
    /// Construct a new [Context] for batching draw calls for the current frame.
    pub(crate) fn new(
        device_target: Rc<ID2D1HwndRenderTarget>,
        render_target: &'t mut RenderTarget,
    ) -> Self {
        Self {
            device_target,
            render_target,
        }
    }

    /// Clears the entire screen by filling with `color`.
    pub fn clear(&self, color: Color) {
        unsafe {
            self.device_target.Clear(Some(&color.into() as _));
        }
    }

    /// TEMP/HACK
    /// Put a single pixel to the screen using `brush` at `coord`.
    /// TODO: allow any type of brush, not just SolidColor
    pub fn put_pixel(&mut self, origin: Point2D<f32>, brush: &mut SolidColorBrush) {
        brush.recreate_if_needed(self.render_target);

        let rect = Rect2D::from_size_and_origin(Size2D::pixel(), origin);
        unsafe {
            self.device_target
                .FillRectangle(&rect.into() as _, brush.device_brush());
        }
    }

    /// Draws a line between the specified points using a solid stroke of width
    /// `stroke_width`.
    pub fn draw_line(
        &mut self,
        p0: Point2D<f32>,
        p1: Point2D<f32>,
        stroke_width: f32,
        brush: &mut SolidColorBrush,
    ) {
        brush.recreate_if_needed(self.render_target);

        unsafe {
            self.device_target.DrawLine(
                p0.into(),
                p1.into(),
                brush.device_brush(),
                stroke_width,
                InParam::null(),
            );
        }
    }

    /// Paints the interior of the specified rectangle.
    pub fn fill_rect(&mut self, rect: Rect2D<f32>, brush: &mut SolidColorBrush) {
        brush.recreate_if_needed(self.render_target);
        unsafe {
            self.device_target
                .FillRectangle(&rect.into() as _, brush.device_brush());
        }
    }

    /// Draws the outline of a rectangle that has the specified dimensions with
    /// a solid color stroke.
    pub fn stroke_rect(
        &mut self,
        rect: Rect2D<f32>,
        brush: &mut SolidColorBrush,
        stroke_width: f32,
    ) {
        brush.recreate_if_needed(self.render_target);
        unsafe {
            self.device_target.DrawRectangle(
                &rect.into() as _,
                brush.device_brush(),
                stroke_width,
                InParam::null(),
            );
        }
    }

    /// Paints the interior of the specified rounded rectangle.
    ///
    /// Even when both [`radius_x`] and [`radius_y`] are zero, a
    /// [`RoundedRect2D`] is different from a [`Rect2D`]. When stroked, the
    /// corners of the rounded rectangle are roundly joined, not mitered
    /// (square).
    ///
    /// [`radius_x`]: RoundedRect2D.radius_x
    /// [`radius_y`]: RoundedRect2D.radius_y
    pub fn fill_rounded_rect(&mut self, rect: RoundedRect2D<f32>, brush: &mut SolidColorBrush) {
        brush.recreate_if_needed(self.render_target);
        unsafe {
            self.device_target
                .FillRoundedRectangle(&rect.into() as _, brush.device_brush());
        }
    }

    /// Draws the outline of a rounded rectangle that has the specified
    /// dimensions with a solid color stroke.
    ///
    /// Even when both [`radius_x`] and [`radius_y`] are zero, a
    /// [`RoundedRect2D`] is different from a [`Rect2D`]. When stroked, the
    /// corners of the rounded rectangle are roundly joined, not mitered
    /// (square).
    ///
    /// [`radius_x`]: RoundedRect2D.radius_x
    /// [`radius_y`]: RoundedRect2D.radius_y
    pub fn stroke_rounded_rect(
        &mut self,
        rect: RoundedRect2D<f32>,
        brush: &mut SolidColorBrush,
        stroke_width: f32,
    ) {
        brush.recreate_if_needed(self.render_target);
        unsafe {
            self.device_target.DrawRoundedRectangle(
                &rect.into() as _,
                brush.device_brush(),
                stroke_width,
                InParam::null(),
            );
        }
    }

    /// Paints the interior of the specified ellipse.
    pub fn fill_ellipse(&mut self, ellipse: Ellipse2D<f32>, brush: &mut SolidColorBrush) {
        brush.recreate_if_needed(self.render_target);
        unsafe {
            self.device_target
                .FillEllipse(&ellipse.into() as _, brush.device_brush());
        }
    }

    /// Draws the outline of an ellipse that has the specified dimensions with a
    /// solid color stroke.
    pub fn stroke_ellipse(
        &mut self,
        ellipse: Ellipse2D<f32>,
        brush: &mut SolidColorBrush,
        stroke_width: f32,
    ) {
        brush.recreate_if_needed(self.render_target);
        unsafe {
            self.device_target.DrawEllipse(
                &ellipse.into() as _,
                brush.device_brush(),
                stroke_width,
                InParam::null(),
            );
        }
    }

    /// Ends drawing operations on the render target causing the changes to
    /// become visible and the render target to become ready for the next
    /// [`begin_draw`](RenderTarget::begin_draw) call.
    pub fn end_draw(self) {
        let Self {
            render_target,
            device_target,
        } = self;

        render_target.end_draw(device_target);
    }
}
