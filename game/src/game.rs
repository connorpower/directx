use crate::resources::FERRIS_ICON;

use ::tracing::debug;
use ::win32::{geom::Dimension2D, window::Window, *};
use ::windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, PostQuitMessage, TranslateMessage, MSG,
};

pub struct Game {}

impl Game {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut main_window = Option::Some(Window::new(
            Dimension2D {
                width: 800,
                height: 600,
            },
            "Main Window",
            Some(FERRIS_ICON.id().into()),
        )?);

        let mut msg = MSG::default();
        while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
            unsafe { TranslateMessage(&msg) };
            unsafe { DispatchMessageW(&msg) };

            if main_window
                .as_mut()
                .map(Window::clear_close_request)
                .unwrap_or(false)
            {
                debug!("main window requested to close");
                unsafe {
                    PostQuitMessage(0);
                }
                main_window.take();
            }
        }

        Ok(())
    }
}
