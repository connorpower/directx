use crate::resources::FERRIS_ICON;

use ::tracing::debug;
use ::win32::{geom::Dimension2D, window::Window, *};
use ::windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, PostQuitMessage, TranslateMessage, MSG,
};

pub struct Game {
    main_window: Window,
}

impl Game {
    pub fn new() -> Self {
        let main_window = Window::new(
            Dimension2D {
                width: 800,
                height: 600,
            },
            "Main Window",
            Some(FERRIS_ICON.id().into()),
        )
        .expect("Failed to create main window");

        Self { main_window }
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut msg = MSG::default();
        while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
            unsafe { TranslateMessage(&msg) };
            unsafe { DispatchMessageW(&msg) };

            if self
                .main_window
                .clear_close_request()
            {
                debug!("main window requested to close");
                unsafe {
                    PostQuitMessage(0);
                }
            }
        }

        Ok(())
    }
}
