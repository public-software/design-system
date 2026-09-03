//! The `dimension` type of the Design Tokens Format Module 2025.10.

use std::fmt;

/// The unit of a dimension: the format allows exactly `px` and `rem`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Unit {
    /// CSS pixels.
    Px,
    /// Root em: relative to the root font size.
    Rem,
}

impl Unit {
    /// The identifier the JSON carries in `unit`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Px => "px",
            Self::Rem => "rem",
        }
    }

    /// The unit for a `unit` identifier, `None` for anything else (`em`, `%`, `pt`, …).
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "px" => Some(Self::Px),
            "rem" => Some(Self::Rem),
            _ => None,
        }
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A distance: the `dimension` type, a number with a unit.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Dimension {
    /// The amount.
    pub value: f64,
    /// Its unit.
    pub unit: Unit,
}

impl Dimension {
    /// A dimension in CSS pixels.
    pub const fn px(value: f64) -> Self {
        Self {
            value,
            unit: Unit::Px,
        }
    }

    /// A dimension in root ems.
    pub const fn rem(value: f64) -> Self {
        Self {
            value,
            unit: Unit::Rem,
        }
    }
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.value, self.unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_units_parse_and_print() {
        assert_eq!(Unit::parse("px"), Some(Unit::Px));
        assert_eq!(Unit::parse("rem"), Some(Unit::Rem));
        for bad in ["em", "%", "pt", "PX", ""] {
            assert_eq!(Unit::parse(bad), None, "{bad}");
        }
        assert_eq!(Dimension::rem(0.25).to_string(), "0.25rem");
        assert_eq!(Dimension::px(4.0).to_string(), "4px");
    }
}
