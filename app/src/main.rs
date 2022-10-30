#![windows_subsystem = "windows"]

mod resources;
mod trace;

use crate::resources::FERRIS_ICON;
use ::tracing::{debug, error, info};
use ::win32::{
    geom::Dimension2D,
    window::{Window, WindowState},
    *,
};
use ::windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageA, GetMessageA, PostQuitMessage, TranslateMessage, MSG,
};

#[::tokio::main]
pub async fn main() {
    crate::trace::configure();

    info!(
        version = env!("CARGO_PKG_VERSION"),
        bin = env!("CARGO_BIN_NAME"),
        "Starting"
    );

    if let Err(e) = run().await {
        error!(error = %e);
    }

    info!(
        version = env!("CARGO_PKG_VERSION"),
        bin = env!("CARGO_BIN_NAME"),
        "Terminating"
    );
}

async fn run() -> Result<()> {
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

    let mut secondary_window = {
        let wnd = Window::new(
            Dimension2D {
                width: 400,
                height: 300,
            },
            "Secondary Window",
            None,
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
            // Only closing the main window will result in app termination.
            unsafe {
                PostQuitMessage(0);
            }
        }

        if secondary_window
            .as_ref()
            .map(|(_, rec)| *rec.borrow() == WindowState::CloseRequested)
            .unwrap_or(false)
        {
            debug!("secondary window requested to close");
            secondary_window.take();
        }
    }

    Ok(())
}
