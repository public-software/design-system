//! What can go wrong reading, building or resolving a token set.

use std::fmt;

use crate::model::{Path, TokenType};

/// An error reading, building or resolving a token set.
///
/// Every variant that concerns one token or group carries its [`Path`], so a reader of a
/// large file is told where to look.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The text is not JSON.
    Json(serde_json::Error),
    /// A token or group name breaks the format's rules: empty, starting with `$` (unless
    /// it is one of the format's properties), or containing `.`, `{` or `}`.
    InvalidName {
        /// The group the name appears in (empty for the root).
        parent: Path,
        /// The offending name.
        name: String,
    },
    /// A name is used twice in the same group (only when building in Rust; JSON objects
    /// cannot express it).
    Duplicate {
        /// The group the name appears in.
        parent: Path,
        /// The name.
        name: String,
    },
    /// A token has neither its own `$type` nor one inherited from a group. The format
    /// forbids inferring a type from the value.
    MissingType {
        /// The token.
        path: Path,
    },
    /// A `$type` that this crate does not know: either not a DTCG type at all, or one the
    /// current slice (colour, dimension) does not carry.
    UnknownType {
        /// The token or group that declares it.
        path: Path,
        /// The `$type` string as written.
        ty: String,
    },
    /// A `$value` that does not have the shape its type asks for.
    InvalidValue {
        /// The token.
        path: Path,
        /// What was wrong, in words.
        reason: String,
    },
    /// A path names no token in the set.
    UnknownToken {
        /// The path looked up.
        path: Path,
    },
    /// Following an alias chain from `path` came back on itself: the alias at `through`
    /// points at a token the chain had already visited.
    Cycle {
        /// The token whose value was asked for.
        path: Path,
        /// The last token of the chain, whose alias closes the loop.
        through: Path,
    },
    /// An alias points at a token of another type.
    TypeMismatch {
        /// The alias token.
        path: Path,
        /// The type the alias declares.
        expected: TokenType,
        /// The token the alias (eventually) points at.
        target: Path,
        /// That token's type.
        found: TokenType,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(e) => write!(f, "the text is not JSON: {e}"),
            Self::InvalidName { parent, name } if parent.is_root() => {
                write!(f, "{name:?} is not a valid token or group name")
            }
            Self::InvalidName { parent, name } => {
                write!(f, "{name:?} in {parent} is not a valid token or group name")
            }
            Self::Duplicate { parent, name } if parent.is_root() => {
                write!(f, "{name:?} is already in the root group")
            }
            Self::Duplicate { parent, name } => write!(f, "{name:?} is already in {parent}"),
            Self::MissingType { path } => {
                write!(f, "{path} has no $type of its own and inherits none")
            }
            Self::UnknownType { path, ty } => {
                write!(f, "{path}: unknown or unsupported $type {ty:?}")
            }
            Self::InvalidValue { path, reason } => write!(f, "{path}: invalid $value: {reason}"),
            Self::UnknownToken { path } => write!(f, "no token at {path}"),
            Self::Cycle { path, through } => {
                write!(
                    f,
                    "{path}: the alias chain is circular, closed at {through}"
                )
            }
            Self::TypeMismatch {
                path,
                expected,
                target,
                found,
            } => write!(
                f,
                "{path} is a {expected} but its alias target {target} is a {found}"
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}
