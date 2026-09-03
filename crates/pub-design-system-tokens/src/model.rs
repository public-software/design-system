//! The token model: paths, types, values, tokens, groups and the set that holds them.

use std::collections::BTreeMap;
use std::fmt;

use crate::color::Color;
use crate::dimension::Dimension;
use crate::error::Error;

/// Whether `name` may name a token or a group: not empty, not starting with `$`, and free
/// of `.`, `{` and `}` (the format reserves them for aliases).
pub fn is_valid_name(name: &str) -> bool {
    !name.is_empty() && !name.starts_with('$') && !name.contains(['.', '{', '}'])
}

/// Where a token or group sits: its ancestors' names and its own, written `a.b.c`.
///
/// The root group's path is empty ([`Path::root`]). A path is what an alias holds and
/// what [`TokenSet::get`] and [`TokenSet::resolve`] take.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Path(Vec<String>);

impl Path {
    /// The root group's path: no segments.
    pub const fn root() -> Self {
        Self(Vec::new())
    }

    /// Reads `a.b.c`: one or more valid names joined by `.`.
    ///
    /// ```
    /// use pub_design_system_tokens::Path;
    /// assert_eq!(Path::parse("color.brand.blue").unwrap().to_string(), "color.brand.blue");
    /// assert!(Path::parse("color..blue").is_err());
    /// ```
    pub fn parse(text: &str) -> Result<Self, Error> {
        let mut segments = Vec::new();
        for segment in text.split('.') {
            if !is_valid_name(segment) {
                return Err(Error::InvalidName {
                    parent: Self(segments),
                    name: segment.to_owned(),
                });
            }
            segments.push(segment.to_owned());
        }
        Ok(Self(segments))
    }

    /// The names from the root down.
    pub fn segments(&self) -> &[String] {
        &self.0
    }

    /// Whether this is the root group's path.
    pub fn is_root(&self) -> bool {
        self.0.is_empty()
    }

    /// The last name, `None` for the root.
    pub fn name(&self) -> Option<&str> {
        self.0.last().map(String::as_str)
    }

    /// The path of the enclosing group, `None` for the root.
    pub fn parent(&self) -> Option<Self> {
        self.0.split_last().map(|(_, rest)| Self(rest.to_vec()))
    }

    /// This path with one more name at the end. The name is not checked here; the group
    /// that inserts it does.
    pub fn join(&self, name: &str) -> Self {
        let mut segments = self.0.clone();
        segments.push(name.to_owned());
        Self(segments)
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.join("."))
    }
}

/// The `$type` of a token: which shape its value has.
///
/// This slice of the format carries the two types the suite's first tokens need; the
/// others (`fontFamily`, `fontWeight`, `duration`, `cubicBezier`, `number` and the
/// composite types) are read as [`Error::UnknownType`] until they land.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TokenType {
    /// A [`Color`].
    Color,
    /// A [`Dimension`].
    Dimension,
}

impl TokenType {
    /// The identifier the JSON carries in `$type`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Color => "color",
            Self::Dimension => "dimension",
        }
    }

    /// The type for a `$type` identifier, `None` for anything else.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "color" => Some(Self::Color),
            "dimension" => Some(Self::Dimension),
            _ => None,
        }
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A token's `$value`: a colour, a dimension, or an alias of another token.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A colour.
    Color(Color),
    /// A dimension.
    Dimension(Dimension),
    /// A reference to another token's value, `{group.token}` in the JSON. Kept as a
    /// reference; [`TokenSet::resolve`] follows it.
    Alias(Path),
}

impl Value {
    /// The type this value has on its own; `None` for an alias, whose type is the token's
    /// declared `$type`.
    pub fn token_type(&self) -> Option<TokenType> {
        match self {
            Self::Color(_) => Some(TokenType::Color),
            Self::Dimension(_) => Some(TokenType::Dimension),
            Self::Alias(_) => None,
        }
    }
}

/// The `$deprecated` property: a flag, or a string saying why and what to use instead.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Deprecated {
    /// `true` or `false` as written (`false` undoes a group's deprecation for one token).
    Flag(bool),
    /// Deprecated, with the reason.
    Reason(String),
}

/// Vendor data under `$extensions`, keyed by reverse-domain names (`dev.publicsoftware.x`)
/// and kept as arbitrary JSON.
pub type Extensions = BTreeMap<String, serde_json::Value>;

/// A design token: a typed value with its metadata.
///
/// The fields are public so a set can be built in Rust; the constructors keep `ty` and
/// `value` in agreement, which the JSON reader also requires (a `dimension` token whose
/// value is a colour object is [`Error::InvalidValue`]).
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// The token's type: its own `$type`, or the one inherited from its groups.
    pub ty: TokenType,
    /// The value.
    pub value: Value,
    /// `$description`.
    pub description: Option<String>,
    /// `$deprecated`.
    pub deprecated: Option<Deprecated>,
    /// `$extensions`.
    pub extensions: Extensions,
}

impl Token {
    fn new(ty: TokenType, value: Value) -> Self {
        Self {
            ty,
            value,
            description: None,
            deprecated: None,
            extensions: Extensions::new(),
        }
    }

    /// A colour token.
    pub fn color(color: Color) -> Self {
        Self::new(TokenType::Color, Value::Color(color))
    }

    /// A dimension token.
    pub fn dimension(dimension: Dimension) -> Self {
        Self::new(TokenType::Dimension, Value::Dimension(dimension))
    }

    /// A token of type `ty` whose value is another token's.
    pub fn alias(ty: TokenType, target: Path) -> Self {
        Self::new(ty, Value::Alias(target))
    }

    /// The same token with a `$description`.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// The same token with a `$deprecated`.
    #[must_use]
    pub fn with_deprecated(mut self, deprecated: Deprecated) -> Self {
        self.deprecated = Some(deprecated);
        self
    }

    /// The same token with one more `$extensions` entry.
    #[must_use]
    pub fn with_extension(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.extensions.insert(key.into(), value);
        self
    }
}

/// A group: a named collection of tokens and groups, with an optional `$type` every token
/// below it inherits unless it declares its own.
///
/// Children are kept sorted by name, so an export is canonical.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Group {
    /// `$type`: the default type of the tokens below.
    pub ty: Option<TokenType>,
    /// `$description`.
    pub description: Option<String>,
    /// `$deprecated`; applies to every token below unless one overrides it.
    pub deprecated: Option<Deprecated>,
    /// `$extensions`.
    pub extensions: Extensions,
    children: BTreeMap<String, Node>,
}

impl Group {
    /// An empty group with no metadata.
    pub fn new() -> Self {
        Self::default()
    }

    /// The child called `name`, token or group.
    pub fn child(&self, name: &str) -> Option<&Node> {
        self.children.get(name)
    }

    /// The child called `name`, for changing it.
    pub fn child_mut(&mut self, name: &str) -> Option<&mut Node> {
        self.children.get_mut(name)
    }

    /// The children in name order.
    pub fn children(&self) -> impl Iterator<Item = (&str, &Node)> {
        self.children
            .iter()
            .map(|(name, node)| (name.as_str(), node))
    }

    /// How many direct children the group has.
    pub fn len(&self) -> usize {
        self.children.len()
    }

    /// Whether the group has no children.
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// Adds a child. The name must be valid ([`is_valid_name`]) and unused; the errors
    /// name the group as the root, since a detached group does not know its own path.
    pub fn insert(&mut self, name: &str, node: Node) -> Result<(), Error> {
        if !is_valid_name(name) {
            return Err(Error::InvalidName {
                parent: Path::root(),
                name: name.to_owned(),
            });
        }
        if self.children.contains_key(name) {
            return Err(Error::Duplicate {
                parent: Path::root(),
                name: name.to_owned(),
            });
        }
        self.children.insert(name.to_owned(), node);
        Ok(())
    }

    /// Adds a token; see [`Group::insert`].
    pub fn insert_token(&mut self, name: &str, token: Token) -> Result<(), Error> {
        self.insert(name, Node::Token(token))
    }

    /// Adds a group; see [`Group::insert`].
    pub fn insert_group(&mut self, name: &str, group: Group) -> Result<(), Error> {
        self.insert(name, Node::Group(group))
    }

    /// Removes and returns the child called `name`.
    pub fn remove(&mut self, name: &str) -> Option<Node> {
        self.children.remove(name)
    }

    fn walk<'a>(&'a self, path: &Path, out: &mut Vec<(Path, &'a Token)>) {
        for (name, node) in &self.children {
            let child = path.join(name);
            match node {
                Node::Token(token) => out.push((child, token)),
                Node::Group(group) => group.walk(&child, out),
            }
        }
    }
}

/// A member of a group: a token or a nested group.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// A token.
    Token(Token),
    /// A group.
    Group(Group),
}

/// A whole token file: the root group and everything below it.
///
/// ```
/// use pub_design_system_tokens::{Color, Group, Path, Token, TokenSet, TokenType, Value};
///
/// let mut set = TokenSet::new();
/// let mut color = Group::new();
/// color.ty = Some(TokenType::Color);
/// color.insert_token("blue", Token::color(Color::from_rgb8(0x00, 0x66, 0xcc)))?;
/// color.insert_token("primary", Token::alias(TokenType::Color, Path::parse("color.blue")?))?;
/// set.root_mut().insert_group("color", color)?;
///
/// let primary = Path::parse("color.primary")?;
/// assert!(matches!(set.get(&primary).unwrap().value, Value::Alias(_)));
/// assert!(matches!(set.resolve(&primary)?, Value::Color(_)));
///
/// let json = set.to_json_string();
/// assert_eq!(TokenSet::from_json_str(&json)?, set);
/// # Ok::<(), pub_design_system_tokens::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TokenSet {
    root: Group,
}

impl TokenSet {
    /// An empty set.
    pub fn new() -> Self {
        Self::default()
    }

    /// A set around an existing root group.
    pub fn from_root(root: Group) -> Self {
        Self { root }
    }

    /// The root group.
    pub fn root(&self) -> &Group {
        &self.root
    }

    /// The root group, for changing it.
    pub fn root_mut(&mut self) -> &mut Group {
        &mut self.root
    }

    /// The token or group at `path`; the root group for the root path.
    pub fn node(&self, path: &Path) -> Option<NodeRef<'_>> {
        let mut group = &self.root;
        let segments = path.segments();
        let Some((last, ancestors)) = segments.split_last() else {
            return Some(NodeRef::Group(group));
        };
        for name in ancestors {
            match group.child(name)? {
                Node::Group(g) => group = g,
                Node::Token(_) => return None,
            }
        }
        match group.child(last)? {
            Node::Token(token) => Some(NodeRef::Token(token)),
            Node::Group(g) => Some(NodeRef::Group(g)),
        }
    }

    /// The token at `path`, as written (an alias stays an alias).
    pub fn get(&self, path: &Path) -> Option<&Token> {
        match self.node(path)? {
            NodeRef::Token(token) => Some(token),
            NodeRef::Group(_) => None,
        }
    }

    /// The value the token at `path` ends up with: its own, or the one at the end of its
    /// alias chain. Never returns [`Value::Alias`].
    ///
    /// Errors: [`Error::UnknownToken`] when `path` or a link of the chain names no token,
    /// [`Error::Cycle`] when the chain comes back on itself, [`Error::TypeMismatch`] when a
    /// link's type differs from the declared type of the token at `path`.
    pub fn resolve(&self, path: &Path) -> Result<&Value, Error> {
        let start = self
            .get(path)
            .ok_or_else(|| Error::UnknownToken { path: path.clone() })?;
        let mut visited = vec![path.clone()];
        let mut current = start;
        loop {
            let Value::Alias(target) = &current.value else {
                return Ok(&current.value);
            };
            if visited.contains(target) {
                return Err(Error::Cycle {
                    path: path.clone(),
                    through: visited.last().cloned().unwrap_or_else(Path::root),
                });
            }
            let next = self.get(target).ok_or_else(|| Error::UnknownToken {
                path: target.clone(),
            })?;
            if next.ty != start.ty {
                return Err(Error::TypeMismatch {
                    path: path.clone(),
                    expected: start.ty,
                    target: target.clone(),
                    found: next.ty,
                });
            }
            visited.push(target.clone());
            current = next;
        }
    }

    /// Every token with its path, depth first in name order (which is path order).
    pub fn tokens(&self) -> impl Iterator<Item = (Path, &Token)> {
        let mut out = Vec::new();
        self.root.walk(&Path::root(), &mut out);
        out.into_iter()
    }
}

/// What [`TokenSet::node`] finds at a path.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeRef<'a> {
    /// A token.
    Token(&'a Token),
    /// A group (the root, for the root path).
    Group(&'a Group),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_follow_the_format_rules() {
        for ok in ["a", "0", "brand-blue", "Brand Blue", "a$b", "ünïcödé"] {
            assert!(is_valid_name(ok), "{ok}");
        }
        for bad in ["", "$a", "a.b", "{a", "a}"] {
            assert!(!is_valid_name(bad), "{bad}");
        }
    }

    #[test]
    fn paths_join_split_and_print() {
        let p = Path::parse("a.b.c").unwrap();
        assert_eq!(p.name(), Some("c"));
        assert_eq!(p.parent(), Some(Path::parse("a.b").unwrap()));
        assert_eq!(Path::root().parent(), None);
        assert_eq!(Path::root().name(), None);
        assert!(Path::root().is_root());
        assert_eq!(Path::root().to_string(), "");
        assert_eq!(Path::root().join("x"), Path::parse("x").unwrap());
        assert!(matches!(
            Path::parse("a.$b"),
            Err(Error::InvalidName { parent, name }) if parent == Path::parse("a").unwrap() && name == "$b"
        ));
    }

    #[test]
    fn a_group_refuses_bad_and_duplicate_names_and_keeps_name_order() {
        let mut g = Group::new();
        g.insert_token("z", Token::dimension(Dimension::px(1.0)))
            .unwrap();
        g.insert_group("a", Group::new()).unwrap();
        assert!(
            matches!(g.insert_group("a", Group::new()), Err(Error::Duplicate { name, .. }) if name == "a")
        );
        assert!(
            matches!(g.insert_group("$a", Group::new()), Err(Error::InvalidName { name, .. }) if name == "$a")
        );
        let names: Vec<&str> = g.children().map(|(n, _)| n).collect();
        assert_eq!(names, ["a", "z"]);
        assert_eq!(g.len(), 2);
        assert!(g.child_mut("z").is_some());
        assert!(g.remove("z").is_some());
        assert!(g.remove("z").is_none());
        assert_eq!(g.len(), 1);
    }

    #[test]
    fn node_walks_groups_and_stops_at_tokens() {
        let mut set = TokenSet::new();
        let mut g = Group::new();
        g.insert_token("t", Token::dimension(Dimension::px(1.0)))
            .unwrap();
        set.root_mut().insert_group("g", g).unwrap();
        assert!(matches!(set.node(&Path::root()), Some(NodeRef::Group(_))));
        assert!(matches!(
            set.node(&Path::parse("g").unwrap()),
            Some(NodeRef::Group(_))
        ));
        assert!(matches!(
            set.node(&Path::parse("g.t").unwrap()),
            Some(NodeRef::Token(_))
        ));
        assert_eq!(set.node(&Path::parse("g.t.x").unwrap()), None);
        assert_eq!(set.node(&Path::parse("h").unwrap()), None);
        assert!(set.get(&Path::parse("g").unwrap()).is_none());
        assert_eq!(TokenSet::from_root(set.root().clone()), set);
    }

    #[test]
    fn a_self_alias_is_a_cycle_through_itself() {
        let mut set = TokenSet::new();
        let me = Path::parse("me").unwrap();
        set.root_mut()
            .insert_token("me", Token::alias(TokenType::Color, me.clone()))
            .unwrap();
        assert!(matches!(
            set.resolve(&me),
            Err(Error::Cycle { path, through }) if path == me && through == me
        ));
    }

    #[test]
    fn a_value_knows_its_type_except_an_alias() {
        assert_eq!(
            Value::Dimension(Dimension::px(1.0)).token_type(),
            Some(TokenType::Dimension)
        );
        assert_eq!(Value::Alias(Path::root()).token_type(), None);
        assert_eq!(TokenType::parse("color"), Some(TokenType::Color));
        assert_eq!(TokenType::parse("colour"), None);
        assert_eq!(TokenType::Dimension.to_string(), "dimension");
    }

    #[test]
    fn builders_set_the_metadata() {
        let t = Token::dimension(Dimension::px(1.0))
            .with_description("d")
            .with_deprecated(Deprecated::Flag(true))
            .with_extension("org.example", serde_json::json!(1));
        assert_eq!(t.description.as_deref(), Some("d"));
        assert_eq!(t.deprecated, Some(Deprecated::Flag(true)));
        assert_eq!(t.extensions["org.example"], serde_json::json!(1));
    }
}
