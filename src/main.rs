use ::windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageA, GetMessageA, TranslateMessage, WaitMessage, MSG,
};

use ::directx::win32::{
    window::{MainWindow, Window},
    *,
};

pub fn main() {
    match run() {
        Ok(_) => (),
        Err(e) => {
            eprintln!("{e}");
        }
    }
}

fn run() -> Result<()> {
    let on_paint = || {
        println!("on paint");
    };

    let _main_window = MainWindow::new(on_paint)?;

    // TODO: shift to a tokio loop
    while unsafe { WaitMessage() }.as_bool() {
        let mut msg = MSG::default();
        if !unsafe { GetMessageA(&mut msg, None, 0, 0) }.as_bool() {
            break;
        }
        unsafe { TranslateMessage(&msg) };
        unsafe { DispatchMessageA(&msg) };
    }

    Ok(())
}
