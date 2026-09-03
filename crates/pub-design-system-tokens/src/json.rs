//! Reading a token set from the DTCG JSON interchange and writing it back.
//!
//! Reading is strict: every rule of the format this crate knows is checked and reported
//! with the path it concerns. Writing is canonical: keys in name order, an inherited
//! `$type` not repeated, `hex` in lower case; so `import(export(set)) == set`, and
//! `export` of the result is byte-identical.

use serde_json::{Map, Value as Json};

use crate::color::{Color, ColorSpace, Component, Hex};
use crate::dimension::{Dimension, Unit};
use crate::error::Error;
use crate::model::{Deprecated, Extensions, Group, Node, Path, Token, TokenSet, TokenType, Value};

// ---- reading -------------------------------------------------------------------------------

impl TokenSet {
    /// Reads a token file from its JSON text.
    ///
    /// Errors: [`Error::Json`] for text that is not JSON, then the format's rules, each
    /// with the path of the token or group that breaks it (see [`Error`]).
    pub fn from_json_str(text: &str) -> Result<Self, Error> {
        Self::from_json_value(&serde_json::from_str(text)?)
    }

    /// Reads a token file from parsed JSON.
    pub fn from_json_value(value: &Json) -> Result<Self, Error> {
        let object = value.as_object().ok_or_else(|| Error::InvalidValue {
            path: Path::root(),
            reason: "the root must be an object of groups and tokens".to_owned(),
        })?;
        Ok(Self::from_root(read_group(object, &Path::root(), None)?))
    }
}

fn invalid(path: &Path, reason: impl Into<String>) -> Error {
    Error::InvalidValue {
        path: path.clone(),
        reason: reason.into(),
    }
}

fn read_type(value: &Json, path: &Path) -> Result<TokenType, Error> {
    let text = value
        .as_str()
        .ok_or_else(|| invalid(path, "$type must be a string"))?;
    TokenType::parse(text).ok_or_else(|| Error::UnknownType {
        path: path.clone(),
        ty: text.to_owned(),
    })
}

fn read_description(value: &Json, path: &Path) -> Result<String, Error> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| invalid(path, "$description must be a string"))
}

fn read_deprecated(value: &Json, path: &Path) -> Result<Deprecated, Error> {
    match value {
        Json::Bool(flag) => Ok(Deprecated::Flag(*flag)),
        Json::String(reason) => Ok(Deprecated::Reason(reason.clone())),
        _ => Err(invalid(
            path,
            "$deprecated is true, false or a reason string",
        )),
    }
}

fn read_extensions(value: &Json, path: &Path) -> Result<Extensions, Error> {
    let object = value
        .as_object()
        .ok_or_else(|| invalid(path, "$extensions must be an object"))?;
    Ok(object
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect())
}

fn read_group(
    object: &Map<String, Json>,
    path: &Path,
    inherited: Option<TokenType>,
) -> Result<Group, Error> {
    let mut group = Group::new();
    for (key, value) in object {
        match key.as_str() {
            "$type" => group.ty = Some(read_type(value, path)?),
            "$description" => group.description = Some(read_description(value, path)?),
            "$deprecated" => group.deprecated = Some(read_deprecated(value, path)?),
            "$extensions" => group.extensions = read_extensions(value, path)?,
            "$extends" => return Err(invalid(path, "$extends is not supported yet")),
            key if key.starts_with('$') => {
                return Err(Error::InvalidName {
                    parent: path.clone(),
                    name: key.to_owned(),
                });
            }
            _ => {}
        }
    }
    let effective = group.ty.or(inherited);
    for (name, value) in object {
        if name.starts_with('$') {
            continue;
        }
        let child = path.join(name);
        if !crate::model::is_valid_name(name) {
            return Err(Error::InvalidName {
                parent: path.clone(),
                name: name.clone(),
            });
        }
        let object = value
            .as_object()
            .ok_or_else(|| invalid(&child, "a token or group is a JSON object"))?;
        let node = if object.contains_key("$value") {
            Node::Token(read_token(object, &child, effective)?)
        } else {
            Node::Group(read_group(object, &child, effective)?)
        };
        group.insert(name, node).map_err(|_| Error::Duplicate {
            parent: path.clone(),
            name: name.clone(),
        })?;
    }
    Ok(group)
}

fn read_token(
    object: &Map<String, Json>,
    path: &Path,
    inherited: Option<TokenType>,
) -> Result<Token, Error> {
    let ty = match object.get("$type") {
        Some(value) => read_type(value, path)?,
        None => inherited.ok_or_else(|| Error::MissingType { path: path.clone() })?,
    };
    let mut description = None;
    let mut deprecated = None;
    let mut extensions = Extensions::new();
    for (key, value) in object {
        match key.as_str() {
            "$type" | "$value" => {}
            "$description" => description = Some(read_description(value, path)?),
            "$deprecated" => deprecated = Some(read_deprecated(value, path)?),
            "$extensions" => extensions = read_extensions(value, path)?,
            _ => {
                return Err(Error::InvalidName {
                    parent: path.clone(),
                    name: key.to_owned(),
                });
            }
        }
    }
    let value = read_value(ty, &object["$value"], path)?;
    Ok(Token {
        ty,
        value,
        description,
        deprecated,
        extensions,
    })
}

fn read_value(ty: TokenType, value: &Json, path: &Path) -> Result<Value, Error> {
    if let Some(text) = value.as_str()
        && let Some(inner) = text.strip_prefix('{').and_then(|t| t.strip_suffix('}'))
    {
        let target = Path::parse(inner).map_err(|e| {
            invalid(
                path,
                format!("a reference is {{group.token}} with valid names: {e}"),
            )
        })?;
        return Ok(Value::Alias(target));
    }
    match ty {
        TokenType::Color => read_color(value, path).map(Value::Color),
        TokenType::Dimension => read_dimension(value, path).map(Value::Dimension),
    }
}

fn read_object<'a>(
    value: &'a Json,
    path: &Path,
    shape: &str,
    fields: &[&str],
) -> Result<&'a Map<String, Json>, Error> {
    let object = value.as_object().ok_or_else(|| {
        invalid(
            path,
            format!(
                "a {shape} $value is an object {{{}}} or a reference {{group.token}}",
                fields.join(", ")
            ),
        )
    })?;
    if let Some(unknown) = object.keys().find(|key| !fields.contains(&key.as_str())) {
        return Err(invalid(
            path,
            format!("unknown field {unknown:?} in a {shape} $value"),
        ));
    }
    Ok(object)
}

fn read_color(value: &Json, path: &Path) -> Result<Color, Error> {
    let object = read_object(
        value,
        path,
        "color",
        &["colorSpace", "components", "alpha", "hex"],
    )?;
    let space = object
        .get("colorSpace")
        .and_then(Json::as_str)
        .ok_or_else(|| invalid(path, "colorSpace is required and is a string"))?;
    let color_space = ColorSpace::parse(space)
        .ok_or_else(|| invalid(path, format!("unknown colorSpace {space:?}")))?;
    let components = object
        .get("components")
        .and_then(Json::as_array)
        .filter(|list| list.len() == 3)
        .ok_or_else(|| {
            invalid(
                path,
                "components is required and is an array of three entries",
            )
        })?;
    let mut parsed = [Component::None; 3];
    for (i, entry) in components.iter().enumerate() {
        parsed[i] = match entry {
            Json::Number(n) => Component::Value(n.as_f64().unwrap_or(f64::NAN)),
            Json::String(s) if s == "none" => Component::None,
            _ => {
                return Err(invalid(
                    path,
                    format!("component {i} is neither a number nor \"none\""),
                ));
            }
        };
    }
    let alpha = match object.get("alpha") {
        None => None,
        Some(value) => Some(
            value
                .as_f64()
                .filter(|a| (0.0..=1.0).contains(a))
                .ok_or_else(|| invalid(path, "alpha is a number between 0 and 1"))?,
        ),
    };
    let hex =
        match object.get("hex") {
            None => None,
            Some(value) => Some(value.as_str().and_then(Hex::parse).ok_or_else(|| {
                invalid(path, "hex is a six-digit CSS hex colour like \"#0066cc\"")
            })?),
        };
    Ok(Color {
        color_space,
        components: parsed,
        alpha,
        hex,
    })
}

fn read_dimension(value: &Json, path: &Path) -> Result<Dimension, Error> {
    let object = read_object(value, path, "dimension", &["value", "unit"])?;
    let amount = object
        .get("value")
        .and_then(Json::as_f64)
        .ok_or_else(|| invalid(path, "value is required and is a number"))?;
    let unit = object
        .get("unit")
        .and_then(Json::as_str)
        .ok_or_else(|| invalid(path, "unit is required and is \"px\" or \"rem\""))?;
    let unit = Unit::parse(unit).ok_or_else(|| invalid(path, format!("unknown unit {unit:?}")))?;
    Ok(Dimension {
        value: amount,
        unit,
    })
}

// ---- writing -------------------------------------------------------------------------------

impl TokenSet {
    /// The set as JSON, canonical: keys in name order, an inherited `$type` not repeated on
    /// the tokens below, `hex` in lower case. A non-finite number becomes `null`, which
    /// does not read back.
    pub fn to_json_value(&self) -> Json {
        Json::Object(write_group(self.root(), None))
    }

    /// [`TokenSet::to_json_value`] pretty-printed, ending in a newline.
    pub fn to_json_string(&self) -> String {
        let mut text = serde_json::to_string_pretty(&self.to_json_value())
            .expect("a JSON value always serializes");
        text.push('\n');
        text
    }
}

fn number(x: f64) -> Json {
    serde_json::Number::from_f64(x).map_or(Json::Null, Json::Number)
}

fn write_metadata(
    out: &mut Map<String, Json>,
    description: Option<&String>,
    deprecated: Option<&Deprecated>,
    extensions: &Extensions,
) {
    if let Some(description) = description {
        out.insert("$description".to_owned(), Json::String(description.clone()));
    }
    match deprecated {
        Some(Deprecated::Flag(flag)) => {
            out.insert("$deprecated".to_owned(), Json::Bool(*flag));
        }
        Some(Deprecated::Reason(reason)) => {
            out.insert("$deprecated".to_owned(), Json::String(reason.clone()));
        }
        None => {}
    }
    if !extensions.is_empty() {
        out.insert(
            "$extensions".to_owned(),
            Json::Object(
                extensions
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            ),
        );
    }
}

fn write_group(group: &Group, inherited: Option<TokenType>) -> Map<String, Json> {
    let mut out = Map::new();
    if let Some(ty) = group.ty {
        out.insert("$type".to_owned(), Json::String(ty.as_str().to_owned()));
    }
    write_metadata(
        &mut out,
        group.description.as_ref(),
        group.deprecated.as_ref(),
        &group.extensions,
    );
    let effective = group.ty.or(inherited);
    for (name, node) in group.children() {
        let value = match node {
            Node::Token(token) => write_token(token, effective),
            Node::Group(group) => write_group(group, effective),
        };
        out.insert(name.to_owned(), Json::Object(value));
    }
    out
}

fn write_token(token: &Token, inherited: Option<TokenType>) -> Map<String, Json> {
    let mut out = Map::new();
    if Some(token.ty) != inherited {
        out.insert(
            "$type".to_owned(),
            Json::String(token.ty.as_str().to_owned()),
        );
    }
    out.insert("$value".to_owned(), write_value(&token.value));
    write_metadata(
        &mut out,
        token.description.as_ref(),
        token.deprecated.as_ref(),
        &token.extensions,
    );
    out
}

fn write_value(value: &Value) -> Json {
    match value {
        Value::Alias(path) => Json::String(format!("{{{path}}}")),
        Value::Color(color) => {
            let mut out = Map::new();
            out.insert(
                "colorSpace".to_owned(),
                Json::String(color.color_space.as_str().to_owned()),
            );
            let components = color
                .components
                .iter()
                .map(|component| match component {
                    Component::Value(x) => number(*x),
                    Component::None => Json::String("none".to_owned()),
                })
                .collect();
            out.insert("components".to_owned(), Json::Array(components));
            if let Some(alpha) = color.alpha {
                out.insert("alpha".to_owned(), number(alpha));
            }
            if let Some(hex) = color.hex {
                out.insert("hex".to_owned(), Json::String(hex.to_string()));
            }
            Json::Object(out)
        }
        Value::Dimension(dimension) => {
            let mut out = Map::new();
            out.insert("value".to_owned(), number(dimension.value));
            out.insert(
                "unit".to_owned(),
                Json::String(dimension.unit.as_str().to_owned()),
            );
            Json::Object(out)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_non_finite_number_is_written_as_null() {
        assert_eq!(number(f64::NAN), Json::Null);
        assert_eq!(number(f64::INFINITY), Json::Null);
        assert_eq!(number(0.25), serde_json::json!(0.25));
    }

    #[test]
    fn an_unknown_field_in_a_value_object_is_rejected() {
        let json = r#"{ "d": { "$type": "dimension", "$value": { "value": 1, "unit": "px", "units": "px" } } }"#;
        assert!(matches!(
            TokenSet::from_json_str(json),
            Err(Error::InvalidValue { reason, .. }) if reason.contains("units")
        ));
    }

    #[test]
    fn bad_metadata_shapes_are_rejected_with_the_path() {
        for (json, word) in [
            (r#"{ "g": { "$description": 1 } }"#, "$description"),
            (r#"{ "g": { "$deprecated": 1 } }"#, "$deprecated"),
            (r#"{ "g": { "$extensions": [] } }"#, "$extensions"),
            (r#"{ "g": { "$type": 1 } }"#, "$type"),
            (r#"{ "g": { "$extends": "{h}" } }"#, "$extends"),
            (r#"{ "g": 1 }"#, "object"),
        ] {
            match TokenSet::from_json_str(json) {
                Err(Error::InvalidValue { path, reason }) => {
                    assert_eq!(path.to_string(), "g", "{json}");
                    assert!(reason.contains(word), "{json}: {reason}");
                }
                other => panic!("{json}: {other:?}"),
            }
        }
    }

    #[test]
    fn a_token_metadata_error_names_the_token() {
        let json = r#"{ "t": { "$type": "dimension", "$value": { "value": 1, "unit": "px" }, "$description": [] } }"#;
        assert!(matches!(
            TokenSet::from_json_str(json),
            Err(Error::InvalidValue { path, .. }) if path.to_string() == "t"
        ));
    }
}
