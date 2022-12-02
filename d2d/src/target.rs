use crate::context::Context;
use ::std::{cell::UnsafeCell, marker::PhantomData};
use ::windows::Win32::Graphics::Direct2D::ID2D1HwndRenderTarget;

/// A [`RenderTarget`]
pub struct RenderTarget {
    // TODO: abstract HWND or DXGISurfaceTarget behind common trait
    inner: ID2D1HwndRenderTarget,

    /// Force !Send & !Sync, as our `render_target` can only be used by the
    /// thread on which it was created.
    phantom: PhantomData<UnsafeCell<()>>,
}

// Crate-internal interface.
impl RenderTarget {
    /// Crate-internal constructor, called by the [`Factory`](super::Factory).
    pub(crate) fn new(inner: ID2D1HwndRenderTarget) -> Self {
        Self {
            phantom: Default::default(),
            inner,
        }
    }

    /// Returns a crate-internal reference to the underlying Direct2D render
    /// target.
    pub(crate) fn target(&self) -> &ID2D1HwndRenderTarget {
        &self.inner
    }
}

// Public interface.
impl RenderTarget {
    /// Make a new drawing [Context] for drawing the next frame. `BeginDraw` and
    /// `EndDraw` will be automatically called and tied to the lifetime of the
    /// `Context`. Drawing can _only_ be achieved via a [Context]. A new
    /// [Context] should be created for each frame.
    pub fn make_context(&mut self) -> Context<'_> {
        Context::new(self)
    }
}
