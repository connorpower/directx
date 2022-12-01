use ::windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;

/// Color representation in RGBA format.
///
/// # Conversion
///
/// Can be converted to/from DirectX `D3DCOLORVALUE` types or Direct2D
/// [`D2D1_COLOR_F`] or [`D2D_COLOR_F`] types.
///
/// [`D2D_COLOR_F`]: ::windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Color {
    /// Floating-point value that specifies the red component of a color. This
    /// value generally is in the range from 0.0 through 1.0. A value of 0.0
    /// indicates the complete absence of the red component, while a value of
    /// 1.0 indicates that red is fully present.
    pub red: f32,

    /// Floating-point value that specifies the green component of a color. This
    /// value generally is in the range from 0.0 through 1.0. A value of 0.0
    /// indicates the complete absence of the green component, while a value of
    /// 1.0 indicates that green is fully present.
    pub green: f32,

    /// Floating-point value that specifies the blue component of a color. This
    /// value generally is in the range from 0.0 through 1.0. A value of 0.0
    /// indicates the complete absence of the blue component, while a value of
    /// 1.0 indicates that blue is fully present.
    pub blue: f32,

    /// Floating-point value that specifies the alpha component of a color. This
    /// value generally is in the range from 0.0 through 1.0. A value of 0.0
    /// indicates fully transparent, while a value of 1.0 indicates fully
    /// opaque.
    pub alpha: f32,
}

impl Color {
    /// Predefined color for pure red.
    pub const fn red() -> Self {
        Self {
            red: 1.0,
            green: 0.0,
            blue: 0.0,
            alpha: 1.0,
        }
    }

    /// Predefined color for pure green.
    pub const fn green() -> Self {
        Self {
            red: 0.0,
            green: 1.0,
            blue: 0.0,
            alpha: 1.0,
        }
    }

    /// Predefined color for pure blue.
    pub const fn blue() -> Self {
        Self {
            red: 0.0,
            green: 0.0,
            blue: 1.0,
            alpha: 1.0,
        }
    }
}

impl Color {
    /// Construct a new color from normalized color values.
    pub fn new_normalized(r: f32, g: f32, b: f32, a: f32) -> Self {
        debug_assert!(r >= 0.0, "Negative red value in color");
        debug_assert!(g >= 0.0, "Negative green value in color");
        debug_assert!(b >= 0.0, "Negative blue value in color");
        debug_assert!(a >= 0.0, "Negative alpha value in color");
        debug_assert!(r <= 1.0, "Red value in color greater than 1.0");
        debug_assert!(g <= 1.0, "Green value in color greater than 1.0");
        debug_assert!(b <= 1.0, "Blue value in color greater than 1.0");
        debug_assert!(a <= 1.0, "Alpha value in color greater than 1.0");

        Self {
            red: r,
            green: g,
            blue: b,
            alpha: a,
        }
    }
}

impl From<Color> for D2D1_COLOR_F {
    fn from(c: Color) -> Self {
        // SAFETY: `D2D1_COLOR_F` and `Color` share the same memory
        // representation.
        unsafe { ::std::mem::transmute::<_, _>(c) }
    }
}
