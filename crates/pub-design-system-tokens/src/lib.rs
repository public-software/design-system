//! `pub-design-system-tokens` — the `tokens` library of
//! [`design-system`](https://github.com/public-software/design-system).
//!
//! The design tokens of the suite: the one look and feel as typed Rust ([`suite`]) and as
//! the JSON interchange every tool reads, the Design Tokens Format Module 2025.10 of the
//! W3C Design Tokens Community Group with its Color Module (both listed in the repository's
//! `PROVENANCE.md`). A [`TokenSet`] is read from that JSON ([`TokenSet::from_json_str`]),
//! written back canonically ([`TokenSet::to_json_string`]), built in Rust, and queried by
//! [`Path`]: [`TokenSet::get`] gives a token as written, [`TokenSet::resolve`] follows its
//! aliases to a value.
//!
//! This first slice carries the `color` and `dimension` types, tokens and groups with
//! `$description`, `$deprecated` and `$extensions`, `$type` inheritance from groups, and
//! `{group.token}` aliases. Any other `$type` is [`Error::UnknownType`]; a token with no
//! resolvable type is [`Error::MissingType`] (the format forbids inferring one); a value of
//! the wrong shape is [`Error::InvalidValue`] with its path.
//!
//! ```
//! use pub_design_system_tokens::{Path, TokenSet, TokenType, Value};
//!
//! let set = TokenSet::from_json_str(r##"{
//!   "color": {
//!     "$type": "color",
//!     "blue": { "$value": { "colorSpace": "srgb", "components": [0, 0.4, 0.8], "hex": "#0066cc" } },
//!     "primary": { "$value": "{color.blue}" }
//!   },
//!   "space": { "1": { "$type": "dimension", "$value": { "value": 0.25, "unit": "rem" } } }
//! }"##)?;
//!
//! let primary = Path::parse("color.primary")?;
//! assert_eq!(set.get(&primary).unwrap().ty, TokenType::Color);
//! assert!(matches!(set.resolve(&primary)?, Value::Color(c) if c.hex.unwrap().to_string() == "#0066cc"));
//! assert_eq!(TokenSet::from_json_str(&set.to_json_string())?, set);
//! # Ok::<(), pub_design_system_tokens::Error>(())
//! ```
//!
//! The suite's own tokens are [`suite::tokens`]; their export is `tokens/suite.tokens.json`.
//!
//! ```
//! assert_eq!(pub_design_system_tokens::NAME, "pub-design-system-tokens");
//! ```

#![forbid(unsafe_code)]

mod color;
mod dimension;
mod error;
mod json;
mod model;
pub mod suite;

pub use color::{Color, ColorSpace, Component, Hex};
pub use dimension::{Dimension, Unit};
pub use error::Error;
pub use model::{
    Deprecated, Extensions, Group, Node, NodeRef, Path, Token, TokenSet, TokenType, Value,
    is_valid_name,
};

/// The crate's name, as `CATALOG.toml` and crates.io know it.
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// The crate's version, as Cargo knows it.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_follows_the_naming_rule() {
        assert_eq!(NAME, "pub-design-system-tokens");
        assert!(NAME.starts_with("pub-design-system-"));
    }

    #[test]
    fn version_is_semver_shaped() {
        assert_eq!(VERSION.split('.').count(), 3, "{VERSION}");
    }

    #[test]
    fn errors_print_their_path() {
        let path = Path::parse("a.b").unwrap();
        let cases: Vec<(Error, &str)> = vec![
            (
                Error::MissingType { path: path.clone() },
                "a.b has no $type",
            ),
            (
                Error::UnknownType {
                    path: path.clone(),
                    ty: "x".into(),
                },
                "\"x\"",
            ),
            (
                Error::InvalidValue {
                    path: path.clone(),
                    reason: "why".into(),
                },
                "a.b: invalid $value: why",
            ),
            (
                Error::UnknownToken { path: path.clone() },
                "no token at a.b",
            ),
            (
                Error::Cycle {
                    path: path.clone(),
                    through: path.clone(),
                },
                "closed at a.b",
            ),
            (
                Error::TypeMismatch {
                    path: path.clone(),
                    expected: TokenType::Color,
                    target: Path::parse("c").unwrap(),
                    found: TokenType::Dimension,
                },
                "a.b is a color but its alias target c is a dimension",
            ),
            (
                Error::InvalidName {
                    parent: Path::root(),
                    name: "$x".into(),
                },
                "\"$x\" is not",
            ),
            (
                Error::InvalidName {
                    parent: path.clone(),
                    name: "$x".into(),
                },
                "in a.b",
            ),
            (
                Error::Duplicate {
                    parent: Path::root(),
                    name: "x".into(),
                },
                "root group",
            ),
            (
                Error::Duplicate {
                    parent: path.clone(),
                    name: "x".into(),
                },
                "already in a.b",
            ),
        ];
        for (error, expected) in cases {
            let text = error.to_string();
            assert!(text.contains(expected), "{text:?} lacks {expected:?}");
        }
        let json: Error = serde_json::from_str::<serde_json::Value>("{")
            .unwrap_err()
            .into();
        assert!(json.to_string().starts_with("the text is not JSON"));
        assert!(std::error::Error::source(&json).is_some());
        assert!(std::error::Error::source(&Error::UnknownToken { path }).is_none());
    }
}
