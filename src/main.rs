use ::windows::{
    core::*,
    Win32::{Foundation::*, System::LibraryLoader::GetModuleHandleA, UI::WindowsAndMessaging::*},
};

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    umsg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match umsg {
        WM_PAINT => {
            println!("should paint...");
            LRESULT(0)
        }
        WM_CLOSE => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcA(hwnd, umsg, wparam, lparam),
    }
}

fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleA(PCSTR::null())?;
        assert!(instance.0 != 0, "GetModulehandleA failed");

        let class_name = s!("DirectXWindow");
        let wnd_class = WNDCLASSEXA {
            cbSize: ::std::mem::size_of::<WNDCLASSEXA>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            lpszClassName: class_name,
            ..Default::default()
        };

        let atom = RegisterClassExA(&wnd_class);
        assert!(atom != 0, "RegisterClassExA failed");

        let hwnd = CreateWindowExA(
            WINDOW_EX_STYLE::default(),
            class_name,
            s!("Hello, DirectX!"),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            800, // TODO: calculate
            600, // TODO: calculate
            HWND::default(),
            HMENU::default(),
            instance,
            None,
        );
        assert!(hwnd.0 != 0, "CreateWindowExA failed");

        ShowWindow(hwnd, SW_SHOWNORMAL);

        while WaitMessage().as_bool() {
            let mut msg = MSG::default();
            if !GetMessageA(&mut msg, HWND::default(), 0, 0).as_bool() {
                break;
            }
            TranslateMessage(&mut msg);
            DispatchMessageA(&msg);
        }
    }

    Ok(())
}
