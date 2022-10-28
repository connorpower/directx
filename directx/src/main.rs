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
    if let Err(e) = run().await {
        eprintln!("{e}");
    }
}

async fn run() -> Result<()> {
    let on_paint = || {
        //        println!("on paint!");
    };

    let mut main_window = {
        let wnd = Window::new(
            Dimension2D {
                width: 800,
                height: 600,
            },
            "Main Window",
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
            println!("main window requested to close");
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
            println!("secondary window requested to close");
            secondary_window.take();
        }
    }

    Ok(())
}
