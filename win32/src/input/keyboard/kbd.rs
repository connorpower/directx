//! Keyboard state and text input tracking.

use ::bitvec::prelude::*;
use ::std::{char::REPLACEMENT_CHARACTER, collections::VecDeque};
use ::tracing::trace;
use ::widestring::WideChar;

use super::{KeyCode, KeystrokeFlags};

/// Length of the input queue, after which point the earliest characters are
/// dropped.
const INPUT_QUEUE_CAPACITY: usize = 32;

/// A representation of a Win32 virtual key event. These are purely internal and
/// are consumed by the `Keyboard` type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum KeyEvent {
    KeyDown {
        key_code: KeyCode,
        flags: KeystrokeFlags,
    },
    KeyUp {
        key_code: KeyCode,
        flags: KeystrokeFlags,
    },
    Input {
        wchar: WideChar,
        flags: KeystrokeFlags,
    },
}

/// A simple abstraction over keyboard input to help track pressed keys and a
/// queue of text input.
pub struct Keyboard {
    /// Bitfield which tracks the press state for the keyboard keys.
    pressed: BitArr!(for 255, in usize, Lsb0),
    /// A queue of printable input text which has been fully processed into
    /// valid unicode.
    input_queue: VecDeque<char>,
    /// High surrogate entry from a surrogate pair. This is `Some` pending
    /// receipt of the following low surrogate. Once the low surrogate arrives,
    /// the pair can be converted into a character and appended to
    /// `input_queue`.
    pending_surrogate: Option<WideChar>,
}

impl Keyboard {
    pub(crate) fn new() -> Self {
        Self {
            pressed: bitarr![usize, Lsb0; 0; 255],
            input_queue: VecDeque::with_capacity(INPUT_QUEUE_CAPACITY),
            pending_surrogate: None,
        }
    }

    /// Process an event from the Win32 system and update internal state. This
    /// event will be reflected in the next user call to [is_key_pressed] or
    pub(crate) fn process_evt(&mut self, evt: KeyEvent) {
        match evt {
            KeyEvent::KeyDown { key_code, flags } => {
                if !flags.was_previous_state_down {
                    *self.mut_bit_for_key(key_code).as_mut() = true;
                }
            }
            KeyEvent::KeyUp { key_code, .. } => {
                *self.mut_bit_for_key(key_code).as_mut() = false;
            }
            KeyEvent::Input { wchar, .. } => {
                match self.pending_surrogate.take() {
                    Some(high) => {
                        // Combine surrogates & append to input queue. If anything fails at this
                        // point we don't have a recourse for recovery so we take the unicode
                        // replacement character instead.
                        self.input_queue.extend(
                            char::decode_utf16([high, wchar])
                                .map(|r| r.unwrap_or(REPLACEMENT_CHARACTER)),
                        );
                    }
                    None => match char::decode_utf16([wchar])
                        .next()
                        .expect("Iterator contains a wchar and should yield at least one result")
                    {
                        // If we've received the first high-surrogate, we must first wait for the
                        // following low surrogate.
                        Err(err) => self.pending_surrogate = Some(err.unpaired_surrogate()),
                        // Happy-path for non-surrogate-pair unicode characters
                        Ok(ch) => self.input_queue.push_back(ch),
                    },
                }

                // Trim queue to avoid growing continuously
                while self.input_queue.len() >= INPUT_QUEUE_CAPACITY {
                    let char = self.input_queue.pop_front().unwrap();
                    trace!("Trimming keyboard input queue, dropped '{char}'.");
                }
            }
        }
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        *self.bit_for_key(key).as_ref()
    }

    /// Drains all accumulated characters in the input queue.
    pub fn drain_input_queue(&mut self) -> impl ExactSizeIterator<Item = char> + '_ {
        self.input_queue.drain(..)
    }

    /// Reset all keyboard state.
    pub fn reset(&mut self) {
        self.input_queue.clear();
        self.pending_surrogate = None;
        self.pressed = BitArray::ZERO;
    }

    fn bit_for_key(&self, key: KeyCode) -> impl AsRef<bool> + '_ {
        self.pressed.get(key.value() as usize).unwrap()
    }

    fn mut_bit_for_key(&mut self, key: KeyCode) -> impl AsMut<bool> + '_ {
        self.pressed.get_mut(key.value() as usize).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ::std::ops::Not;
    use ::strum::IntoEnumIterator;
    use ::widestring::u16str;
    use ::windows::Win32::{
        Foundation::{LPARAM, WPARAM},
        UI::WindowsAndMessaging::*,
    };

    #[derive(PartialEq, Eq)]
    enum KeyRepeat {
        Repeat,
        Initial,
    }

    impl KeystrokeFlags {
        fn test_key_down_flags(repeat: KeyRepeat) -> Self {
            Self {
                repeat_count: if repeat == KeyRepeat::Repeat { 1 } else { 0 },
                scan_code: 0x1E, // 'A'
                is_extended_key: false,
                is_alt_pressed: false,
                was_previous_state_down: repeat == KeyRepeat::Repeat,
                is_key_release: false,
            }
        }

        fn test_key_up_flags(repeat: KeyRepeat) -> Self {
            Self {
                repeat_count: if repeat == KeyRepeat::Repeat { 1 } else { 0 },
                scan_code: 0x1E, // 'A'
                is_extended_key: false,
                is_alt_pressed: false,
                was_previous_state_down: repeat == KeyRepeat::Initial,
                is_key_release: true,
            }
        }
    }

    /// A basic smoke test for key pressed events.
    #[test]
    fn test_key_pressed_basic() {
        let mut kbd = Keyboard::new();

        assert!(kbd.is_key_pressed(KeyCode::Up).not());
        kbd.process_evt(KeyEvent::KeyDown {
            key_code: KeyCode::Up,
            flags: KeystrokeFlags::test_key_down_flags(KeyRepeat::Initial),
        });
        assert!(kbd.is_key_pressed(KeyCode::Up));
    }

    /// Tests correct handling of a series of key down and key up events.
    #[test]
    fn test_key_pressed() {
        let mut kbd = Keyboard::new();

        for key_code in KeyCode::iter() {
            assert!(!kbd.is_key_pressed(key_code));
        }

        for evt in [
            KeyEvent::KeyDown {
                key_code: KeyCode::A,
                flags: KeystrokeFlags::test_key_down_flags(KeyRepeat::Initial),
            },
            KeyEvent::KeyDown {
                key_code: KeyCode::Left,
                flags: KeystrokeFlags::test_key_down_flags(KeyRepeat::Initial),
            },
            KeyEvent::KeyDown {
                key_code: KeyCode::Space,
                flags: KeystrokeFlags::test_key_down_flags(KeyRepeat::Initial),
            },
            KeyEvent::KeyDown {
                key_code: KeyCode::Left,
                flags: KeystrokeFlags::test_key_down_flags(KeyRepeat::Repeat),
            },
            KeyEvent::KeyUp {
                key_code: KeyCode::A,
                flags: KeystrokeFlags::test_key_up_flags(KeyRepeat::Initial),
            },
            KeyEvent::KeyDown {
                key_code: KeyCode::Left,
                flags: KeystrokeFlags::test_key_down_flags(KeyRepeat::Repeat),
            },
        ] {
            kbd.process_evt(evt);
        }

        let expected_pressed = [KeyCode::Space, KeyCode::Left];

        for key_code in expected_pressed {
            assert!(kbd.is_key_pressed(key_code));
        }
        for key_code in KeyCode::iter().filter(|key_code| expected_pressed.contains(key_code).not())
        {
            assert!(!kbd.is_key_pressed(key_code));
        }
    }

    /// We expect that a basic stream of ASCII characters (less than the queue
    /// size), should be collected and returned correctly.
    #[test]
    fn test_input_queue_basic() {
        let mut kbd = Keyboard::new();

        // Test state before any events
        let input: String = kbd.drain_input_queue().collect();
        assert!(
            input.is_empty(),
            "Queue should be empty before first input key event event"
        );

        // Add basic ASCII chars to queue
        for evt in "Hello, world!".chars().map(|c| KeyEvent::Input {
            flags: KeystrokeFlags::test_key_down_flags(KeyRepeat::Initial),
            wchar: c as _,
        }) {
            kbd.process_evt(evt);
        }

        // Confirm queue state after events have been processed
        let input: String = kbd.drain_input_queue().collect();
        assert_eq!(&input, "Hello, world!");
        assert!(
            kbd.drain_input_queue().next().is_none(),
            "Queue should be empty after last call to drain"
        );
    }

    /// Test that valid unicode is handled correctly.
    ///
    /// We use a "Musical Symbol G Clef" character which requires surrogate
    /// pairs to encode in UTF16.
    #[test]
    fn test_input_queue_unicode() {
        let mut kbd = Keyboard::new();

        for evt in [0xD834_u16, 0xDD1E, 0x006d, 0x0075, 0x0073, 0x0069, 0x0063]
            .into_iter()
            .map(|wchar| KeyEvent::Input {
                flags: KeystrokeFlags::test_key_down_flags(KeyRepeat::Initial),
                wchar,
            })
        {
            kbd.process_evt(evt);
        }

        let input: String = kbd.drain_input_queue().collect();
        assert_eq!(&input, "𝄞music");
    }

    /// Test pending surrogate pair handling by enqueueing the high surrogate
    /// and expecting that our drain method returns nothing until the following
    /// low surrogate is enqueued.
    ///
    /// We use a "Musical Symbol G Clef" character which requires surrogate
    /// pairs to encode in UTF16.
    #[test]
    fn test_input_queue_surrogate_pair_handling() {
        let mut kbd = Keyboard::new();

        // TODO: do we need to be sending key up char events too?
        kbd.process_evt(KeyEvent::Input {
            flags: KeystrokeFlags::test_key_down_flags(KeyRepeat::Initial),
            wchar: 0xD834,
        });
        assert!(
            kbd.drain_input_queue().next().is_none(),
            "Input queue should wait for following low surrogate before returning"
        );

        kbd.process_evt(KeyEvent::Input {
            flags: KeystrokeFlags::test_key_down_flags(KeyRepeat::Initial),
            wchar: 0xDD1E,
        });

        let input: String = kbd.drain_input_queue().collect();
        assert_eq!(&input, "𝄞");
    }

    /// Test pending surrogate pair handling by enqueueing an out-of-order low
    /// surrogate (high surrogates must precede low surrogates)
    /// and expecting that our drain method immediately returns the replacement
    /// character.
    ///
    /// We use a "Musical Symbol G Clef" character which requires surrogate
    /// pairs to encode in UTF16.
    #[test]
    fn test_input_queue_lone_low_surrogate() {
        let mut kbd = Keyboard::new();

        for evt in [0xD834_u16, 0xDD1E, 0x006d, 0x0075, 0x0073, 0x0069, 0x0063]
            .into_iter()
            .map(|wchar| KeyEvent::Input {
                flags: KeystrokeFlags::test_key_down_flags(KeyRepeat::Initial),
                wchar,
            })
        {
            kbd.process_evt(evt);
        }

        let input: String = kbd.drain_input_queue().collect();
        assert_eq!(&input, "𝄞music");
    }

    // Test that several unicode characters requiring surrogate pairs are correctly
    // captured.
    ///
    /// We use alternating "Musical Symbol G Clef" and "Bridge at Night Emoji"
    /// characters which both require surrogate pairs to encode in UTF16.
    #[test]
    fn test_input_queue_multiple_surrogate_pair_characters() {
        let mut kbd = Keyboard::new();

        for evt in u16str!("𝄞🌉𝄞🌉a𝄞b🌉c")
            .as_slice()
            .into_iter()
            .map(|c| KeyEvent::Input {
                flags: KeystrokeFlags::test_key_down_flags(KeyRepeat::Initial),
                wchar: *c as _,
            })
        {
            kbd.process_evt(evt);
        }

        // Confirm queue state after events have been processed
        let input: String = kbd.drain_input_queue().collect();
        assert_eq!(&input, "𝄞🌉𝄞🌉a𝄞b🌉c");

        assert!(
            kbd.drain_input_queue().next().is_none(),
            "Queue should be empty after last call to drain"
        );
    }

    /// Tests that our input buffer is trimmed to avoid continuous growth if it
    /// is not regularly drained by the caller.
    #[test]
    fn test_input_queue_buffer_trim() {
        let mut kbd = Keyboard::new();

        // Add basic ASCII chars to queue
        for evt in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
            .chars()
            .map(|c| KeyEvent::Input {
                flags: KeystrokeFlags::test_key_down_flags(KeyRepeat::Initial),
                wchar: c as _,
            })
        {
            kbd.process_evt(evt);
        }

        // Confirm queue state after events have been processed
        let input: String = kbd.drain_input_queue().collect();
        assert_eq!(&input, "vwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ");
        assert_eq!(input.len(), INPUT_QUEUE_CAPACITY - 1);

        assert!(
            kbd.drain_input_queue().next().is_none(),
            "Queue should be empty after last call to drain"
        );
    }

    // Test that buffer trimming does not result in surrogate pair truncation.
    // If the first character to be truncated is a high surrogate pair
    // character, then the following low surrogate pair character should be
    // trimmed too.
    ///
    /// We use alternating "Musical Symbol G Clef" and "Bridge at Night Emoji"
    /// characters which both require surrogate pairs to encode in UTF16.
    #[test]
    fn test_input_queue_buffer_trim_unicode() {
        let mut kbd = Keyboard::new();

        for evt in u16str!("𝄞🌉1𝄞🌉2𝄞🌉3𝄞🌉4𝄞🌉5𝄞🌉6𝄞🌉7𝄞🌉8𝄞🌉9𝄞🌉0𝄞🌉A𝄞🌉B𝄞🌉C𝄞🌉")
            .as_slice()
            .into_iter()
            .map(|c| KeyEvent::Input {
                flags: KeystrokeFlags::test_key_down_flags(KeyRepeat::Initial),
                wchar: *c as _,
            })
        {
            kbd.process_evt(evt);
        }

        // Confirm queue state after events have been processed
        let input: String = kbd.drain_input_queue().collect();
        assert_eq!(&input, "🌉4𝄞🌉5𝄞🌉6𝄞🌉7𝄞🌉8𝄞🌉9𝄞🌉0𝄞🌉A𝄞🌉B𝄞🌉C𝄞🌉");
        assert_eq!(input.chars().count(), INPUT_QUEUE_CAPACITY - 1);

        assert!(
            kbd.drain_input_queue().next().is_none(),
            "Queue should be empty after last call to drain"
        );
    }

    /// Test text entry for 'ö' (" + o combo on international keyboard).
    ///
    /// Events were captured via debugging utils.
    #[test]
    fn test_input_queue_international_input() {
        use super::super::Adapter;
        let mut kbd = Keyboard::new();

        for (umsg, wparam, lparam) in [
            (WM_KEYDOWN, 0x10, 0x002A0001),
            (WM_KEYDOWN, 0xDE, 0x00280001),
            (WM_DEADCHAR, 0x22, 0x00280001),
            (WM_KEYUP, 0xDE, 0xC0280001),
            (WM_KEYUP, 0x10, 0xC02A0001),
            (WM_KEYDOWN, 0x4F, 0x00180001),
            (WM_CHAR, 0xF6, 0x00180001),
            (WM_KEYUP, 0x4F, 0xC0180001),
        ] {
            if let Some(evt) = Adapter::adapt(umsg, WPARAM(wparam), LPARAM(lparam)) {
                kbd.process_evt(evt);
            }
        }

        let input: String = kbd.drain_input_queue().collect();
        assert_eq!(input, "ö");
        for key_code in KeyCode::iter() {
            assert!(
                !kbd.is_key_pressed(key_code),
                "{key_code:?} key still pressed"
            );
        }
    }

    /// Test emoji input for 👌 (using emoji keyboard: "win-.")
    ///
    /// Events captured using debug utils.
    #[test]
    fn test_input_queue_emoji() {
        use super::super::Adapter;
        let mut kbd = Keyboard::new();

        for (umsg, wparam, lparam) in [
            (WM_IME_REQUEST, 0x0006, 0x643E50BC90),
            (WM_GETICON, 0x0000, 0x0000000078),
            (WM_KEYDOWN, 0x005B, 0x00015B0001),
            (WM_KEYUP, 0x00BE, 0x0080340001),
            (WM_KEYUP, 0x005B, 0x00C15B0001),
            (WM_IME_STARTCOMPOSITION, 0x0000, 0x0000000000),
            (WM_IME_NOTIFY, 0x000F, 0x0020600A01),
            (WM_IME_NOTIFY, 0x000F, 0x0020600A01),
            (WM_IME_KEYLAST, 0xD83D, 0x0000000800),
            (WM_IME_CHAR, 0xD83D, 0x0000000001),
            (WM_IME_CHAR, 0xDC4C, 0x0000000001),
            (WM_IME_NOTIFY, 0x010D, 0x0000000000),
            (WM_IME_ENDCOMPOSITION, 0x0000, 0x0000000000),
            (WM_IME_NOTIFY, 0x010E, 0x0000000000),
            (WM_CHAR, 0xD83D, 0x0000000001),
            (WM_CHAR, 0xDC4C, 0x0000000001),
            (0xC052, 0x0001, 0x643E50D570), // Unknown message
            (WM_IME_REQUEST, 0x0006, 0x643E50D570),
        ] {
            if let Some(evt) = Adapter::adapt(umsg, WPARAM(wparam), LPARAM(lparam)) {
                println!("{evt:#?}");
                kbd.process_evt(evt);
            }
        }

        let input: String = kbd.drain_input_queue().collect();
        assert_eq!(input, "👌");
        for key_code in KeyCode::iter() {
            assert!(
                !kbd.is_key_pressed(key_code),
                "{key_code:?} key still pressed"
            );
        }
    }
}
