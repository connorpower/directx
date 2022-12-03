use crate::{context::Context, factory::D2DFactory};
use ::std::rc::Rc;
use ::win32::invoke::check_res;
use ::win_geom::d2::Size2D;
use ::windows::Win32::{
    Foundation::{D2DERR_RECREATE_TARGET, HWND},
    Graphics::Direct2D::ID2D1HwndRenderTarget,
};

/// Renders drawing instructions to a window.
///
/// You must call [`begin_draw`] before issuing drawing commands to receive a
/// [`Context`]. All drawing must be done via the returned [`Context`] object.
/// After you've finished drawing, call [`end_draw`] on that [`Context`] object
/// to indicate that drawing is finished and to release access to the buffer
/// backing the render target.
///
/// [`RenderTarget`] objects are double buffered, so drawing commands issued do
/// not appear immediately, but rather are performed on an offscreen surface.
/// When [`end_draw`] is called, if there have been no rendering errors, the
/// offscreen buffer is presented. If there have been rendering errors in the
/// batch flushed by [`end_draw`], then the buffer is not presented, and the
/// application must call [`begin_draw`] and re-draw the frame.
///
/// # Example
///
/// ```no_run
/// # use ::win_geom::d2::Size2D;
/// # use ::windows::Win32::Foundation::HWND;
/// use ::win_geom::d2::Point2D;
/// use ::d2d::{D2DFactory, Color};
///
/// # let hwnd = HWND(0);
/// # let size = Size2D { width: 100, height: 100 };
/// let factory = D2DFactory::new().unwrap();
/// let mut render_target = factory.make_render_target(hwnd, size);
///
/// let ctx = render_target.begin_draw();
/// ctx.clear(Color::blue());
/// ctx.put_pixel(Point2D { x: 10.0, y: 10.0 }, Color::red());
/// ctx.end_draw();
/// ```
///
/// [`begin_draw`]: Self::begin_draw
/// [`end_draw`]: Context::end_draw
pub struct RenderTarget {
    /// State pattern object helps manage the two states we might find ourselves
    /// in:
    ///
    /// * Target created and device specific resources usable
    /// * Target requires re-creation due to hardware device loss or error.
    state: State,
}

impl RenderTarget {
    /// Make a new drawing [Context] for drawing the next frame.
    ///
    /// After [`begin_draw`] is called, a render target will normally build up a
    /// batch of rendering commands, but defer processing of these commands
    /// until either an internal buffer is full, or until [`end_draw`] is
    /// called. Drawing can _only_ be achieved via a [Context]. A new [Context]
    /// should be created for each frame.
    ///
    /// [`begin_draw`]: Self::begin_draw
    /// [`end_draw`]: Context::end_draw
    pub fn begin_draw(&mut self) -> Context<'_> {
        let state = ::std::mem::replace(&mut self.state, State::Poisoned);
        let (new_state, device_target) = state.begin_draw();
        self.state = new_state;

        unsafe {
            device_target.BeginDraw();
        }

        Context::new(device_target, self)
    }

    /// Ends drawing operations on the render target causing the changes to
    /// become visible and the render target to become ready for the next
    /// [`Self::begin_draw`] call.
    pub(crate) fn end_draw(&mut self, device_target: ID2D1HwndRenderTarget) {
        let must_recreate =
            match check_res(|| unsafe { device_target.EndDraw(None, None) }, "EndDraw") {
                Err(e) if e.code() == Some(D2DERR_RECREATE_TARGET) => true,
                Err(e) => panic!("Unexpected error in Direct2D EndDraw(): {e}"),
                Ok(_) => false,
            };

        self.state = ::std::mem::replace(&mut self.state, State::Poisoned)
            .end_draw(must_recreate, device_target);
    }

    /// Crate-internal constructor, called by the [`Factory`](super::Factory).
    pub(crate) fn new(factory: &Rc<D2DFactory>, hwnd: HWND, size: Size2D<i32>) -> Self {
        Self {
            state: State::RequiresRecreation {
                inner: Inner {
                    factory: factory.clone(),
                    hwnd,
                    size,
                },
            },
        }
    }
}

/// Inner components which are common to all states of our state pattern render
/// target.
struct Inner {
    /// The factory which created this [`RenderTarget`]. A reference is kept
    /// so that the [`RenderTarget`] can be automatically re-created from
    /// within if DirectX reports a `D2DERR_RECREATE_TARGET` error and
    /// requires device-specific resources to be recreated.
    factory: Rc<D2DFactory>,

    /// A win32 Window handle that which our render target will draw into.
    // TODO: This should be a `&'window HWND` or similar, or the render target
    // should be a _property_ of the window to ensure object lifetimes are bound
    // together safely.
    hwnd: HWND,

    /// Size of both the window and the render target.
    size: Size2D<i32>,
}

/// The internal state of our render target, encapsulated as a state pattern.
enum State {
    /// Device-specific resources have been recreated and are usable.
    Created {
        inner: Inner,
        // TODO: abstract HWND or DXGISurfaceTarget behind common trait
        target: ID2D1HwndRenderTarget,
    },
    /// The target is currently in a `BeginDraw` call and has given the
    /// underlying `ID2D1HwndRendererTarget` out to to a [`Context`];
    Drawing { inner: Inner },
    /// Device-specific resources require (re-)creation. This is true for the
    /// first interaction and following any `D2DERR_RECREATE_TARGET` errors
    /// received due to device errors.
    RequiresRecreation { inner: Inner },
    /// Poisoned state. An error occurred mid-transition and this type is no
    /// longer usable.
    Poisoned,
}

impl State {
    /// Transitions to the drawing state and returns (or recreates) the device
    /// render target.
    ///
    /// # Panics
    ///
    /// Panics if called while already in the [`Self::Drawing`] state.
    fn begin_draw(self) -> (Self, ID2D1HwndRenderTarget) {
        match self {
            Self::Poisoned => panic!("Render target state poisoned"),
            Self::Drawing { .. } => panic!("Render target should not be re-created mid-draw"),
            Self::Created { target, inner } => (Self::Drawing { inner }, target),
            Self::RequiresRecreation { inner } => {
                let target = inner
                    .factory
                    .make_device_render_target(inner.hwnd, inner.size)
                    .expect("Failed to create device resources for Direct2D render target");

                // Recurse
                Self::Created { inner, target }.begin_draw()
            }
        }
    }

    /// Ends a drawing cycle, transitioning from either [`Self::Drawing`] to
    /// [`Self::RequiresRecreation`] depending on the value of `must_recreate`.
    fn end_draw(self, must_recreate: bool, target: ID2D1HwndRenderTarget) -> Self {
        match self {
            Self::Poisoned => panic!("Render target state poisoned"),
            Self::Created { .. } => {
                panic!("Render target cannot transition from Drawing to Created")
            }
            Self::RequiresRecreation { .. } => {
                panic!("Render target cannot transition from Drawing to RequiresCreation")
            }
            Self::Drawing { inner } => {
                if must_recreate {
                    Self::RequiresRecreation { inner }
                } else {
                    Self::Created { inner, target }
                }
            }
        }
    }
}
