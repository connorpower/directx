//! Keyboard state and text input tracking.

use ::bitvec::prelude::*;
use ::std::{char::REPLACEMENT_CHARACTER, collections::VecDeque};
use ::tracing::trace;
use ::widestring::WideChar;

use super::KeyCode;

/// Length of the input queue, after which point the earliest characters are
/// dropped.
const INPUT_QUEUE_CAPACITY: usize = 32;

/// A representation of a Win32 virtual key event. These are purely internal and
/// are consumed by the `Keyboard` type.
pub(crate) enum KeyEvent {
    KeyDown {
        key_code: KeyCode,
        is_repeat_event: bool,
    },
    KeyUp {
        key_code: KeyCode,
    },
    Input {
        wchar: WideChar,
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
            KeyEvent::KeyDown {
                key_code,
                is_repeat_event,
            } => {
                if !is_repeat_event {
                    *self.mut_bit_for_key(key_code).as_mut() = true;
                }
            }
            KeyEvent::KeyUp { key_code } => {
                *self.mut_bit_for_key(key_code).as_mut() = false;
            }
            KeyEvent::Input { wchar } => {
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

    /// A basic smoke test for key pressed events.
    #[test]
    fn test_key_pressed_basic() {
        let mut kbd = Keyboard::new();

        assert!(kbd.is_key_pressed(KeyCode::Up).not());
        kbd.process_evt(KeyEvent::KeyDown {
            key_code: KeyCode::Up,
            is_repeat_event: false,
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
                is_repeat_event: false,
            },
            KeyEvent::KeyDown {
                key_code: KeyCode::Left,
                is_repeat_event: false,
            },
            KeyEvent::KeyDown {
                key_code: KeyCode::Space,
                is_repeat_event: false,
            },
            KeyEvent::KeyDown {
                key_code: KeyCode::Left,
                is_repeat_event: true,
            },
            KeyEvent::KeyUp {
                key_code: KeyCode::A,
            },
            KeyEvent::KeyDown {
                key_code: KeyCode::Left,
                is_repeat_event: true,
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
        for evt in "Hello, world!"
            .chars()
            .map(|c| KeyEvent::Input { wchar: c as _ })
        {
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
            .map(|wchar| KeyEvent::Input { wchar })
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

        kbd.process_evt(KeyEvent::Input { wchar: 0xD834 });
        assert!(
            kbd.drain_input_queue().next().is_none(),
            "Input queue should wait for following low surrogate before returning"
        );

        kbd.process_evt(KeyEvent::Input { wchar: 0xDD1E });

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
            .map(|wchar| KeyEvent::Input { wchar })
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
            .map(|c| KeyEvent::Input { wchar: *c as _ })
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
            .map(|c| KeyEvent::Input { wchar: c as _ })
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
            .map(|c| KeyEvent::Input { wchar: *c as _ })
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
}
