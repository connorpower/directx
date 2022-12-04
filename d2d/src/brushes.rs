//! Device-mapped Direct2D brushes for "painting" areas of a render target.

use crate::{color::Color, RenderTarget};
use ::std::fmt::{self, Debug};
use ::windows::Win32::Graphics::Direct2D::ID2D1SolidColorBrush;
use windows::Win32::Graphics::Direct2D::ID2D1Brush;

/// A trait shared in common with all device-specific resources. A
/// device-specific resource is a Direct2D resource which must be re-created if
/// the render target is lost.
pub(crate) trait DeviceResource {
    /// The generation of the render target for which this resource was created.
    /// If the two generations no longer agree, the resource must be re-created.
    fn generation(&self) -> usize;

    /// Re-create the resource if required (i.e. if the resource's generation no
    /// longer matches that of the [`RenderTarget`]).
    fn recreate_if_needed(&mut self, render_target: &mut RenderTarget);
}

pub(crate) trait Brush {
    fn device_brush(&self) -> &'_ ID2D1Brush;
}

/// A brush which paints an area with a solid color.
///
/// This is a device-specific resource and is tied to the [`RenderTarget`] by
/// which it was created. Cache the brush and re-use on subsequent draw calls
/// for best performance.
pub struct SolidColorBrush {
    /// A copy of the [`Color`] from which the brush was created. This is used
    /// to re-create the brush internally in the event that we must re-create
    /// our device specific resources.
    color: Color,
    /// A cached Direct2D device-specific solid color brush. May become
    /// invalidated if the corresponding render target is re-created.
    device_brush: ID2D1SolidColorBrush,
    /// The generation of the render target for which this brush was created. If
    /// the two generations no longer agree, the brush mush be re-created.
    generation: usize,
}

impl SolidColorBrush {
    /// A crate-private constructor. Only a [`RenderTarget`] should be able to
    /// create brushes.
    pub(crate) fn new(color: Color, device_brush: ID2D1SolidColorBrush, generation: usize) -> Self {
        Self {
            color,
            device_brush,
            generation,
        }
    }

    /// The color of the brush.
    pub fn color(&self) -> Color {
        self.color
    }
}

impl Brush for SolidColorBrush {
    fn device_brush(&self) -> &'_ ID2D1Brush {
        (&self.device_brush).into()
    }
}

impl DeviceResource for SolidColorBrush {
    fn generation(&self) -> usize {
        self.generation
    }

    fn recreate_if_needed(&mut self, render_target: &mut RenderTarget) {
        if self.generation() != render_target.generation() {
            *self = render_target.make_solid_color_brush(self.color);
        }
    }
}

impl Debug for SolidColorBrush {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SolidColorBrush")
            .field("color", &self.color)
            .field("generation", &self.generation)
            .finish()
        // TODO: needs-recreation?
    }
}
