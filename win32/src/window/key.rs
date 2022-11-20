/// Key codes for key-pressed and key-released events. These are not the same as
/// the unicode characters which result from keyboard entry, and should
/// therefore not be used for text input but instead for simple key
/// pressed/released tracking (useful for instance if using the keyboard to
/// control a game).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, ::strum::EnumIter)]
pub enum KeyCode {
    /// Left mouse button
    LeftMouseButton = 0x01,
    /// Right mouse button
    RightMouseButton = 0x02,
    /// Control-break processing
    Cancel = 0x03,
    /// Middle mouse button (three-button mouse)
    MiddleMouseButton = 0x04,
    /// X1 mouse button
    X1MouseButton = 0x05,
    /// X2 mouse button
    X2MouseButton = 0x06,
    /// BACKSPACE key
    Back = 0x08,
    /// TAB key
    Tab = 0x09,
    /// CLEAR key
    Clear = 0x0C,
    /// ENTER key
    Return = 0x0D,
    /// SHIFT key
    Shift = 0x10,
    /// CTRL key
    Control = 0x11,
    /// ALT key
    Menu = 0x12,
    /// PAUSE key
    Pause = 0x13,
    /// CAPS LOCK key
    Capital = 0x14,
    /// IME Kana or Hangel mode
    IMEKanaHangel = 0x15,
    /// IME Junja mode
    IMEJunja = 0x17,
    /// IME final mode
    IMEFinal = 0x18,
    /// IME Hanja or Kanji mode
    IMEHanja = 0x19,
    /// ESC key
    Escape = 0x1B,
    /// IME convert
    IMEConvert = 0x1C,
    /// IME nonconvert
    IMENonconvert = 0x1D,
    /// IME accept
    IMEAccept = 0x1E,
    /// IME mode change request
    IMEModechange = 0x1F,
    /// SPACEBAR
    Space = 0x20,
    /// PAGE UP key
    PageUp = 0x21,
    /// PAGE DOWN key
    PageDown = 0x22,
    /// END key
    End = 0x23,
    /// HOME key
    Home = 0x24,
    /// LEFT ARROW key
    Left = 0x25,
    /// UP ARROW key
    Up = 0x26,
    /// RIGHT ARROW key
    Right = 0x27,
    /// DOWN ARROW key
    Down = 0x28,
    /// SELECT key
    Select = 0x29,
    /// PRINT key
    Print = 0x2A,
    /// EXECUTE key
    Execute = 0x2B,
    /// PRINT SCREEN key
    PrintScreen = 0x2C,
    /// INS key
    Insert = 0x2D,
    /// DEL key
    Delete = 0x2E,
    /// HELP key
    Help = 0x2F,
    /// 0 key
    Zero = 0x30,
    /// 1 key
    One = 0x31,
    /// 2 key
    Two = 0x32,
    /// 3 key
    Three = 0x33,
    /// 4 key
    Four = 0x34,
    /// 5 key
    Five = 0x35,
    /// 6 key
    Six = 0x36,
    /// 7 key
    Seven = 0x37,
    /// 8 key
    Eight = 0x38,
    /// 9 key
    Nine = 0x39,
    /// A key
    A = 0x41,
    /// B key
    B = 0x42,
    /// C key
    C = 0x43,
    /// D key
    D = 0x44,
    /// E key
    E = 0x45,
    /// F key
    F = 0x46,
    /// G key
    G = 0x47,
    /// H key
    H = 0x48,
    /// I key
    I = 0x49,
    /// J key
    J = 0x4A,
    /// K key
    K = 0x4B,
    /// L key
    L = 0x4C,
    /// M key
    M = 0x4D,
    /// N key
    N = 0x4E,
    /// O key
    O = 0x4F,
    /// P key
    P = 0x50,
    /// Q key
    Q = 0x51,
    /// R key
    R = 0x52,
    /// S key
    S = 0x53,
    /// T key
    T = 0x54,
    /// U key
    U = 0x55,
    /// V key
    V = 0x56,
    /// W key
    W = 0x57,
    /// X key
    X = 0x58,
    /// Y key
    Y = 0x59,
    /// Z key
    Z = 0x5A,
    /// Left Windows key (Natural keyboard)
    LeftWindows = 0x5B,
    /// Right Windows key (Natural keyboard)
    RightWindows = 0x5C,
    /// Applications key (Natural keyboard)
    Apps = 0x5D,
    /// Computer Sleep key
    Sleep = 0x5F,
    /// Numeric keypad 0 key
    Numpad0 = 0x60,
    /// Numeric keypad 1 key
    Numpad1 = 0x61,
    /// Numeric keypad 2 key
    Numpad2 = 0x62,
    /// Numeric keypad 3 key
    Numpad3 = 0x63,
    /// Numeric keypad 4 key
    Numpad4 = 0x64,
    /// Numeric keypad 5 key
    Numpad5 = 0x65,
    /// Numeric keypad 6 key
    Numpad6 = 0x66,
    /// Numeric keypad 7 key
    Numpad7 = 0x67,
    /// Numeric keypad 8 key
    Numpad8 = 0x68,
    /// Numeric keypad 9 key
    Numpad9 = 0x69,
    /// Multiply key
    Multiply = 0x6A,
    /// Add key
    Add = 0x6B,
    /// Separator key
    Separator = 0x6C,
    /// Subtract key
    Subtract = 0x6D,
    /// Decimal key
    Decimal = 0x6E,
    /// Divide key
    Divide = 0x6F,
    /// F1 key
    F1 = 0x70,
    /// F2 key
    F2 = 0x71,
    /// F3 key
    F3 = 0x72,
    /// F4 key
    F4 = 0x73,
    /// F5 key
    F5 = 0x74,
    /// F6 key
    F6 = 0x75,
    /// F7 key
    F7 = 0x76,
    /// F8 key
    F8 = 0x77,
    /// F9 key
    F9 = 0x78,
    /// F10 key
    F10 = 0x79,
    /// F11 key
    F11 = 0x7A,
    /// F12 key
    F12 = 0x7B,
    /// F13 key
    F13 = 0x7C,
    /// F14 key
    F14 = 0x7D,
    /// F15 key
    F15 = 0x7E,
    /// F16 key
    F16 = 0x7F,
    /// F17 key
    F17 = 0x80,
    /// F18 key
    F18 = 0x81,
    /// F19 key
    F19 = 0x82,
    /// F20 key
    F20 = 0x83,
    /// F21 key
    F21 = 0x84,
    /// F22 key
    F22 = 0x85,
    /// F23 key
    F23 = 0x86,
    /// F24 key
    F24 = 0x87,
    /// NUM LOCK key
    Numlock = 0x90,
    /// SCROLL LOCK key
    ScrollLock = 0x91,
    /// Left SHIFT key
    LeftShift = 0xA0,
    /// Right SHIFT key
    RightShift = 0xA1,
    /// Left CONTROL key
    LeftControl = 0xA2,
    /// Right CONTROL key
    RightControl = 0xA3,
    /// Left MENU key
    LeftAlt = 0xA4,
    /// Right MENU key
    RightAlt = 0xA5,
    /// Browser Back key
    BrowserBack = 0xA6,
    /// Browser Forward key
    BrowserForward = 0xA7,
    /// Browser Refresh key
    BrowserRefresh = 0xA8,
    /// Browser Stop key
    BrowserStop = 0xA9,
    /// Browser Search key
    BrowserSearch = 0xAA,
    /// Browser Favorites key
    BrowserFavorites = 0xAB,
    /// Browser Start and Home key
    BrowserHome = 0xAC,
    /// Volume Mute key
    VolumeMute = 0xAD,
    /// Volume Down key
    VolumeDown = 0xAE,
    /// Volume Up key
    VolumeUp = 0xAF,
    /// Next Track key
    MediaNextTrack = 0xB0,
    /// Previous Track key
    MediaPrevTrack = 0xB1,
    /// Stop Media key
    MediaStop = 0xB2,
    /// Play/Pause Media key
    MediaPlayPause = 0xB3,
    /// Start Mail key
    LaunchMail = 0xB4,
    /// Select Media key
    LaunchMediaSelect = 0xB5,
    /// Start Application 1 key
    LaunchApp1 = 0xB6,
    /// Start Application 2 key
    LaunchApp2 = 0xB7,
    /// Used for miscellaneous characters; it can vary by keyboard.
    OEM1 = 0xBA,
    /// For any country/region, the '+' key
    OEMPlus = 0xBB,
    /// For any country/region, the ',' key
    OEMComma = 0xBC,
    /// For any country/region, the '-' key
    OEMMinus = 0xBD,
    /// For any country/region, the '.' key
    OEMPeriod = 0xBE,
    /// Used for miscellaneous characters; it can vary by keyboard.
    OEM2 = 0xBF,
    /// Used for miscellaneous characters; it can vary by keyboard.
    OEM3 = 0xC0,
    /// Used for miscellaneous characters; it can vary by keyboard.
    OEM4 = 0xDB,
    /// Used for miscellaneous characters; it can vary by keyboard.
    OEM5 = 0xDC,
    /// Used for miscellaneous characters; it can vary by keyboard.
    OEM6 = 0xDD,
    /// Used for miscellaneous characters; it can vary by keyboard.
    OEM7 = 0xDE,
    /// Used for miscellaneous characters; it can vary by keyboard.
    OEM8 = 0xDF,
    /// Either the angle bracket key or the backslash key on the RT 102-key
    /// keyboard
    OEM102 = 0xE2,
    /// IME PROCESS key
    Processkey = 0xE5,
    /// Used to pass Unicode characters as if they were keystrokes.
    Packet = 0xE7,
    /// Attn key
    Attn = 0xF6,
    /// CrSel key
    CrSel = 0xF7,
    /// ExSel key
    ExSel = 0xF8,
    /// Erase EOF key
    EraseEOF = 0xF9,
    /// Play key
    Play = 0xFA,
    /// Zoom key
    Zoom = 0xFB,
    /// PA1 key
    Pa1 = 0xFD,
    /// Clear key
    OEMClear = 0xFE,
}

impl KeyCode {
    pub const fn value(&self) -> u8 {
        *self as u8
    }
}

impl TryFrom<u16> for KeyCode {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        assert!(
            value <= u8::MAX as u16,
            "Win32 virutal key code exceeded 256"
        );

        LookupTable[(value as u8) as usize].ok_or(())
    }
}

/// An internal lookup table wich maps between Win32 virtual key codes and our
/// strong `KeyCode` type.
const LookupTable: [Option<KeyCode>; 256] = [
    None,
    Some(KeyCode::LeftMouseButton),
    Some(KeyCode::RightMouseButton),
    Some(KeyCode::Cancel),
    Some(KeyCode::MiddleMouseButton),
    Some(KeyCode::X1MouseButton),
    Some(KeyCode::X2MouseButton),
    None,
    Some(KeyCode::Back),
    Some(KeyCode::Tab),
    None,
    None,
    Some(KeyCode::Clear),
    Some(KeyCode::Return),
    None,
    None,
    Some(KeyCode::Shift),
    Some(KeyCode::Control),
    Some(KeyCode::Menu),
    Some(KeyCode::Pause),
    Some(KeyCode::Capital),
    Some(KeyCode::IMEKanaHangel),
    None,
    Some(KeyCode::IMEJunja),
    Some(KeyCode::IMEFinal),
    Some(KeyCode::IMEHanja),
    None,
    Some(KeyCode::Escape),
    Some(KeyCode::IMEConvert),
    Some(KeyCode::IMENonconvert),
    Some(KeyCode::IMEAccept),
    Some(KeyCode::IMEModechange),
    Some(KeyCode::Space),
    Some(KeyCode::PageUp),
    Some(KeyCode::PageDown),
    Some(KeyCode::End),
    Some(KeyCode::Home),
    Some(KeyCode::Left),
    Some(KeyCode::Up),
    Some(KeyCode::Right),
    Some(KeyCode::Down),
    Some(KeyCode::Select),
    Some(KeyCode::Print),
    Some(KeyCode::Execute),
    Some(KeyCode::PrintScreen),
    Some(KeyCode::Insert),
    Some(KeyCode::Delete),
    Some(KeyCode::Help),
    Some(KeyCode::Zero),
    Some(KeyCode::One),
    Some(KeyCode::Two),
    Some(KeyCode::Three),
    Some(KeyCode::Four),
    Some(KeyCode::Five),
    Some(KeyCode::Six),
    Some(KeyCode::Seven),
    Some(KeyCode::Eight),
    Some(KeyCode::Nine),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(KeyCode::A),
    Some(KeyCode::B),
    Some(KeyCode::C),
    Some(KeyCode::D),
    Some(KeyCode::E),
    Some(KeyCode::F),
    Some(KeyCode::G),
    Some(KeyCode::H),
    Some(KeyCode::I),
    Some(KeyCode::J),
    Some(KeyCode::K),
    Some(KeyCode::L),
    Some(KeyCode::M),
    Some(KeyCode::N),
    Some(KeyCode::O),
    Some(KeyCode::P),
    Some(KeyCode::Q),
    Some(KeyCode::R),
    Some(KeyCode::S),
    Some(KeyCode::T),
    Some(KeyCode::U),
    Some(KeyCode::V),
    Some(KeyCode::W),
    Some(KeyCode::X),
    Some(KeyCode::Y),
    Some(KeyCode::Z),
    Some(KeyCode::LeftWindows),
    Some(KeyCode::RightWindows),
    Some(KeyCode::Apps),
    None,
    Some(KeyCode::Sleep),
    Some(KeyCode::Numpad0),
    Some(KeyCode::Numpad1),
    Some(KeyCode::Numpad2),
    Some(KeyCode::Numpad3),
    Some(KeyCode::Numpad4),
    Some(KeyCode::Numpad5),
    Some(KeyCode::Numpad6),
    Some(KeyCode::Numpad7),
    Some(KeyCode::Numpad8),
    Some(KeyCode::Numpad9),
    Some(KeyCode::Multiply),
    Some(KeyCode::Add),
    Some(KeyCode::Separator),
    Some(KeyCode::Subtract),
    Some(KeyCode::Decimal),
    Some(KeyCode::Divide),
    Some(KeyCode::F1),
    Some(KeyCode::F2),
    Some(KeyCode::F3),
    Some(KeyCode::F4),
    Some(KeyCode::F5),
    Some(KeyCode::F6),
    Some(KeyCode::F7),
    Some(KeyCode::F8),
    Some(KeyCode::F9),
    Some(KeyCode::F10),
    Some(KeyCode::F11),
    Some(KeyCode::F12),
    Some(KeyCode::F13),
    Some(KeyCode::F14),
    Some(KeyCode::F15),
    Some(KeyCode::F16),
    Some(KeyCode::F17),
    Some(KeyCode::F18),
    Some(KeyCode::F19),
    Some(KeyCode::F20),
    Some(KeyCode::F21),
    Some(KeyCode::F22),
    Some(KeyCode::F23),
    Some(KeyCode::F24),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(KeyCode::Numlock),
    Some(KeyCode::ScrollLock),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(KeyCode::LeftShift),
    Some(KeyCode::RightShift),
    Some(KeyCode::LeftControl),
    Some(KeyCode::RightControl),
    Some(KeyCode::LeftAlt),
    Some(KeyCode::RightAlt),
    Some(KeyCode::BrowserBack),
    Some(KeyCode::BrowserForward),
    Some(KeyCode::BrowserRefresh),
    Some(KeyCode::BrowserStop),
    Some(KeyCode::BrowserSearch),
    Some(KeyCode::BrowserFavorites),
    Some(KeyCode::BrowserHome),
    Some(KeyCode::VolumeMute),
    Some(KeyCode::VolumeDown),
    Some(KeyCode::VolumeUp),
    Some(KeyCode::MediaNextTrack),
    Some(KeyCode::MediaPrevTrack),
    Some(KeyCode::MediaStop),
    Some(KeyCode::MediaPlayPause),
    Some(KeyCode::LaunchMail),
    Some(KeyCode::LaunchMediaSelect),
    Some(KeyCode::LaunchApp1),
    Some(KeyCode::LaunchApp2),
    None,
    None,
    Some(KeyCode::OEM1),
    Some(KeyCode::OEMPlus),
    Some(KeyCode::OEMComma),
    Some(KeyCode::OEMMinus),
    Some(KeyCode::OEMPeriod),
    Some(KeyCode::OEM2),
    Some(KeyCode::OEM3),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(KeyCode::OEM4),
    Some(KeyCode::OEM5),
    Some(KeyCode::OEM6),
    Some(KeyCode::OEM7),
    Some(KeyCode::OEM8),
    None,
    None,
    Some(KeyCode::OEM102),
    None,
    None,
    Some(KeyCode::Processkey),
    None,
    Some(KeyCode::Packet),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(KeyCode::Attn),
    Some(KeyCode::CrSel),
    Some(KeyCode::ExSel),
    Some(KeyCode::EraseEOF),
    Some(KeyCode::Play),
    Some(KeyCode::Zoom),
    None,
    Some(KeyCode::Pa1),
    Some(KeyCode::OEMClear),
    None,
];
