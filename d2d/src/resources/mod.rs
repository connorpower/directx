//! Device-specific Direct2D resources (brushed, bitmaps, etc.)

use crate::RenderTarget;

pub mod brushes;

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
