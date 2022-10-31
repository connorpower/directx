use crate::resources::FERRIS_ICON;

use ::tracing::{debug};
use ::win32::{
    geom::Dimension2D,
    window::{Window, WindowState},
    *,
};
use ::windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageA, GetMessageA, PostQuitMessage, TranslateMessage, MSG,
};

pub struct Game {}

impl Game {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(&mut self) -> Result<()> {
        let on_paint = || {
            // no-op
        };

        let mut main_window = {
            let wnd = Window::new(
                Dimension2D {
                    width: 800,
                    height: 600,
                },
                "Main Window",
                Some(FERRIS_ICON.id()),
                on_paint,
            )?;
            let rec = wnd.close_receiver();
            Some((wnd, rec))
        };

        let mut msg = MSG::default();
        while unsafe { GetMessageA(&mut msg, None, 0, 0) }.as_bool() {
            unsafe { TranslateMessage(&msg) };
            unsafe { DispatchMessageA(&msg) };

            if main_window
                .as_ref()
                .map(|(_, rec)| *rec.borrow() == WindowState::CloseRequested)
                .unwrap_or(false)
            {
                debug!("main window requested to close");
                main_window.take();
                unsafe {
                    PostQuitMessage(0);
                }
            }
        }

        Ok(())
    }
}
