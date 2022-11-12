//! Win32 windows classs wrapper. Helps define classes e.g. "categories" of
//! windows.

use crate::{errors::*, invoke::chk, types::*, window::WndProc};

use ::std::sync::{Arc, Weak as SyncWeak};
use ::tracing::{debug, error};
use ::windows::Win32::{
    Foundation::HINSTANCE,
    System::LibraryLoader::GetModuleHandleA,
    UI::WindowsAndMessaging::{
        LoadCursorW, LoadImageA, RegisterClassExA, UnregisterClassA, CS_HREDRAW, CS_VREDRAW, HICON,
        IDC_ARROW, IMAGE_ICON, LR_DEFAULTSIZE, WNDCLASSEXA,
    },
};

use ::lazy_static::lazy_static;
use ::parking_lot::Mutex;
use ::std::collections::{hash_map::Entry, HashMap};

lazy_static! {
    static ref WINDOW_REGISTRATIONS: Mutex<HashMap<String, SyncWeak<WindowClass>>> =
        Default::default();
}

pub(super) struct WindowClass {
    class_name: WinString,
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
        let class_name = format!(
            "{prefix}-{icon}",
            prefix = class_name_prefix,
            icon = icon_id.unwrap_or_default()
        );

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

    pub(super) fn class_name(&self) -> &WinString {
        &self.class_name
    }

    fn register(
        class_name: String,
        icon_id: Option<ResourceId>,
        wnd_proc_setup: WndProc,
    ) -> Result<Arc<Self>> {
        debug!(wnd_class = class_name, "Registering window class");
        let class_name = WinString::new(class_name).expect("Window ClassName contained null byte");

        let module = chk!(res; GetModuleHandleA(None))?;
        let cursor = chk!(res;
            LoadCursorW(
                HINSTANCE::default(),
                IDC_ARROW
            )
        )?;
        let icon = icon_id
            .map(|resource_id| {
                chk!(res;
                    LoadImageA(
                        module,
                        &WinString::from_resource_id(resource_id),
                        IMAGE_ICON,
                        0,
                        0,
                        LR_DEFAULTSIZE
                    )
                )
            })
            .transpose()?;

        let wnd_class = WNDCLASSEXA {
            cbSize: ::std::mem::size_of::<WNDCLASSEXA>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc_setup),
            lpszClassName: (&class_name).into(),
            hCursor: cursor,
            hIcon: HICON(icon.map(|i| i.0).unwrap_or(0)),
            ..Default::default()
        };
        let _atom = chk!(nonzero_u16; RegisterClassExA(&wnd_class))?;

        Ok(Arc::new(Self { class_name }))
    }

    fn unregister(&self) -> Result<()> {
        debug!(wnd_class = %self.class_name(), "Unregistering window class");
        let module = chk!(res; GetModuleHandleA(None))?;
        chk!(bool; UnregisterClassA(self.class_name(), module))?;
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
