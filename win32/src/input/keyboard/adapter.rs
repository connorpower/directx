//! Adapter for Win32 keyboard events into their strongly-typed Rust
//! counterparts.

use ::deku::prelude::*;
use ::windows::Win32::{
    Foundation::{LPARAM, WPARAM},
    UI::WindowsAndMessaging::{WM_CHAR, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP},
};

use super::{KeyCode, KeyEvent};

/// Win32 Keystroke message flags as defined here:
/// https://learn.microsoft.com/en-us/windows/win32/inputdev/about-keyboard-input#keystroke-message-flags
#[derive(Clone, Copy, Debug, PartialEq, Eq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub(crate) struct KeystrokeFlags {
    /// Bit 31. The transition state. The value is 1 if the key is being
    /// released, or it is 0 if the key is being pressed.
    #[deku(bits = "1")]
    pub(crate) is_key_release: bool,

    /// Bit 30. The previous key state. The value is 1 if the key is down
    /// before the message is sent, or it is 0 if the key is up.
    #[deku(bits = "1")]
    pub(crate) was_previous_state_down: bool,

    /// Bit 29. The context code.
    ///
    /// For a WM_KEYDOWN or WM_CHAR event, the value is 1 if the ALT key is held
    /// down while the key is pressed; otherwise, the value is 0.
    ///
    /// For a WM_KEYUP event, the value is always 0.
    #[deku(bits = "1")]
    pub(crate) is_alt_pressed: bool,

    /// Bit 24. Indicates whether the key is an extended key, such as the
    /// right-hand ALT and CTRL keys that appear on an enhanced 101- or
    /// 102-key keyboard. The value is 1 if it is an extended key;
    /// otherwise, it is 0.
    #[deku(pad_bits_before = "4", bits = "1")]
    pub(crate) is_extended_key: bool,

    /// Bits 16-23. The scan code. The value depends on the OEM.
    pub(crate) scan_code: u8,

    /// Bits 0-15. The repeat count for the current message. The value is
    /// the number of times the keystroke is auto-repeated as a
    /// result of the user holding down the key. If the keystroke is
    /// held long enough, multiple messages are sent. However, the
    /// repeat count is not cumulative.
    #[deku(bits = "16")]
    pub(crate) repeat_count: u16,
}

impl From<LPARAM> for KeystrokeFlags {
    fn from(lparam: LPARAM) -> Self {
        Self::from_bytes((&(lparam.0 as u32).to_be_bytes(), 0))
            .unwrap()
            .1
    }
}

pub(crate) struct Adapter {}

impl Adapter {
    /// Indicates whether the [Adapter] handles the given message. If it does,
    /// [adapt] should be called and the message should be considered handled
    /// regardless of whether a [KeyEvent] is generated.
    pub(crate) const fn handles_msg(umsg: u32, _wparam: WPARAM, _lparam: LPARAM) -> bool {
        matches!(
            umsg,
            WM_KEYDOWN | WM_SYSKEYDOWN | WM_KEYUP | WM_SYSKEYUP | WM_CHAR
        )
    }

    /// Adapts a Win32 windows procedure function into a [KeyEvent]. This
    /// function should only be called if [handles_msg] indicated that the
    /// [Adapter] will handle a wnd proc message with these parameters.
    pub(crate) fn adapt(umsg: u32, wparam: WPARAM, lparam: LPARAM) -> Option<KeyEvent> {
        match umsg {
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                KeyCode::try_from(wparam.0)
                    .ok()
                    .map(|key_code| KeyEvent::KeyDown {
                        key_code,
                        flags: lparam.into(),
                    })
            }
            WM_KEYUP | WM_SYSKEYUP => {
                KeyCode::try_from(wparam.0)
                    .ok()
                    .map(|key_code| KeyEvent::KeyUp {
                        key_code,
                        flags: lparam.into(),
                    })
            }
            WM_CHAR => Some(KeyEvent::Input {
                wchar: wparam.0 as u16,
                flags: lparam.into(),
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ::pretty_assertions::assert_eq;

    /// Pressing 'h' without any modifiers.
    #[test]
    fn test_key_down() {
        // Event captured via `debug::DebugMsg` dump.
        let event = Adapter::adapt(WM_KEYDOWN, WPARAM(0x48), LPARAM(0x230001)).unwrap();

        assert_eq!(
            event,
            KeyEvent::KeyDown {
                key_code: KeyCode::H,
                flags: KeystrokeFlags {
                    repeat_count: 1,
                    scan_code: 35,
                    is_extended_key: false,
                    is_alt_pressed: false,
                    was_previous_state_down: false,
                    is_key_release: false
                }
            }
        );
    }

    /// Char event emitted when pressing 'h';
    #[test]
    fn test_char_event() {
        // Event captured via `debug::DebugMsg` dump.
        let event = Adapter::adapt(WM_CHAR, WPARAM(0x68), LPARAM(0x230001)).unwrap();

        assert_eq!(
            event,
            KeyEvent::Input {
                wchar: b'h' as u16,
                flags: KeystrokeFlags {
                    repeat_count: 1,
                    scan_code: 35,
                    is_extended_key: false,
                    is_alt_pressed: false,
                    was_previous_state_down: false,
                    is_key_release: false
                }
            }
        );
    }

    /// Test releasing 'h' without any modifiers.
    #[test]
    fn test_key_up() {
        // Event captured via `debug::DebugMsg` dump.
        let event = Adapter::adapt(WM_KEYUP, WPARAM(0x48), LPARAM(0xC0230001))
            .expect("Valid KEYDOWN event should be parsed");

        assert_eq!(
            event,
            KeyEvent::KeyUp {
                key_code: KeyCode::H,
                flags: KeystrokeFlags {
                    repeat_count: 1,
                    scan_code: 35,
                    is_extended_key: false,
                    is_alt_pressed: false,
                    was_previous_state_down: true,
                    is_key_release: true,
                }
            }
        );
    }

    /// Pressing 'alt-h'.
    #[test]
    fn test_key_down_with_modifier() {
        // Event captured via `debug::DebugMsg` dump.
        let event = Adapter::adapt(WM_SYSKEYDOWN, WPARAM(0x48), LPARAM(0x20230001)).unwrap();

        assert_eq!(
            event,
            KeyEvent::KeyDown {
                key_code: KeyCode::H,
                flags: KeystrokeFlags {
                    repeat_count: 1,
                    scan_code: 35,
                    is_extended_key: false,
                    is_alt_pressed: true,
                    was_previous_state_down: false,
                    is_key_release: false
                }
            }
        );
    }

    /// Test releasing 'alt-h'.
    #[test]
    fn test_key_up_with_modifiers() {
        // Event captured via `debug::DebugMsg` dump.
        let event = Adapter::adapt(WM_SYSKEYUP, WPARAM(0x48), LPARAM(0xE0230001)).unwrap();

        assert_eq!(
            event,
            KeyEvent::KeyUp {
                key_code: KeyCode::H,
                flags: KeystrokeFlags {
                    repeat_count: 1,
                    scan_code: 35,
                    is_extended_key: false,
                    is_alt_pressed: true,
                    was_previous_state_down: true,
                    is_key_release: true,
                }
            }
        );
    }

    /// Pressing 'h' with key repeat.
    #[test]
    fn test_key_down_with_repeat() {
        // Event captured via `debug::DebugMsg` dump.
        let event = Adapter::adapt(WM_KEYDOWN, WPARAM(0x48), LPARAM(0x40230001)).unwrap();

        assert_eq!(
            event,
            KeyEvent::KeyDown {
                key_code: KeyCode::H,
                flags: KeystrokeFlags {
                    repeat_count: 1,
                    scan_code: 35,
                    is_extended_key: false,
                    is_alt_pressed: false,
                    was_previous_state_down: true,
                    is_key_release: false
                }
            }
        );
    }
}
