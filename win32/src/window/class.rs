//! Win32 windows classs wrapper. Helps define classes e.g. "categories" of
//! windows.

use crate::{errors::*, invoke::chk, types::*};

use ::std::{
    fmt::Write,
    sync::{Arc, Weak as SyncWeak},
};
use ::tracing::{debug, error};
use ::widestring::U16CString;
use ::windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            LoadCursorW, LoadImageW, RegisterClassExW, UnregisterClassW, CS_HREDRAW, CS_VREDRAW,
            HICON, IDC_ARROW, IMAGE_ICON, LR_DEFAULTSIZE, WNDCLASSEXW,
        },
    },
};

use ::lazy_static::lazy_static;
use ::parking_lot::Mutex;
use ::std::collections::{hash_map::Entry, HashMap};

/// Typedef for the Win32 windows procedure function - the primary entry point
/// for the Windows message pump.
type WndProc = extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;

lazy_static! {
    static ref WINDOW_REGISTRATIONS: Mutex<HashMap<U16CString, SyncWeak<WindowClass>>> =
        Default::default();
}

pub(super) struct WindowClass {
    class_name: U16CString,
}

impl WindowClass {
    /// Gets a handle to an existing window class registration, or registers
    /// the window class for the first time.
    pub(super) fn get_or_create(
        class_name_prefix: &str,
        icon_id: Option<ResourceId>,
        wnd_proc_setup: WndProc,
    ) -> Result<Arc<Self>> {
        let mut registry = WINDOW_REGISTRATIONS.lock();
        let mut class_name = class_name_prefix.to_owned();
        if let Some(icon) = icon_id {
            class_name.write_fmt(format_args!("-{icon}")).unwrap();
        }
        let class_name =
            U16CString::from_str(class_name).expect("Null byte found in window class name");

        match registry.entry(class_name) {
            Entry::Vacant(entry) => {
                let class = Self::register(entry.key().clone(), icon_id, wnd_proc_setup)?;
                entry.insert(Arc::downgrade(&class));
                Ok(class)
            }
            Entry::Occupied(mut entry) => {
                if let Some(strong_ref) = entry.get().upgrade() {
                    Ok(strong_ref)
                } else {
                    let class = Self::register(entry.key().clone(), icon_id, wnd_proc_setup)?;
                    entry.insert(Arc::downgrade(&class));
                    Ok(class)
                }
            }
        }
    }

    pub(super) fn class_name(&self) -> &U16CString {
        &self.class_name
    }

    fn register(
        class_name: U16CString,
        icon_id: Option<ResourceId>,
        wnd_proc_setup: WndProc,
    ) -> Result<Arc<Self>> {
        debug!(
            wnd_class = class_name.to_string_lossy(),
            "Register window class"
        );

        let module = chk!(res; GetModuleHandleW(None))?;
        let cursor = chk!(res;
            LoadCursorW(
                HINSTANCE::default(),
                IDC_ARROW
            )
        )?;
        let icon = icon_id
            .map(|resource_id: ResourceId| {
                chk!(res;
                    LoadImageW(
                        module,
                        resource_id.into_pcwstr(),
                        IMAGE_ICON,
                        0,
                        0,
                        LR_DEFAULTSIZE
                    )
                )
            })
            .transpose()?;

        let wnd_class = WNDCLASSEXW {
            cbSize: ::std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc_setup),
            lpszClassName: PCWSTR::from_raw(class_name.as_ptr()),
            hCursor: cursor,
            hIcon: HICON(icon.map(|i| i.0).unwrap_or(0)),
            ..Default::default()
        };
        let _atom = chk!(nonzero_u16; RegisterClassExW(&wnd_class))?;

        Ok(Arc::new(Self { class_name }))
    }

    fn unregister(&self) -> Result<()> {
        debug!(wnd_class = ?self.class_name(), "Unregister window class");
        let module = chk!(res; GetModuleHandleW(None))?;
        chk!(bool; UnregisterClassW(PCWSTR::from_raw(self.class_name().as_ptr()), module))?;
        Ok(())
    }
}

impl Drop for WindowClass {
    fn drop(&mut self) {
        if let Err(e) = self.unregister() {
            error!(error = %e);
        }
    }
}
