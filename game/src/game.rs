use crate::resources::FERRIS_ICON;

use ::d2d::{brushes::SolidColorBrush, Color, D2DFactory, RenderTarget};
use ::std::rc::Rc;
use ::tracing::info;
use ::win32::{errors::Result, window::Window};
use ::win_geom::d2::{Point2D, Size2D};
use ::windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, PostQuitMessage, TranslateMessage, MSG,
};

pub struct Game {
    main_window: Window,
    window_title: String,

    _factory: Rc<D2DFactory>,
    render_target: RenderTarget,

    pixel_brush: SolidColorBrush,

    /// Dirty flag for changes that require rendering. If not dirty, we can skip
    /// rendering.
    is_render_dirty: bool,

    /// Tracks whether the main window is shutting down. If true, we should
    /// continue to pump winproc messages to finalize this process but we should
    /// avoid calling `update()`/`render()` or anything else that might interact
    /// with the window.
    is_shutting_down: bool,
}

impl Game {
    pub fn new() -> Self {
        let window_title = "Main Window".to_string();

        let dimension = Size2D {
            width: 800,
            height: 600,
        };
        let main_window = Window::new(dimension, &window_title, Some(FERRIS_ICON.id().into()))
            .expect("Failed to create main window");

        let factory = D2DFactory::new().expect("Failed to create Direct2D factory");
        let mut render_target = factory.make_render_target(main_window.hwnd(), dimension);

        let pixel_brush = render_target.make_solid_color_brush(Color::red());

        Self {
            main_window,
            window_title,
            _factory: factory,
            render_target,
            pixel_brush,
            is_render_dirty: true,
            is_shutting_down: false,
        }
    }

    fn update(&mut self) {
        let len = self.window_title.len();

        {
            let mut kbd = self.main_window.keyboard();
            let mut input = kbd.drain_input();
            self.window_title.truncate(
                self.window_title
                    .len()
                    .saturating_sub(input.num_backspaces()),
            );
            self.window_title.extend(input.chars());
        }

        if self.window_title.len() != len {
            self.is_render_dirty = true;
        }
    }

    fn draw(&mut self) {
        if !self.is_render_dirty {
            return;
        }

        let mut ctx = self.render_target.begin_draw();
        ctx.clear(Color::blue());
        ctx.put_pixel(Point2D { x: 10.0, y: 10.0 }, &mut self.pixel_brush);

        ctx.end_draw();

        self.main_window.set_title(&self.window_title).unwrap();
        self.is_render_dirty = false;
    }

    pub fn run(&mut self) -> Result<()> {
        let mut msg = MSG::default();
        while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
            unsafe { TranslateMessage(&msg) };
            unsafe { DispatchMessageW(&msg) };

            if self.main_window.clear_close_request() {
                info!("posting quit message");
                unsafe {
                    PostQuitMessage(0);
                }
                self.is_shutting_down = true;
            }

            if !self.is_shutting_down {
                self.update();
                self.draw();
            }
        }

        Ok(())
    }
}
