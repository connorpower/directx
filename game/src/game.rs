use crate::resources::FERRIS_ICON;

use ::d2d::{brushes::SolidColorBrush, win_ui_colors, Color, D2DFactory, RenderTarget};
use ::std::rc::Rc;
use ::tracing::info;
use ::win32::{errors::Result, window::Window};
use ::win_geom::d2::{Point2D, Size2D};
use ::windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, PostQuitMessage, TranslateMessage, MSG,
};
use win_geom::d2::Rect2D;

struct DeviceResources {
    light_slate_gray_brush: SolidColorBrush,
    cornflower_blue_brush: SolidColorBrush,
}

impl DeviceResources {
    fn make(render_target: &mut RenderTarget) -> Self {
        Self {
            light_slate_gray_brush: render_target
                .make_solid_color_brush(win_ui_colors::light_slate_gray()),
            cornflower_blue_brush: render_target
                .make_solid_color_brush(win_ui_colors::cornflower_blue()),
        }
    }
}

pub struct Game {
    main_window: Window,

    _factory: Rc<D2DFactory>,
    render_target: RenderTarget,
    resources: DeviceResources,

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
        let dimension = Size2D {
            width: 800,
            height: 600,
        };

        let main_window = Window::new(dimension, "Main Window", Some(FERRIS_ICON.id().into()))
            .expect("Failed to create main window");

        let system_dpi = unsafe { ::windows::Win32::UI::HiDpi::GetDpiForSystem() };
        let window_dpi = main_window.dpi();

        ::tracing::debug!("System DPI: {system_dpi}");
        ::tracing::debug!("Window DPI: {window_dpi}");

        let factory = D2DFactory::new().expect("Failed to create Direct2D factory");
        let mut render_target = factory.make_render_target(main_window.hwnd(), dimension);
        let resources = DeviceResources::make(&mut render_target);

        Self {
            main_window,
            _factory: factory,
            render_target,
            resources,
            is_render_dirty: true,
            is_shutting_down: false,
        }
    }

    fn update(&mut self) {
        // TODO...
    }

    fn draw(&mut self) {
        if !self.is_render_dirty {
            return;
        }

        let mut ctx = self.render_target.begin_draw();
        ctx.clear(Color::white());

        let u_dim = self.main_window.size();
        let f_dim = u_dim.cast::<f32>();

        // Draw light grey grid with 10px squares
        let stroke_width = 0.5;
        for x in (0..u_dim.width).step_by(10).map(|u| u as f32) {
            ctx.draw_line(
                Point2D { x, y: 0.0 },
                Point2D { x, y: f_dim.height },
                stroke_width,
                &mut self.resources.light_slate_gray_brush,
            );
        }
        for y in (0..u_dim.height).step_by(10).map(|u| u as f32) {
            ctx.draw_line(
                Point2D { x: 0.0, y },
                Point2D { x: f_dim.width, y },
                stroke_width,
                &mut self.resources.light_slate_gray_brush,
            );
        }

        // Draw two rectangles, one inner filled gray and one outer stroked blue
        let stroke_width = 1.0;
        ctx.fill_rect(
            Rect2D {
                left: (u_dim.width / 2 - 50) as _,
                right: (u_dim.width / 2 + 50) as _,
                top: (u_dim.height / 2 - 50) as _,
                bottom: (u_dim.height / 2 + 50) as _,
            },
            &mut self.resources.light_slate_gray_brush,
        );
        ctx.stroke_rect(
            Rect2D {
                left: (u_dim.width / 2 - 100) as _,
                right: (u_dim.width / 2 + 100) as _,
                top: (u_dim.height / 2 - 100) as _,
                bottom: (u_dim.height / 2 + 100) as _,
            },
            &mut self.resources.cornflower_blue_brush,
            stroke_width,
        );

        ctx.end_draw();
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
