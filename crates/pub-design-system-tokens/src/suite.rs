//! The suite's own tokens: the one look and feel, as typed Rust and as the
//! [`TokenSet`] every program and tool reads.
//!
//! The constants are the source; [`tokens`] assembles them, and `tokens/suite.tokens.json`
//! in this crate is their export, kept current by a test (regenerate it with
//! `cargo run --example export > tokens/suite.tokens.json`). This first slice is colour and
//! spacing; the type scale, radii, motion durations and elevation follow.
//!
//! The colour tokens are two layers: the **palette** (`color.neutral.*`, `color.accent.*`,
//! `color.success.*`, `color.warning.*`, `color.danger.*`), sRGB colours written from their
//! 8-bit channels so the `hex` fallback and the components agree; and the **roles**
//! (`color.text.*`, `color.surface.*`, `color.border.*`, `color.action.*`), aliases into the
//! palette. A theme changes the roles, not the palette. Spacing is one scale,
//! `space.<n>` = `n × 0.25 rem`, the 4-pixel grid at the default root size.

use crate::model::{Group, Path, Token, TokenSet, TokenType};

/// The palette: every colour constant, and the list `tokens` builds from.
pub mod color {
    use crate::color::Color;

    /// `color.neutral.0`: white.
    pub const NEUTRAL_0: Color = Color::from_rgb8(0xff, 0xff, 0xff);
    /// `color.neutral.50`.
    pub const NEUTRAL_50: Color = Color::from_rgb8(0xf7, 0xf8, 0xfa);
    /// `color.neutral.100`.
    pub const NEUTRAL_100: Color = Color::from_rgb8(0xec, 0xee, 0xf2);
    /// `color.neutral.200`.
    pub const NEUTRAL_200: Color = Color::from_rgb8(0xd9, 0xdd, 0xe4);
    /// `color.neutral.300`.
    pub const NEUTRAL_300: Color = Color::from_rgb8(0xb9, 0xc0, 0xcc);
    /// `color.neutral.400`.
    pub const NEUTRAL_400: Color = Color::from_rgb8(0x8d, 0x97, 0xa8);
    /// `color.neutral.500`.
    pub const NEUTRAL_500: Color = Color::from_rgb8(0x6b, 0x75, 0x88);
    /// `color.neutral.600`.
    pub const NEUTRAL_600: Color = Color::from_rgb8(0x52, 0x5b, 0x6c);
    /// `color.neutral.700`.
    pub const NEUTRAL_700: Color = Color::from_rgb8(0x3d, 0x44, 0x52);
    /// `color.neutral.800`.
    pub const NEUTRAL_800: Color = Color::from_rgb8(0x2a, 0x2f, 0x3a);
    /// `color.neutral.900`.
    pub const NEUTRAL_900: Color = Color::from_rgb8(0x1a, 0x1d, 0x25);
    /// `color.neutral.1000`: the darkest neutral, not pure black.
    pub const NEUTRAL_1000: Color = Color::from_rgb8(0x0d, 0x0f, 0x14);

    /// `color.accent.100`.
    pub const ACCENT_100: Color = Color::from_rgb8(0xdb, 0xe9, 0xff);
    /// `color.accent.200`.
    pub const ACCENT_200: Color = Color::from_rgb8(0xb3, 0xd1, 0xff);
    /// `color.accent.300`.
    pub const ACCENT_300: Color = Color::from_rgb8(0x7f, 0xb0, 0xff);
    /// `color.accent.400`.
    pub const ACCENT_400: Color = Color::from_rgb8(0x4a, 0x8d, 0xff);
    /// `color.accent.500`: the accent itself.
    pub const ACCENT_500: Color = Color::from_rgb8(0x1f, 0x6a, 0xeb);
    /// `color.accent.600`.
    pub const ACCENT_600: Color = Color::from_rgb8(0x15, 0x54, 0xc2);
    /// `color.accent.700`.
    pub const ACCENT_700: Color = Color::from_rgb8(0x0f, 0x40, 0x99);
    /// `color.accent.800`.
    pub const ACCENT_800: Color = Color::from_rgb8(0x0b, 0x2f, 0x70);
    /// `color.accent.900`.
    pub const ACCENT_900: Color = Color::from_rgb8(0x07, 0x1f, 0x4a);

    /// `color.success.100`.
    pub const SUCCESS_100: Color = Color::from_rgb8(0xdc, 0xf5, 0xe6);
    /// `color.success.500`.
    pub const SUCCESS_500: Color = Color::from_rgb8(0x1a, 0x8f, 0x4e);
    /// `color.success.700`.
    pub const SUCCESS_700: Color = Color::from_rgb8(0x11, 0x63, 0x36);
    /// `color.warning.100`.
    pub const WARNING_100: Color = Color::from_rgb8(0xfd, 0xf0, 0xd5);
    /// `color.warning.500`.
    pub const WARNING_500: Color = Color::from_rgb8(0xc7, 0x7b, 0x00);
    /// `color.warning.700`.
    pub const WARNING_700: Color = Color::from_rgb8(0x8a, 0x55, 0x00);
    /// `color.danger.100`.
    pub const DANGER_100: Color = Color::from_rgb8(0xfd, 0xe2, 0xe2);
    /// `color.danger.500`.
    pub const DANGER_500: Color = Color::from_rgb8(0xd1, 0x34, 0x38);
    /// `color.danger.700`.
    pub const DANGER_700: Color = Color::from_rgb8(0x94, 0x22, 0x25);

    /// Every palette colour with its path below `color`.
    pub const PALETTE: [(&str, Color); 30] = [
        ("neutral.0", NEUTRAL_0),
        ("neutral.50", NEUTRAL_50),
        ("neutral.100", NEUTRAL_100),
        ("neutral.200", NEUTRAL_200),
        ("neutral.300", NEUTRAL_300),
        ("neutral.400", NEUTRAL_400),
        ("neutral.500", NEUTRAL_500),
        ("neutral.600", NEUTRAL_600),
        ("neutral.700", NEUTRAL_700),
        ("neutral.800", NEUTRAL_800),
        ("neutral.900", NEUTRAL_900),
        ("neutral.1000", NEUTRAL_1000),
        ("accent.100", ACCENT_100),
        ("accent.200", ACCENT_200),
        ("accent.300", ACCENT_300),
        ("accent.400", ACCENT_400),
        ("accent.500", ACCENT_500),
        ("accent.600", ACCENT_600),
        ("accent.700", ACCENT_700),
        ("accent.800", ACCENT_800),
        ("accent.900", ACCENT_900),
        ("success.100", SUCCESS_100),
        ("success.500", SUCCESS_500),
        ("success.700", SUCCESS_700),
        ("warning.100", WARNING_100),
        ("warning.500", WARNING_500),
        ("warning.700", WARNING_700),
        ("danger.100", DANGER_100),
        ("danger.500", DANGER_500),
        ("danger.700", DANGER_700),
    ];

    /// The roles: each `color.<role>` token and the palette token it aliases, both as paths
    /// below `color`.
    pub const ROLES: [(&str, &str); 11] = [
        ("text.primary", "neutral.900"),
        ("text.secondary", "neutral.600"),
        ("text.inverse", "neutral.0"),
        ("surface.default", "neutral.0"),
        ("surface.raised", "neutral.50"),
        ("surface.sunken", "neutral.100"),
        ("border.default", "neutral.200"),
        ("border.strong", "neutral.400"),
        ("action.primary", "accent.500"),
        ("action.primary-hover", "accent.600"),
        ("action.primary-active", "accent.700"),
    ];
}

/// The spacing scale: `space.<n>` = `n × STEP_REM` rem.
pub mod space {
    use crate::dimension::Dimension;

    /// One step of the grid in rem: `0.25` (4 pixels at a 16-pixel root).
    pub const STEP_REM: f64 = 0.25;

    /// `space.0`: no space.
    pub const SPACE_0: Dimension = Dimension::rem(0.0);
    /// `space.1`: one step.
    pub const SPACE_1: Dimension = Dimension::rem(STEP_REM);
    /// `space.2`.
    pub const SPACE_2: Dimension = Dimension::rem(2.0 * STEP_REM);
    /// `space.3`.
    pub const SPACE_3: Dimension = Dimension::rem(3.0 * STEP_REM);
    /// `space.4`: one rem.
    pub const SPACE_4: Dimension = Dimension::rem(4.0 * STEP_REM);
    /// `space.6`.
    pub const SPACE_6: Dimension = Dimension::rem(6.0 * STEP_REM);
    /// `space.8`.
    pub const SPACE_8: Dimension = Dimension::rem(8.0 * STEP_REM);
    /// `space.12`.
    pub const SPACE_12: Dimension = Dimension::rem(12.0 * STEP_REM);
    /// `space.16`.
    pub const SPACE_16: Dimension = Dimension::rem(16.0 * STEP_REM);
    /// `space.24`.
    pub const SPACE_24: Dimension = Dimension::rem(24.0 * STEP_REM);
    /// `space.32`.
    pub const SPACE_32: Dimension = Dimension::rem(32.0 * STEP_REM);

    /// Every spacing token with its name below `space`.
    pub const SCALE: [(&str, Dimension); 11] = [
        ("0", SPACE_0),
        ("1", SPACE_1),
        ("2", SPACE_2),
        ("3", SPACE_3),
        ("4", SPACE_4),
        ("6", SPACE_6),
        ("8", SPACE_8),
        ("12", SPACE_12),
        ("16", SPACE_16),
        ("24", SPACE_24),
        ("32", SPACE_32),
    ];
}

/// Inserts `token` at `path` below `root`, creating the groups on the way.
fn insert_at(root: &mut Group, path: &str, token: Token) {
    let path = Path::parse(path).expect("the suite's paths are well-formed");
    let (name, groups) = path
        .segments()
        .split_last()
        .expect("a token path is not empty");
    let mut group = root;
    for segment in groups {
        if group.child(segment).is_none() {
            group
                .insert_group(segment, Group::new())
                .expect("a fresh name");
        }
        group = match group.child_mut(segment) {
            Some(crate::model::Node::Group(g)) => g,
            _ => unreachable!("the suite's paths never cross a token"),
        };
    }
    group
        .insert_token(name, token)
        .expect("the suite's names are unique");
}

/// The suite's token set: the palette and the roles under `color` (a `color`-typed
/// group), the scale under `space` (a `dimension`-typed group).
///
/// ```
/// use pub_design_system_tokens::{Path, Value, suite};
///
/// let set = suite::tokens();
/// let primary = set.resolve(&Path::parse("color.action.primary")?)?;
/// assert_eq!(primary, &Value::Color(suite::color::ACCENT_500));
/// # Ok::<(), pub_design_system_tokens::Error>(())
/// ```
pub fn tokens() -> TokenSet {
    let mut set = TokenSet::new();

    let mut colors = Group::new();
    colors.ty = Some(TokenType::Color);
    colors.description = Some(
        "The palette (neutral, accent, success, warning, danger) and the roles that alias into it; \
         a theme changes the roles."
            .to_owned(),
    );
    for (path, value) in color::PALETTE {
        insert_at(&mut colors, path, Token::color(value));
    }
    for (role, target) in color::ROLES {
        let target = Path::parse(&format!("color.{target}")).expect("a palette path");
        insert_at(&mut colors, role, Token::alias(TokenType::Color, target));
    }
    set.root_mut()
        .insert_group("color", colors)
        .expect("a fresh root");

    let mut spaces = Group::new();
    spaces.ty = Some(TokenType::Dimension);
    spaces.description =
        Some("The spacing scale: space.n is n quarter-rems, the 4-pixel grid.".to_owned());
    for (name, value) in space::SCALE {
        spaces
            .insert_token(name, Token::dimension(value))
            .expect("the scale's names are unique");
    }
    set.root_mut()
        .insert_group("space", spaces)
        .expect("a fresh root");

    set
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_role_points_at_a_palette_entry() {
        let palette: Vec<&str> = color::PALETTE.iter().map(|(p, _)| *p).collect();
        for (role, target) in color::ROLES {
            assert!(palette.contains(&target), "{role} -> {target}");
        }
    }

    #[test]
    fn the_set_has_the_palette_the_roles_and_the_scale() {
        let set = tokens();
        assert_eq!(
            set.tokens().count(),
            color::PALETTE.len() + color::ROLES.len() + space::SCALE.len()
        );
        for (name, value) in space::SCALE {
            let path = Path::parse(&format!("space.{name}")).unwrap();
            assert_eq!(
                set.get(&path).unwrap().value,
                crate::model::Value::Dimension(value)
            );
        }
    }
}
