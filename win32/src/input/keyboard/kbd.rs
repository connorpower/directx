//! Keyboard state and text input tracking.

use ::bitvec::prelude::*;
use ::std::{char::REPLACEMENT_CHARACTER, collections::VecDeque};
use ::tracing::trace;
use ::widestring::WideChar;

use super::{KeyCode, KeystrokeFlags};

/// Length of the input queue, after which point the earliest characters are
/// dropped.
const INPUT_QUEUE_CAPACITY: usize = 32;

const BACKSPACE: char = '\x08';

/// An object which encapsulates the state of the input buffer.
pub struct InputBuffer<I>
where
    I: ExactSizeIterator<Item = char>,
{
    chars: I,
    n_backspaces: usize,
}

impl<I> InputBuffer<I>
where
    I: ExactSizeIterator<Item = char>,
{
    /// The number of backspaces which preceded any text in the [Self::chars]
    /// buffer and should be removed from to any _previously_ drained input.
    pub fn num_backspaces(&self) -> usize {
        self.n_backspaces
    }

    /// The current input buffer. Any backspace events which happened within
    /// the buffer have already been applied to the buffer contents.
    pub fn chars(&mut self) -> &mut I {
        &mut self.chars
    }
}

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
    /// The number of pending backspace events which should be applied to any
    /// previously retrieved text.
    n_backspaces: usize,
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
            n_backspaces: 0,
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
                        let low = wchar;
                        // Combine surrogates & append to input queue. If anything fails at this
                        // point we don't have a recourse for recovery so we take the unicode
                        // replacement character instead.
                        self.process_char_input(
                            char::decode_utf16([high, low])
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
                        Ok(ch) => self.process_char_input([ch]),
                    },
                }
            }
        }
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        *self.bit_for_key(key).as_ref()
    }

    /// Drains all accumulated characters in the input queue and clears any
    /// pending backspace events.
    pub fn drain_input(&mut self) -> InputBuffer<impl ExactSizeIterator<Item = char> + '_> {
        let n_backspaces = self.n_backspaces;
        self.n_backspaces = 0;

        InputBuffer {
            n_backspaces,
            chars: self.input_queue.drain(..),
        }
    }

    /// Reset all keyboard state.
    pub fn reset(&mut self) {
        self.input_queue.clear();
        self.pending_surrogate = None;
        self.pressed = BitArray::ZERO;
    }

    /// Handles character input and appends or modifies the input queue. The
    /// char iterator could contain only a single char, multiple characters, and
    /// could include control characters such as backspace or delete.
    /// [process_char_input] will account for deletion events.
    fn process_char_input<I>(&mut self, chars: I)
    where
        I: IntoIterator<Item = char>,
    {
        let chars = chars.into_iter();
        for c in chars {
            match c {
                // TODO: detect delete
                BACKSPACE => {
                    if self.input_queue.pop_back().is_none() {
                        self.n_backspaces += 1;
                    }
                }
                // Drop any control characters that are not whitespace
                _ if c.is_control() && !c.is_whitespace() => (),
                _ => self.input_queue.push_back(c),
            }
        }

        // Trim queue to avoid growing continuously
        while self.input_queue.len() >= INPUT_QUEUE_CAPACITY {
            let char = self.input_queue.pop_front().unwrap();
            trace!("Trimming keyboard input queue, dropped '{char}'.");
        }
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

    mod event_samples {
        use super::*;

        /// An event sampled from winproc messages.
        /// .0 = winproc umsg
        /// .1 = winproc WPARAM
        /// .2 = winproc LPARAM
        pub type EventSample = (u32, usize, isize);

        /// Press and release "a" character.
        pub const PRESS_RELEASE_A: &[EventSample] = &[
            (WM_KEYDOWN, 0x41, 0x000001E0001),
            (WM_CHAR, 0x61, 0x000001E0001),
            (WM_KEYUP, 0x41, 0x000C01E0001),
        ];

        /// Press and release "b" character.
        pub const PRESS_RELEASE_B: &[EventSample] = &[
            (WM_KEYDOWN, 0x42, 0x00000300001),
            (WM_CHAR, 0x62, 0x00000300001),
            (WM_KEYUP, 0x42, 0x000C0300001),
        ];

        /// Press and release "c" character.
        pub const PRESS_RELEASE_C: &[EventSample] = &[
            (WM_KEYDOWN, 0x43, 0x000002E0001),
            (WM_CHAR, 0x63, 0x000002E0001),
            (WM_KEYUP, 0x43, 0x000C02E0001),
        ];

        /// Press and release "backspace" key.
        pub const PRESS_RELEASE_BACKSPACE: &[EventSample] = &[
            (WM_KEYDOWN, 0x8, 0x000000E0001),
            (WM_CHAR, 0x8, 0x000000E0001),
            (WM_KEYUP, 0x8, 0x000C00E0001),
        ];

        /// Text entry for 'ö' ('"' + 'o' combo on international keyboard).
        pub const PRESS_RELEASE_INTERNATIONAL_UMLAUT: &[EventSample] = &[
            (WM_KEYDOWN, 0x10, 0x002A0001),
            (WM_KEYDOWN, 0xDE, 0x00280001),
            (WM_DEADCHAR, 0x22, 0x00280001),
            (WM_KEYUP, 0xDE, 0xC0280001),
            (WM_KEYUP, 0x10, 0xC02A0001),
            (WM_KEYDOWN, 0x4F, 0x00180001),
            (WM_CHAR, 0xF6, 0x00180001),
            (WM_KEYUP, 0x4F, 0xC0180001),
        ];

        /// Emoji input for 👌 using emoji keyboard ("win-.")
        pub const EMOJI_INPUT_OK_HAND: &[EventSample] = &[
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
        ];
    }

    #[derive(PartialEq, Eq)]
    enum KeyRepeat {
        Repeat,
        Initial,
    }

    impl KeystrokeFlags {
        fn test_key_down_flags(repeat: KeyRepeat) -> Self {
            Self {
                repeat_count: u16::from(repeat == KeyRepeat::Repeat),
                scan_code: 0x1E, // 'A'
                is_extended_key: false,
                is_alt_pressed: false,
                was_previous_state_down: repeat == KeyRepeat::Repeat,
                is_key_release: false,
            }
        }

        fn test_key_up_flags(repeat: KeyRepeat) -> Self {
            Self {
                repeat_count: u16::from(repeat == KeyRepeat::Repeat),
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

        assert!(!kbd.is_key_pressed(KeyCode::Up));
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
        let input: String = kbd.drain_input().chars().collect();
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
        let input: String = kbd.drain_input().chars().collect();
        assert_eq!(&input, "Hello, world!");
        assert!(
            kbd.drain_input().chars().next().is_none(),
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

        let input: String = kbd.drain_input().chars().collect();
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

        kbd.process_evt(KeyEvent::Input {
            flags: KeystrokeFlags::test_key_down_flags(KeyRepeat::Initial),
            wchar: 0xD834,
        });
        assert!(
            kbd.drain_input().chars().next().is_none(),
            "Input queue should wait for following low surrogate before returning"
        );

        kbd.process_evt(KeyEvent::Input {
            flags: KeystrokeFlags::test_key_down_flags(KeyRepeat::Initial),
            wchar: 0xDD1E,
        });

        let input: String = kbd.drain_input().chars().collect();
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

        for evt in [0xDD1E, 0x006d].into_iter().map(|wchar| KeyEvent::Input {
            flags: KeystrokeFlags::test_key_down_flags(KeyRepeat::Initial),
            wchar,
        }) {
            kbd.process_evt(evt);
        }

        let input: String = kbd.drain_input().chars().collect();
        assert_eq!(&input, "�m");
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
            .iter()
            .map(|c| KeyEvent::Input {
                flags: KeystrokeFlags::test_key_down_flags(KeyRepeat::Initial),
                wchar: *c as _,
            })
        {
            kbd.process_evt(evt);
        }

        // Confirm queue state after events have been processed
        let input: String = kbd.drain_input().chars().collect();
        assert_eq!(&input, "𝄞🌉𝄞🌉a𝄞b🌉c");

        assert!(
            kbd.drain_input().chars().next().is_none(),
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
        let input: String = kbd.drain_input().chars().collect();
        assert_eq!(&input, "vwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ");
        assert_eq!(input.len(), INPUT_QUEUE_CAPACITY - 1);

        assert!(
            kbd.drain_input().chars().next().is_none(),
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
            .iter()
            .map(|c| KeyEvent::Input {
                flags: KeystrokeFlags::test_key_down_flags(KeyRepeat::Initial),
                wchar: *c as _,
            })
        {
            kbd.process_evt(evt);
        }

        // Confirm queue state after events have been processed
        let input: String = kbd.drain_input().chars().collect();
        assert_eq!(&input, "🌉4𝄞🌉5𝄞🌉6𝄞🌉7𝄞🌉8𝄞🌉9𝄞🌉0𝄞🌉A𝄞🌉B𝄞🌉C𝄞🌉");
        assert_eq!(input.chars().count(), INPUT_QUEUE_CAPACITY - 1);

        assert!(
            kbd.drain_input().chars().next().is_none(),
            "Queue should be empty after last call to drain"
        );
    }

    /// Test text entry for 'ö' ('"' + 'o' combo on international keyboard).
    ///
    /// Events were captured via debugging utils.
    #[test]
    fn test_input_queue_international_input() {
        use super::super::Adapter;
        let mut kbd = Keyboard::new();

        for &(umsg, wparam, lparam) in event_samples::PRESS_RELEASE_INTERNATIONAL_UMLAUT {
            if let Some(evt) = Adapter::adapt(umsg, WPARAM(wparam), LPARAM(lparam)) {
                kbd.process_evt(evt);
            }
        }

        let input: String = kbd.drain_input().chars().collect();
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

        for &(umsg, wparam, lparam) in event_samples::EMOJI_INPUT_OK_HAND {
            if let Some(evt) = Adapter::adapt(umsg, WPARAM(wparam), LPARAM(lparam)) {
                println!("{evt:#?}");
                kbd.process_evt(evt);
            }
        }

        let input: String = kbd.drain_input().chars().collect();
        assert_eq!(input, "👌");
        for key_code in KeyCode::iter() {
            assert!(
                !kbd.is_key_pressed(key_code),
                "{key_code:?} key still pressed"
            );
        }
    }

    /// Pressing backspace without any input in the queue should accumulate
    /// pending delete backspace events that can be applied to previously
    /// drained characters. If backspace is pressed while the input queue has
    /// some input should result in pending input being removed.
    #[test]
    fn test_backspace_key() {
        use super::super::Adapter;
        let mut kbd = Keyboard::new();

        for &(umsg, wparam, lparam) in [
            event_samples::PRESS_RELEASE_BACKSPACE,
            event_samples::PRESS_RELEASE_BACKSPACE,
            event_samples::PRESS_RELEASE_A,
            event_samples::PRESS_RELEASE_B,
            event_samples::PRESS_RELEASE_BACKSPACE,
            event_samples::PRESS_RELEASE_C,
        ]
        .into_iter()
        .flatten()
        {
            if let Some(evt) = Adapter::adapt(umsg, WPARAM(wparam), LPARAM(lparam)) {
                kbd.process_evt(evt);
            }
        }

        let mut state = kbd.drain_input();
        assert_eq!(state.num_backspaces(), 2);
        let input: String = state.chars().collect();
        assert_eq!(input, "ac");
    }
    // TODO: delete key
}
