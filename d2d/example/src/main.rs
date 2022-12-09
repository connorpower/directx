#![cfg_attr(not(feature = "stdio"), windows_subsystem = "windows")]

use ::d2d::{brushes::SolidColorBrush, Color, D2DFactory, RenderTarget};
use ::std::rc::Rc;
use ::win32::{
    proc::ComLibraryHandle,
    types::ResourceId,
    {
        errors::Result,
        window::{Theme, Window},
    },
};
use ::win_geom::d2::{Point2D, Rect2D, Size2D};
use ::windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, PostQuitMessage, TranslateMessage, MSG,
};

pub fn main() {
    // Ensure COM library is loaded
    let _com_handle = ComLibraryHandle::acquire();

    // Use dimensions which are divisible by 8 to work well on 100%, 125%
    // and 150% DPI.
    let size = Size2D {
        width: 720,
        height: 640,
    };

    // Start our example program and pump the message loop
    let mut example = ExampleApp::new(size);
    example.run_message_loop().unwrap();
}

/// A simple structure which holds our Direct2D device dependent resources.
/// These are cached and re-used across drawing calls.
struct Resources {
    rect_stroke_brush: SolidColorBrush,
    rect_fill_brush: SolidColorBrush,
    grid_brush: SolidColorBrush,
    background_color: Color,
}

impl Resources {
    fn make(render_target: &mut RenderTarget, theme: Theme) -> Self {
        match theme {
            Theme::DarkMode => Self {
                rect_stroke_brush: render_target.make_solid_color_brush(Color::cornflower_blue()),
                rect_fill_brush: render_target.make_solid_color_brush(Color::dark_slate_gray()),
                grid_brush: render_target.make_solid_color_brush(Color::dark_slate_gray()),
                background_color: Color::black(),
            },
            Theme::LightMode => Self {
                rect_stroke_brush: render_target.make_solid_color_brush(Color::cornflower_blue()),
                rect_fill_brush: render_target.make_solid_color_brush(Color::light_slate_gray()),
                grid_brush: render_target.make_solid_color_brush(Color::light_slate_gray()),
                background_color: Color::white(),
            },
        }
    }
}

/// Our example app and all state.
pub struct ExampleApp {
    /// The main window for our native Win32 application.
    main_window: Window,
    /// A reference to the Direct2D factory, which is the primary way to create
    /// Direct2D resources.
    _factory: Rc<D2DFactory>,
    /// Our Direct2D render target which pains the main window's client area.
    render_target: RenderTarget,
    /// Cached device-specific drawing resources re-used in each drawing call.
    resources: Resources,
}

impl ExampleApp {
    /// Build a new app, which includes the main window, and display the window.
    pub fn new(size: Size2D<i32>) -> Self {
        let theme = Theme::DarkMode;

        let main_window = Window::new(size, "Direct2D Example", Some(ResourceId(1)), theme)
            .expect("Failed to create main window");

        let factory = D2DFactory::new().expect("Failed to create Direct2D factory");
        let mut render_target = factory.make_render_target(main_window.hwnd(), size);
        let resources = Resources::make(&mut render_target, theme);

        Self {
            main_window,
            _factory: factory,
            render_target,
            resources,
        }
    }

    /// Draw the main window contents. This is a simple example of Direct2D
    /// drawing and will paint the following:
    ///
    /// * An 8x8 (density independent pixel) grid of alternating red, green,
    /// blue lines * A large outline of a rectangle in the center * A smaller
    /// filled rounded rectangle within the larger rect * A small circle within
    /// the rounded rectangle
    fn draw(&mut self) {
        // Drawing must always begin with a `begin_draw` call. All drawing is
        // done via the returned `Context`, and our render target is held locked
        // until the corresponding `end_draw` call.
        let mut ctx = self.render_target.begin_draw();
        // Erase the last contents by paining the client area white.
        ctx.clear(self.resources.background_color);

        // Cache our main window dimensions both as i32 and f32 values.
        let dimensions = self.main_window.size().cast::<f32>();

        // Draw grid with 8px squares
        let stroke_width = 0.5;
        for x in (0..self.main_window.size().width)
            .step_by(8)
            .map(|u| u as f32)
        {
            ctx.draw_line(
                Point2D { x, y: 0.0 },
                Point2D {
                    x,
                    y: dimensions.height,
                },
                stroke_width,
                &mut self.resources.grid_brush,
            );
        }
        for y in (0..self.main_window.size().height)
            .step_by(8)
            .map(|u| u as f32)
        {
            ctx.draw_line(
                Point2D { x: 0.0, y },
                Point2D {
                    x: dimensions.width,
                    y,
                },
                stroke_width,
                &mut self.resources.grid_brush,
            );
        }

        // Draw two rectangles, one inner filled rectangle and one stroked
        // larger rectangle.
        ctx.fill_rect(
            Rect2D {
                left: (dimensions.width / 2.0 - 56.0),
                right: (dimensions.width / 2.0 + 56.0),
                top: (dimensions.height / 2.0 - 56.0),
                bottom: (dimensions.height / 2.0 + 56.0),
            },
            &mut self.resources.rect_fill_brush,
        );
        let stroke_width = 1.0;
        ctx.stroke_rect(
            Rect2D {
                left: (dimensions.width / 2.0 - 104.0),
                right: (dimensions.width / 2.0 + 104.0),
                top: (dimensions.height / 2.0 - 104.0),
                bottom: (dimensions.height / 2.0 + 104.0),
            },
            &mut self.resources.rect_stroke_brush,
            stroke_width,
        );

        // Drawing must end with `end_draw`. This causes the batched changes to
        // be pushed to the hardware and drawn to the screen. It also releases
        // the lock on the render target which is required before any subsequent
        // draw calls.
        ctx.end_draw();
    }

    /// Pump our Win32 message loop. The inner `main_window` will handle most
    /// aspects, we just need to test for any pending close or redraw flags and
    /// action them accordingly.
    pub fn run_message_loop(&mut self) -> Result<()> {
        let mut msg = MSG::default();
        while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
            unsafe { TranslateMessage(&msg) };
            unsafe { DispatchMessageW(&msg) };

            if self.main_window.clear_redraw_request() {
                self.draw();
            }

            if self.main_window.clear_close_request() {
                unsafe {
                    PostQuitMessage(0);
                }
            }
        }

        Ok(())
    }
}
