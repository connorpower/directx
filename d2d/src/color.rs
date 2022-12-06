use ::windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;

/// Color representation in RGBA format.
///
/// # Conversion
///
/// Can be converted to/from DirectX `D3DCOLORVALUE` types or Direct2D
/// [`D2D1_COLOR_F`] or [`D2D_COLOR_F`] types.
///
/// [`D2D_COLOR_F`]: ::windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F
///
/// # Microsoft UI Colors
///
/// [`Color`] includes static definitions for all system colors in the Microsoft
/// UI core library.
///
/// <https://learn.microsoft.com/en-us/uwp/api/windows.ui.colors?view=winrt-22621>
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

impl From<Color> for D2D1_COLOR_F {
    fn from(c: Color) -> Self {
        // SAFETY: `D2D1_COLOR_F` and `Color` share the same memory
        // representation.
        unsafe { ::std::mem::transmute::<_, _>(c) }
    }
}

impl Color {
    /// Construct a new color from byte color values (0 - 255).
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            red: r as f32 / 255.0,
            green: g as f32 / 255.0,
            blue: b as f32 / 255.0,
            alpha: a as f32 / 255.0,
        }
    }

    /// Construct a new color from a hex value (0xRRGGBBAA).
    pub fn new_rgba(val: u32) -> Self {
        Self {
            red: ((val >> 24) & 0xFF) as f32 / 255.0,
            green: ((val >> 16) & 0xFF) as f32 / 255.0,
            blue: ((val >> 8) & 0xFF) as f32 / 255.0,
            alpha: (val & 0xFF) as f32 / 255.0,
        }
    }

    /// Construct a new color from a hex value (0xAARRGGBB). ARGBA is the format
    /// commonly used by Win UI frameworks (but not by DirectX).
    pub fn new_argb(val: u32) -> Self {
        Self {
            alpha: ((val >> 24) & 0xFF) as f32 / 255.0,
            red: ((val >> 16) & 0xFF) as f32 / 255.0,
            green: ((val >> 8) & 0xFF) as f32 / 255.0,
            blue: (val & 0xFF) as f32 / 255.0,
        }
    }

    /// Construct a new color from normalized float color values (0.0 - 1.0).
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

    /// AliceBlue predefined color from the Microsoft UI core library.
    pub fn alice_blue() -> Color {
        Color::new_argb(0xFFF0F8FF)
    }
    /// AntiqueWhite predefined color from the Microsoft UI core library.
    pub fn antique_white() -> Color {
        Color::new_argb(0xFFFAEBD7)
    }
    /// Aqua predefined color from the Microsoft UI core library.
    pub fn aqua() -> Color {
        Color::new_argb(0xFF00FFFF)
    }
    /// Aquamarine predefined color from the Microsoft UI core library.
    pub fn aquamarine() -> Color {
        Color::new_argb(0xFF7FFFD4)
    }
    /// Azure predefined color from the Microsoft UI core library.
    pub fn azure() -> Color {
        Color::new_argb(0xFFF0FFFF)
    }
    /// Beige predefined color from the Microsoft UI core library.
    pub fn beige() -> Color {
        Color::new_argb(0xFFF5F5DC)
    }
    /// Bisque predefined color from the Microsoft UI core library.
    pub fn bisque() -> Color {
        Color::new_argb(0xFFFFE4C4)
    }
    /// Black predefined color from the Microsoft UI core library.
    pub fn black() -> Color {
        Color::new_argb(0xFF000000)
    }
    /// BlanchedAlmond predefined color from the Microsoft UI core library.
    pub fn blanched_almond() -> Color {
        Color::new_argb(0xFFFFEBCD)
    }
    /// Blue predefined color from the Microsoft UI core library.
    pub fn blue() -> Color {
        Color::new_argb(0xFF0000FF)
    }
    /// BlueViolet predefined color from the Microsoft UI core library.
    pub fn blue_violet() -> Color {
        Color::new_argb(0xFF8A2BE2)
    }
    /// Brown predefined color from the Microsoft UI core library.
    pub fn brown() -> Color {
        Color::new_argb(0xFFA52A2A)
    }
    /// BurlyWood predefined color from the Microsoft UI core library.
    pub fn burly_wood() -> Color {
        Color::new_argb(0xFFDEB887)
    }
    /// CadetBlue predefined color from the Microsoft UI core library.
    pub fn cadet_blue() -> Color {
        Color::new_argb(0xFF5F9EA0)
    }
    /// Chartreuse predefined color from the Microsoft UI core library.
    pub fn chartreuse() -> Color {
        Color::new_argb(0xFF7FFF00)
    }
    /// Chocolate predefined color from the Microsoft UI core library.
    pub fn chocolate() -> Color {
        Color::new_argb(0xFFD2691E)
    }
    /// Coral predefined color from the Microsoft UI core library.
    pub fn coral() -> Color {
        Color::new_argb(0xFFFF7F50)
    }
    /// CornflowerBlue predefined color from the Microsoft UI core library.
    pub fn cornflower_blue() -> Color {
        Color::new_argb(0xFF6495ED)
    }
    /// Cornsilk predefined color from the Microsoft UI core library.
    pub fn cornsilk() -> Color {
        Color::new_argb(0xFFFFF8DC)
    }
    /// Crimson predefined color from the Microsoft UI core library.
    pub fn crimson() -> Color {
        Color::new_argb(0xFFDC143C)
    }
    /// Cyan predefined color from the Microsoft UI core library.
    pub fn cyan() -> Color {
        Color::new_argb(0xFF00FFFF)
    }
    /// DarkBlue predefined color from the Microsoft UI core library.
    pub fn dark_blue() -> Color {
        Color::new_argb(0xFF00008B)
    }
    /// DarkCyan predefined color from the Microsoft UI core library.
    pub fn dark_cyan() -> Color {
        Color::new_argb(0xFF008B8B)
    }
    /// DarkGoldenrod predefined color from the Microsoft UI core library.
    pub fn dark_goldenrod() -> Color {
        Color::new_argb(0xFFB8860B)
    }
    /// DarkGray predefined color from the Microsoft UI core library.
    pub fn dark_gray() -> Color {
        Color::new_argb(0xFFA9A9A9)
    }
    /// DarkGreen predefined color from the Microsoft UI core library.
    pub fn dark_green() -> Color {
        Color::new_argb(0xFF006400)
    }
    /// DarkKhaki predefined color from the Microsoft UI core library.
    pub fn dark_khaki() -> Color {
        Color::new_argb(0xFFBDB76B)
    }
    /// DarkMagenta predefined color from the Microsoft UI core library.
    pub fn dark_magenta() -> Color {
        Color::new_argb(0xFF8B008B)
    }
    /// dark_olive_green predefined color from the Microsoft UI core library.
    pub fn dark_olive_green() -> Color {
        Color::new_argb(0xFF556B2F)
    }
    /// DarkOrange predefined color from the Microsoft UI core library.
    pub fn dark_orange() -> Color {
        Color::new_argb(0xFFFF8C00)
    }
    /// DarkOrchid predefined color from the Microsoft UI core library.
    pub fn dark_orchid() -> Color {
        Color::new_argb(0xFF9932CC)
    }
    /// DarkRed predefined color from the Microsoft UI core library.
    pub fn dark_red() -> Color {
        Color::new_argb(0xFF8B0000)
    }
    /// DarkSalmon predefined color from the Microsoft UI core library.
    pub fn dark_salmon() -> Color {
        Color::new_argb(0xFFE9967A)
    }
    /// dark_sea_green predefined color from the Microsoft UI core library.
    pub fn dark_sea_green() -> Color {
        Color::new_argb(0xFF8FBC8F)
    }
    /// dark_slate_blue predefined color from the Microsoft UI core library.
    pub fn dark_slate_blue() -> Color {
        Color::new_argb(0xFF483D8B)
    }
    /// dark_slate_gray predefined color from the Microsoft UI core library.
    pub fn dark_slate_gray() -> Color {
        Color::new_argb(0xFF2F4F4F)
    }
    /// DarkTurquoise predefined color from the Microsoft UI core library.
    pub fn dark_turquoise() -> Color {
        Color::new_argb(0xFF00CED1)
    }
    /// DarkViolet predefined color from the Microsoft UI core library.
    pub fn dark_violet() -> Color {
        Color::new_argb(0xFF9400D3)
    }
    /// DeepPink predefined color from the Microsoft UI core library.
    pub fn deep_pink() -> Color {
        Color::new_argb(0xFFFF1493)
    }
    /// deep_sky_blue predefined color from the Microsoft UI core library.
    pub fn deep_sky_blue() -> Color {
        Color::new_argb(0xFF00BFFF)
    }
    /// DimGray predefined color from the Microsoft UI core library.
    pub fn dim_gray() -> Color {
        Color::new_argb(0xFF696969)
    }
    /// DodgerBlue predefined color from the Microsoft UI core library.
    pub fn dodger_blue() -> Color {
        Color::new_argb(0xFF1E90FF)
    }
    /// Firebrick predefined color from the Microsoft UI core library.
    pub fn firebrick() -> Color {
        Color::new_argb(0xFFB22222)
    }
    /// FloralWhite predefined color from the Microsoft UI core library.
    pub fn floral_white() -> Color {
        Color::new_argb(0xFFFFFAF0)
    }
    /// ForestGreen predefined color from the Microsoft UI core library.
    pub fn forest_green() -> Color {
        Color::new_argb(0xFF228B22)
    }
    /// Fuchsia predefined color from the Microsoft UI core library.
    pub fn fuchsia() -> Color {
        Color::new_argb(0xFFFF00FF)
    }
    /// Gainsboro predefined color from the Microsoft UI core library.
    pub fn gainsboro() -> Color {
        Color::new_argb(0xFFDCDCDC)
    }
    /// GhostWhite predefined color from the Microsoft UI core library.
    pub fn ghost_white() -> Color {
        Color::new_argb(0xFFF8F8FF)
    }
    /// Gold predefined color from the Microsoft UI core library.
    pub fn gold() -> Color {
        Color::new_argb(0xFFFFD700)
    }
    /// Goldenrod predefined color from the Microsoft UI core library.
    pub fn goldenrod() -> Color {
        Color::new_argb(0xFFDAA520)
    }
    /// Gray predefined color from the Microsoft UI core library.
    pub fn gray() -> Color {
        Color::new_argb(0xFF808080)
    }
    /// Green predefined color from the Microsoft UI core library.
    pub fn green() -> Color {
        Color::new_argb(0xFF008000)
    }
    /// GreenYellow predefined color from the Microsoft UI core library.
    pub fn green_yellow() -> Color {
        Color::new_argb(0xFFADFF2F)
    }
    /// Honeydew predefined color from the Microsoft UI core library.
    pub fn honeydew() -> Color {
        Color::new_argb(0xFFF0FFF0)
    }
    /// HotPink predefined color from the Microsoft UI core library.
    pub fn hot_pink() -> Color {
        Color::new_argb(0xFFFF69B4)
    }
    /// IndianRed predefined color from the Microsoft UI core library.
    pub fn indian_red() -> Color {
        Color::new_argb(0xFFCD5C5C)
    }
    /// Indigo predefined color from the Microsoft UI core library.
    pub fn indigo() -> Color {
        Color::new_argb(0xFF4B0082)
    }
    /// Ivory predefined color from the Microsoft UI core library.
    pub fn ivory() -> Color {
        Color::new_argb(0xFFFFFFF0)
    }
    /// Khaki predefined color from the Microsoft UI core library.
    pub fn khaki() -> Color {
        Color::new_argb(0xFFF0E68C)
    }
    /// Lavender predefined color from the Microsoft UI core library.
    pub fn lavender() -> Color {
        Color::new_argb(0xFFE6E6FA)
    }
    /// LavenderBlush predefined color from the Microsoft UI core library.
    pub fn lavender_blush() -> Color {
        Color::new_argb(0xFFFFF0F5)
    }
    /// LawnGreen predefined color from the Microsoft UI core library.
    pub fn lawn_green() -> Color {
        Color::new_argb(0xFF7CFC00)
    }
    /// LemonChiffon predefined color from the Microsoft UI core library.
    pub fn lemon_chiffon() -> Color {
        Color::new_argb(0xFFFFFACD)
    }
    /// LightBlue predefined color from the Microsoft UI core library.
    pub fn light_blue() -> Color {
        Color::new_argb(0xFFADD8E6)
    }
    /// LightCoral predefined color from the Microsoft UI core library.
    pub fn light_coral() -> Color {
        Color::new_argb(0xFFF08080)
    }
    /// LightCyan predefined color from the Microsoft UI core library.
    pub fn light_cyan() -> Color {
        Color::new_argb(0xFFE0FFFF)
    }
    /// light_goldenrod_yellow predefined color from the Microsoft UI core
    /// library.
    pub fn light_goldenrod_yellow() -> Color {
        Color::new_argb(0xFFFAFAD2)
    }
    /// LightGray predefined color from the Microsoft UI core library.
    pub fn light_gray() -> Color {
        Color::new_argb(0xFFD3D3D3)
    }
    /// LightGreen predefined color from the Microsoft UI core library.
    pub fn light_green() -> Color {
        Color::new_argb(0xFF90EE90)
    }
    /// LightPink predefined color from the Microsoft UI core library.
    pub fn light_pink() -> Color {
        Color::new_argb(0xFFFFB6C1)
    }
    /// LightSalmon predefined color from the Microsoft UI core library.
    pub fn light_salmon() -> Color {
        Color::new_argb(0xFFFFA07A)
    }
    /// light_sea_green predefined color from the Microsoft UI core library.
    pub fn light_sea_green() -> Color {
        Color::new_argb(0xFF20B2AA)
    }
    /// light_sky_blue predefined color from the Microsoft UI core library.
    pub fn light_sky_blue() -> Color {
        Color::new_argb(0xFF87CEFA)
    }
    /// light_slate_gray predefined color from the Microsoft UI core library.
    pub fn light_slate_gray() -> Color {
        Color::new_argb(0xFF778899)
    }
    /// light_steel_blue predefined color from the Microsoft UI core library.
    pub fn light_steel_blue() -> Color {
        Color::new_argb(0xFFB0C4DE)
    }
    /// LightYellow predefined color from the Microsoft UI core library.
    pub fn light_yellow() -> Color {
        Color::new_argb(0xFFFFFFE0)
    }
    /// Lime predefined color from the Microsoft UI core library.
    pub fn lime() -> Color {
        Color::new_argb(0xFF00FF00)
    }
    /// LimeGreen predefined color from the Microsoft UI core library.
    pub fn lime_green() -> Color {
        Color::new_argb(0xFF32CD32)
    }
    /// Linen predefined color from the Microsoft UI core library.
    pub fn linen() -> Color {
        Color::new_argb(0xFFFAF0E6)
    }
    /// Magenta predefined color from the Microsoft UI core library.
    pub fn magenta() -> Color {
        Color::new_argb(0xFFFF00FF)
    }
    /// Maroon predefined color from the Microsoft UI core library.
    pub fn maroon() -> Color {
        Color::new_argb(0xFF800000)
    }
    /// MediumAquamarine predefined color from the Microsoft UI core library.
    pub fn medium_aquamarine() -> Color {
        Color::new_argb(0xFF66CDAA)
    }
    /// MediumBlue predefined color from the Microsoft UI core library.
    pub fn medium_blue() -> Color {
        Color::new_argb(0xFF0000CD)
    }
    /// MediumOrchid predefined color from the Microsoft UI core library.
    pub fn medium_orchid() -> Color {
        Color::new_argb(0xFFBA55D3)
    }
    /// MediumPurple predefined color from the Microsoft UI core library.
    pub fn medium_purple() -> Color {
        Color::new_argb(0xFF9370DB)
    }
    /// medium_sea_green predefined color from the Microsoft UI core library.
    pub fn medium_sea_green() -> Color {
        Color::new_argb(0xFF3CB371)
    }
    /// medium_slate_blue predefined color from the Microsoft UI core library.
    pub fn medium_slate_blue() -> Color {
        Color::new_argb(0xFF7B68EE)
    }
    /// medium_spring_green predefined color from the Microsoft UI core library.
    pub fn medium_spring_green() -> Color {
        Color::new_argb(0xFF00FA9A)
    }
    /// MediumTurquoise predefined color from the Microsoft UI core library.
    pub fn medium_turquoise() -> Color {
        Color::new_argb(0xFF48D1CC)
    }
    /// medium_violet_red predefined color from the Microsoft UI core library.
    pub fn medium_violet_red() -> Color {
        Color::new_argb(0xFFC71585)
    }
    /// MidnightBlue predefined color from the Microsoft UI core library.
    pub fn midnight_blue() -> Color {
        Color::new_argb(0xFF191970)
    }
    /// MintCream predefined color from the Microsoft UI core library.
    pub fn mint_cream() -> Color {
        Color::new_argb(0xFFF5FFFA)
    }
    /// MistyRose predefined color from the Microsoft UI core library.
    pub fn misty_rose() -> Color {
        Color::new_argb(0xFFFFE4E1)
    }
    /// Moccasin predefined color from the Microsoft UI core library.
    pub fn moccasin() -> Color {
        Color::new_argb(0xFFFFE4B5)
    }
    /// NavajoWhite predefined color from the Microsoft UI core library.
    pub fn navajo_white() -> Color {
        Color::new_argb(0xFFFFDEAD)
    }
    /// Navy predefined color from the Microsoft UI core library.
    pub fn navy() -> Color {
        Color::new_argb(0xFF000080)
    }
    /// OldLace predefined color from the Microsoft UI core library.
    pub fn old_lace() -> Color {
        Color::new_argb(0xFFFDF5E6)
    }
    /// Olive predefined color from the Microsoft UI core library.
    pub fn olive() -> Color {
        Color::new_argb(0xFF808000)
    }
    /// OliveDrab predefined color from the Microsoft UI core library.
    pub fn olive_drab() -> Color {
        Color::new_argb(0xFF6B8E23)
    }
    /// Orange predefined color from the Microsoft UI core library.
    pub fn orange() -> Color {
        Color::new_argb(0xFFFFA500)
    }
    /// OrangeRed predefined color from the Microsoft UI core library.
    pub fn orange_red() -> Color {
        Color::new_argb(0xFFFF4500)
    }
    /// Orchid predefined color from the Microsoft UI core library.
    pub fn orchid() -> Color {
        Color::new_argb(0xFFDA70D6)
    }
    /// PaleGoldenrod predefined color from the Microsoft UI core library.
    pub fn pale_goldenrod() -> Color {
        Color::new_argb(0xFFEEE8AA)
    }
    /// PaleGreen predefined color from the Microsoft UI core library.
    pub fn pale_green() -> Color {
        Color::new_argb(0xFF98FB98)
    }
    /// PaleTurquoise predefined color from the Microsoft UI core library.
    pub fn pale_turquoise() -> Color {
        Color::new_argb(0xFFAFEEEE)
    }
    /// pale_violet_red predefined color from the Microsoft UI core library.
    pub fn pale_violet_red() -> Color {
        Color::new_argb(0xFFDB7093)
    }
    /// PapayaWhip predefined color from the Microsoft UI core library.
    pub fn papaya_whip() -> Color {
        Color::new_argb(0xFFFFEFD5)
    }
    /// PeachPuff predefined color from the Microsoft UI core library.
    pub fn peach_puff() -> Color {
        Color::new_argb(0xFFFFDAB9)
    }
    /// Peru predefined color from the Microsoft UI core library.
    pub fn peru() -> Color {
        Color::new_argb(0xFFCD853F)
    }
    /// Pink predefined color from the Microsoft UI core library.
    pub fn pink() -> Color {
        Color::new_argb(0xFFFFC0CB)
    }
    /// Plum predefined color from the Microsoft UI core library.
    pub fn plum() -> Color {
        Color::new_argb(0xFFDDA0DD)
    }
    /// PowderBlue predefined color from the Microsoft UI core library.
    pub fn powder_blue() -> Color {
        Color::new_argb(0xFFB0E0E6)
    }
    /// Purple predefined color from the Microsoft UI core library.
    pub fn purple() -> Color {
        Color::new_argb(0xFF800080)
    }
    /// Red predefined color from the Microsoft UI core library.
    pub fn red() -> Color {
        Color::new_argb(0xFFFF0000)
    }
    /// RosyBrown predefined color from the Microsoft UI core library.
    pub fn rosy_brown() -> Color {
        Color::new_argb(0xFFBC8F8F)
    }
    /// RoyalBlue predefined color from the Microsoft UI core library.
    pub fn royal_blue() -> Color {
        Color::new_argb(0xFF4169E1)
    }
    /// SaddleBrown predefined color from the Microsoft UI core library.
    pub fn saddle_brown() -> Color {
        Color::new_argb(0xFF8B4513)
    }
    /// Salmon predefined color from the Microsoft UI core library.
    pub fn salmon() -> Color {
        Color::new_argb(0xFFFA8072)
    }
    /// SandyBrown predefined color from the Microsoft UI core library.
    pub fn sandy_brown() -> Color {
        Color::new_argb(0xFFF4A460)
    }
    /// SeaGreen predefined color from the Microsoft UI core library.
    pub fn sea_green() -> Color {
        Color::new_argb(0xFF2E8B57)
    }
    /// SeaShell predefined color from the Microsoft UI core library.
    pub fn sea_shell() -> Color {
        Color::new_argb(0xFFFFF5EE)
    }
    /// Sienna predefined color from the Microsoft UI core library.
    pub fn sienna() -> Color {
        Color::new_argb(0xFFA0522D)
    }
    /// Silver predefined color from the Microsoft UI core library.
    pub fn silver() -> Color {
        Color::new_argb(0xFFC0C0C0)
    }
    /// SkyBlue predefined color from the Microsoft UI core library.
    pub fn sky_blue() -> Color {
        Color::new_argb(0xFF87CEEB)
    }
    /// SlateBlue predefined color from the Microsoft UI core library.
    pub fn slate_blue() -> Color {
        Color::new_argb(0xFF6A5ACD)
    }
    /// SlateGray predefined color from the Microsoft UI core library.
    pub fn slate_gray() -> Color {
        Color::new_argb(0xFF708090)
    }
    /// Snow predefined color from the Microsoft UI core library.
    pub fn snow() -> Color {
        Color::new_argb(0xFFFFFAFA)
    }
    /// SpringGreen predefined color from the Microsoft UI core library.
    pub fn spring_green() -> Color {
        Color::new_argb(0xFF00FF7F)
    }
    /// SteelBlue predefined color from the Microsoft UI core library.
    pub fn steel_blue() -> Color {
        Color::new_argb(0xFF4682B4)
    }
    /// Tan predefined color from the Microsoft UI core library.
    pub fn tan() -> Color {
        Color::new_argb(0xFFD2B48C)
    }
    /// Teal predefined color from the Microsoft UI core library.
    pub fn teal() -> Color {
        Color::new_argb(0xFF008080)
    }
    /// Thistle predefined color from the Microsoft UI core library.
    pub fn thistle() -> Color {
        Color::new_argb(0xFFD8BFD8)
    }
    /// Tomato predefined color from the Microsoft UI core library.
    pub fn tomato() -> Color {
        Color::new_argb(0xFFFF6347)
    }
    /// Transparent predefined color from the Microsoft UI core library.
    pub fn transparent() -> Color {
        Color::new_argb(0x00FFFFFF)
    }
    /// Turquoise predefined color from the Microsoft UI core library.
    pub fn turquoise() -> Color {
        Color::new_argb(0xFF40E0D0)
    }
    /// Violet predefined color from the Microsoft UI core library.
    pub fn violet() -> Color {
        Color::new_argb(0xFFEE82EE)
    }
    /// Wheat predefined color from the Microsoft UI core library.
    pub fn wheat() -> Color {
        Color::new_argb(0xFFF5DEB3)
    }
    /// White predefined color from the Microsoft UI core library.
    pub fn white() -> Color {
        Color::new_argb(0xFFFFFFFF)
    }
    /// WhiteSmoke predefined color from the Microsoft UI core library.
    pub fn white_smoke() -> Color {
        Color::new_argb(0xFFF5F5F5)
    }
    /// Yellow predefined color from the Microsoft UI core library.
    pub fn yellow() -> Color {
        Color::new_argb(0xFFFFFF00)
    }
    /// YellowGreen predefined color from the Microsoft UI core library.
    pub fn yellow_green() -> Color {
        Color::new_argb(0xFF9ACD32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ::pretty_assertions::assert_eq;

    #[test]
    fn test_byte_color() {
        let color = Color::new(0x9A, 0xCD, 0x32, 0xFF);
        assert_eq!(color.red, 0x9A as f32 / 255.0);
        assert_eq!(color.green, 0xCD as f32 / 255.0);
        assert_eq!(color.blue, 0x32 as f32 / 255.0);
        assert_eq!(color.alpha, 0xFF as f32 / 255.0);

        assert_eq!(color, Color::yellow_green());
    }

    #[test]
    fn test_rgba_color() {
        let color = Color::new_rgba(0x9ACD32FF);

        assert_eq!(color.red, 0x9A as f32 / 255.0);
        assert_eq!(color.green, 0xCD as f32 / 255.0);
        assert_eq!(color.blue, 0x32 as f32 / 255.0);
        assert_eq!(color.alpha, 0xFF as f32 / 255.0);

        assert_eq!(color, Color::yellow_green());
    }

    #[test]
    fn test_argb_color() {
        let color = Color::new_argb(0xFF9ACD32);

        assert_eq!(color.red, 0x9A as f32 / 255.0);
        assert_eq!(color.green, 0xCD as f32 / 255.0);
        assert_eq!(color.blue, 0x32 as f32 / 255.0);
        assert_eq!(color.alpha, 0xFF as f32 / 255.0);

        assert_eq!(color, Color::yellow_green());
    }
}
