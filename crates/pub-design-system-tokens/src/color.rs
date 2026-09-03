//! The `color` type of the Design Tokens Color Module 2025.10.

use std::fmt;

/// A colour space of the Color Module: the coordinate system `components` are read in.
///
/// The identifiers are the module's (`as_str`), which are CSS Color 4's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ColorSpace {
    /// `srgb`: red, green, blue in `[0, 1]`.
    Srgb,
    /// `srgb-linear`: red, green, blue in `[0, 1]`, linear light.
    SrgbLinear,
    /// `hsl`: hue in `[0, 360)`, saturation and lightness in `[0, 100]`.
    Hsl,
    /// `hwb`: hue in `[0, 360)`, whiteness and blackness in `[0, 100]`.
    Hwb,
    /// `lab`: CIE lightness in `[0, 100]`, `a` and `b` unbounded.
    Lab,
    /// `lch`: CIE lightness in `[0, 100]`, chroma `≥ 0`, hue in `[0, 360)`.
    Lch,
    /// `oklab`: lightness in `[0, 1]`, `a` and `b` unbounded.
    Oklab,
    /// `oklch`: lightness in `[0, 1]`, chroma `≥ 0`, hue in `[0, 360)`.
    Oklch,
    /// `display-p3`: red, green, blue in `[0, 1]`.
    DisplayP3,
    /// `a98-rgb`: red, green, blue in `[0, 1]`.
    A98Rgb,
    /// `prophoto-rgb`: red, green, blue in `[0, 1]`.
    ProphotoRgb,
    /// `rec2020`: red, green, blue in `[0, 1]`.
    Rec2020,
    /// `xyz-d65`: X, Y, Z in `[0, 1]`.
    XyzD65,
    /// `xyz-d50`: X, Y, Z in `[0, 1]`.
    XyzD50,
}

impl ColorSpace {
    /// Every colour space the module defines, in the module's order.
    pub const ALL: [ColorSpace; 14] = [
        Self::Srgb,
        Self::SrgbLinear,
        Self::Hsl,
        Self::Hwb,
        Self::Lab,
        Self::Lch,
        Self::Oklab,
        Self::Oklch,
        Self::DisplayP3,
        Self::A98Rgb,
        Self::ProphotoRgb,
        Self::Rec2020,
        Self::XyzD65,
        Self::XyzD50,
    ];

    /// The identifier the JSON carries in `colorSpace`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Srgb => "srgb",
            Self::SrgbLinear => "srgb-linear",
            Self::Hsl => "hsl",
            Self::Hwb => "hwb",
            Self::Lab => "lab",
            Self::Lch => "lch",
            Self::Oklab => "oklab",
            Self::Oklch => "oklch",
            Self::DisplayP3 => "display-p3",
            Self::A98Rgb => "a98-rgb",
            Self::ProphotoRgb => "prophoto-rgb",
            Self::Rec2020 => "rec2020",
            Self::XyzD65 => "xyz-d65",
            Self::XyzD50 => "xyz-d50",
        }
    }

    /// The colour space for a `colorSpace` identifier, `None` for anything else.
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|space| space.as_str() == s)
    }
}

impl fmt::Display for ColorSpace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One coordinate of a colour: a number, or the module's `none` keyword for a component
/// that is not applicable or not specified (a hue at zero chroma, say).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Component {
    /// A coordinate in the colour space's range.
    Value(f64),
    /// The `none` keyword.
    None,
}

/// A six-digit CSS hex colour, the module's sRGB fallback (`#rrggbb`).
///
/// It is stored as three bytes and always written in lower case, so `#FF0000` reads back
/// as `#ff0000`: equal as a colour, normalized as text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Hex(pub [u8; 3]);

impl Hex {
    /// Reads `#rrggbb` in either case; anything else (three digits, no `#`, a non-hex
    /// digit, an alpha byte) is `None`.
    pub fn parse(s: &str) -> Option<Self> {
        let digits = s.strip_prefix('#')?;
        if digits.len() != 6 || !digits.is_ascii() {
            return None;
        }
        let byte = |i: usize| u8::from_str_radix(&digits[i..i + 2], 16).ok();
        Some(Self([byte(0)?, byte(2)?, byte(4)?]))
    }
}

impl fmt::Display for Hex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [r, g, b] = self.0;
        write!(f, "#{r:02x}{g:02x}{b:02x}")
    }
}

/// A colour: the `color` type of the Color Module.
///
/// `alpha` is `None` when the JSON omits it, which the module reads as fully opaque; it is
/// kept as written so that a file round-trips unchanged. `hex` is the optional sRGB
/// fallback; nothing checks that it agrees with `components` (the module does not ask for
/// it), but [`Color::from_rgb8`] builds the two from the same bytes.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Color {
    /// The coordinate system of `components`.
    pub color_space: ColorSpace,
    /// The three coordinates, in the colour space's order.
    pub components: [Component; 3],
    /// Opacity in `[0, 1]`; `None` when not written (the module's default is `1`).
    pub alpha: Option<f64>,
    /// The six-digit sRGB fallback, when written.
    pub hex: Option<Hex>,
}

impl Color {
    /// A colour in `color_space` from its three coordinates, opaque, without a fallback.
    pub const fn new(color_space: ColorSpace, components: [f64; 3]) -> Self {
        Self {
            color_space,
            components: [
                Component::Value(components[0]),
                Component::Value(components[1]),
                Component::Value(components[2]),
            ],
            alpha: None,
            hex: None,
        }
    }

    /// An sRGB colour from its 8-bit channels, with the matching `hex` fallback: the
    /// components are `channel / 255`, so the two agree by construction.
    ///
    /// ```
    /// use pub_design_system_tokens::{Color, Component};
    /// let c = Color::from_rgb8(0x00, 0x66, 0xcc);
    /// assert_eq!(c.components[1], Component::Value(0.4));
    /// assert_eq!(c.hex.unwrap().to_string(), "#0066cc");
    /// ```
    pub const fn from_rgb8(r: u8, g: u8, b: u8) -> Self {
        Self {
            color_space: ColorSpace::Srgb,
            components: [
                Component::Value(r as f64 / 255.0),
                Component::Value(g as f64 / 255.0),
                Component::Value(b as f64 / 255.0),
            ],
            alpha: None,
            hex: Some(Hex([r, g, b])),
        }
    }

    /// The same colour with an explicit opacity in `[0, 1]`.
    pub const fn with_alpha(mut self, alpha: f64) -> Self {
        self.alpha = Some(alpha);
        self
    }

    /// The same colour with a `hex` fallback.
    pub const fn with_hex(mut self, hex: Hex) -> Self {
        self.hex = Some(hex);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_space_parses_back_from_its_identifier() {
        for space in ColorSpace::ALL {
            assert_eq!(ColorSpace::parse(space.as_str()), Some(space));
            assert_eq!(space.to_string(), space.as_str());
        }
        assert_eq!(ColorSpace::parse("SRGB"), None);
        assert_eq!(ColorSpace::parse(""), None);
    }

    #[test]
    fn hex_reads_six_digits_only() {
        assert_eq!(Hex::parse("#0066CC"), Some(Hex([0, 0x66, 0xcc])));
        assert_eq!(Hex::parse("#0066cc").unwrap().to_string(), "#0066cc");
        for bad in [
            "#fff",
            "0066cc",
            "#0066cc80",
            "#00 6cc",
            "#00é6cc",
            "#gg0000",
            "#",
        ] {
            assert_eq!(Hex::parse(bad), None, "{bad}");
        }
    }

    #[test]
    fn rgb8_components_are_channel_over_255() {
        let c = Color::from_rgb8(255, 0, 51);
        assert_eq!(
            c.components,
            [
                Component::Value(1.0),
                Component::Value(0.0),
                Component::Value(0.2)
            ]
        );
        assert_eq!(c.hex, Some(Hex([255, 0, 51])));
        assert_eq!(c.with_alpha(0.5).alpha, Some(0.5));
        assert_eq!(Color::new(ColorSpace::Oklch, [0.5, 0.1, 30.0]).hex, None);
    }
}
