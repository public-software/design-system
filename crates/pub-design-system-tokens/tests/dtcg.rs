//! The DTCG round trip and the suite's own token set, pinned from the outside.
//!
//! Every test here names a rule of the Design Tokens Format Module 2025.10 or of the Color
//! Module 2025.10 (both listed in the repository's `PROVENANCE.md`) or a promise the crate
//! makes about the suite's tokens.

use pub_design_system_tokens::{
    Color, ColorSpace, Component, Deprecated, Dimension, Error, Group, Path, Token, TokenSet,
    TokenType, Unit, Value, suite,
};

fn parse(json: &str) -> Result<TokenSet, Error> {
    TokenSet::from_json_str(json)
}

fn path(s: &str) -> Path {
    Path::parse(s).expect("a well-formed path")
}

const BLUE: &str = r##"{"colorSpace": "srgb", "components": [0, 0.4, 0.8], "hex": "#0066cc"}"##;

// ---- types -------------------------------------------------------------------------------

#[test]
fn a_token_with_an_unknown_type_is_rejected() {
    let json = format!(r##"{{ "x": {{ "$type": "flavor", "$value": {BLUE} }} }}"##);
    match parse(&json) {
        Err(Error::UnknownType { path: p, ty }) => {
            assert_eq!(p, path("x"));
            assert_eq!(ty, "flavor");
        }
        other => panic!("expected UnknownType, got {other:?}"),
    }
}

#[test]
fn a_type_the_slice_does_not_carry_is_rejected_the_same_way() {
    // `gradient` is a DTCG type; this slice reads colour and dimension only.
    let json = r##"{ "g": { "$type": "gradient", "$value": [] } }"##;
    assert!(matches!(parse(json), Err(Error::UnknownType { ty, .. }) if ty == "gradient"));
}

#[test]
fn a_token_with_no_resolvable_type_is_rejected() {
    // The format module: a token with no explicit or inherited type is invalid; tools must
    // not infer a type from the value.
    let json = format!(r##"{{ "color": {{ "blue": {{ "$value": {BLUE} }} }} }}"##);
    match parse(&json) {
        Err(Error::MissingType { path: p }) => assert_eq!(p, path("color.blue")),
        other => panic!("expected MissingType, got {other:?}"),
    }
}

#[test]
fn a_group_type_is_inherited_by_nested_tokens() {
    let json = format!(
        r##"{{ "color": {{ "$type": "color", "brand": {{ "blue": {{ "$value": {BLUE} }} }} }} }}"##
    );
    let set = parse(&json).unwrap();
    let token = set
        .get(&path("color.brand.blue"))
        .expect("the token exists");
    assert_eq!(token.ty, TokenType::Color);
    assert!(matches!(token.value, Value::Color(_)));
}

#[test]
fn a_token_type_beats_the_group_type() {
    let json = r##"{ "color": { "$type": "color",
        "gap": { "$type": "dimension", "$value": { "value": 4, "unit": "px" } } } }"##;
    let set = parse(json).unwrap();
    let token = set.get(&path("color.gap")).unwrap();
    assert_eq!(token.ty, TokenType::Dimension);
    assert_eq!(token.value, Value::Dimension(Dimension::px(4.0)));
}

#[test]
fn an_unknown_type_on_a_group_is_rejected() {
    let json = r##"{ "g": { "$type": "sound", "a": { "$value": 1 } } }"##;
    assert!(matches!(parse(json), Err(Error::UnknownType { ty, .. }) if ty == "sound"));
}

#[test]
fn an_object_without_a_value_is_a_group_even_when_typed_and_empty() {
    let json = r##"{ "g": { "$type": "color", "$description": "nothing yet" } }"##;
    let set = parse(json).unwrap();
    let Some(pub_design_system_tokens::Node::Group(group)) = set.root().child("g") else {
        panic!("g is a group");
    };
    assert_eq!(group.ty, Some(TokenType::Color));
    assert_eq!(group.description.as_deref(), Some("nothing yet"));
    assert!(group.is_empty());
}

// ---- names -------------------------------------------------------------------------------

#[test]
fn a_name_starting_with_a_dollar_that_is_not_a_property_is_rejected() {
    let json = format!(r##"{{ "$flavor": {{ "$type": "color", "$value": {BLUE} }} }}"##);
    assert!(matches!(parse(&json), Err(Error::InvalidName { name, .. }) if name == "$flavor"));
}

#[test]
fn a_name_containing_a_dot_or_a_brace_is_rejected() {
    for bad in ["a.b", "a{b", "a}b", ""] {
        let json = format!(
            r##"{{ "{bad}": {{ "$type": "dimension", "$value": {{ "value": 1, "unit": "px" }} }} }}"##
        );
        assert!(
            matches!(parse(&json), Err(Error::InvalidName { name, .. }) if name == bad),
            "{bad:?}"
        );
    }
    let mut group = Group::new();
    assert!(matches!(
        group.insert_token("a.b", Token::dimension(Dimension::px(1.0))),
        Err(Error::InvalidName { .. })
    ));
}

#[test]
fn a_path_is_dot_separated_names() {
    assert_eq!(path("a.b.c").to_string(), "a.b.c");
    assert_eq!(path("a.b.c").segments(), ["a", "b", "c"]);
    assert!(matches!(Path::parse(""), Err(Error::InvalidName { .. })));
    assert!(matches!(
        Path::parse("a..b"),
        Err(Error::InvalidName { .. })
    ));
    assert!(matches!(
        Path::parse("a.{b}"),
        Err(Error::InvalidName { .. })
    ));
}

// ---- colour values (Color Module 2025.10) ------------------------------------------------

#[test]
fn a_colour_has_a_colour_space_three_components_and_optional_alpha_and_hex() {
    let json = r##"{ "c": { "$type": "color", "$value":
        { "colorSpace": "oklch", "components": [0.7, 0.15, 250], "alpha": 0.5 } } }"##;
    let set = parse(json).unwrap();
    let Value::Color(c) = &set.get(&path("c")).unwrap().value else {
        panic!("a colour")
    };
    assert_eq!(c.color_space, ColorSpace::Oklch);
    assert_eq!(
        c.components,
        [
            Component::Value(0.7),
            Component::Value(0.15),
            Component::Value(250.0)
        ]
    );
    assert_eq!(c.alpha, Some(0.5));
    assert_eq!(c.hex, None);
}

#[test]
fn every_colour_space_of_the_colour_module_is_read() {
    for (name, space) in [
        ("srgb", ColorSpace::Srgb),
        ("srgb-linear", ColorSpace::SrgbLinear),
        ("hsl", ColorSpace::Hsl),
        ("hwb", ColorSpace::Hwb),
        ("lab", ColorSpace::Lab),
        ("lch", ColorSpace::Lch),
        ("oklab", ColorSpace::Oklab),
        ("oklch", ColorSpace::Oklch),
        ("display-p3", ColorSpace::DisplayP3),
        ("a98-rgb", ColorSpace::A98Rgb),
        ("prophoto-rgb", ColorSpace::ProphotoRgb),
        ("rec2020", ColorSpace::Rec2020),
        ("xyz-d65", ColorSpace::XyzD65),
        ("xyz-d50", ColorSpace::XyzD50),
    ] {
        let json = format!(
            r##"{{ "c": {{ "$type": "color", "$value": {{ "colorSpace": "{name}", "components": [0, 0, 0] }} }} }}"##
        );
        let set = parse(&json).unwrap_or_else(|e| panic!("{name}: {e}"));
        let Value::Color(c) = &set.get(&path("c")).unwrap().value else {
            panic!()
        };
        assert_eq!(c.color_space, space, "{name}");
        assert_eq!(space.as_str(), name);
    }
}

#[test]
fn a_none_component_is_read_and_written_as_the_keyword() {
    let json = r##"{ "c": { "$type": "color", "$value": { "colorSpace": "hsl", "components": ["none", 0, 100] } } }"##;
    let set = parse(json).unwrap();
    let Value::Color(c) = &set.get(&path("c")).unwrap().value else {
        panic!()
    };
    assert_eq!(c.components[0], Component::None);
    let out = set.to_json_value();
    assert_eq!(
        out["c"]["$value"]["components"][0],
        serde_json::json!("none")
    );
}

#[test]
fn a_malformed_colour_is_rejected_with_its_path() {
    let cases = [
        (
            r##"{ "colorSpace": "cmyk", "components": [0, 0, 0] }"##,
            "cmyk",
        ),
        (
            r##"{ "colorSpace": "srgb", "components": [0, 0] }"##,
            "three",
        ),
        (
            r##"{ "colorSpace": "srgb", "components": [0, 0, 0, 0] }"##,
            "three",
        ),
        (
            r##"{ "colorSpace": "srgb", "components": [0, "some", 0] }"##,
            "none",
        ),
        (
            r##"{ "colorSpace": "srgb", "components": [0, 0, 0], "alpha": 1.5 }"##,
            "alpha",
        ),
        (
            r##"{ "colorSpace": "srgb", "components": [0, 0, 0], "hex": "#fff" }"##,
            "hex",
        ),
        (
            r##"{ "colorSpace": "srgb", "components": [0, 0, 0], "hex": "0000ff" }"##,
            "hex",
        ),
        (
            r##"{ "colorSpace": "srgb", "components": [0, 0, 0], "hex": "#gg0000" }"##,
            "hex",
        ),
        (r##"{ "components": [0, 0, 0] }"##, "colorSpace"),
        (r##"{ "colorSpace": "srgb" }"##, "components"),
        (r##""#ff0000""##, "object"),
    ];
    for (value, word) in cases {
        let json = format!(r##"{{ "g": {{ "c": {{ "$type": "color", "$value": {value} }} }} }}"##);
        match parse(&json) {
            Err(Error::InvalidValue { path: p, reason }) => {
                assert_eq!(p, path("g.c"), "{value}");
                assert!(reason.contains(word), "{value}: {reason}");
            }
            other => panic!("{value}: expected InvalidValue, got {other:?}"),
        }
    }
}

#[test]
fn a_hex_fallback_is_normalized_to_lower_case_and_agrees_with_rgb8() {
    let json = r##"{ "c": { "$type": "color", "$value": { "colorSpace": "srgb", "components": [1, 0, 0], "hex": "#FF0000" } } }"##;
    let set = parse(json).unwrap();
    assert_eq!(
        set.to_json_value()["c"]["$value"]["hex"],
        serde_json::json!("#ff0000")
    );

    let c = Color::from_rgb8(0x00, 0x66, 0xcc);
    assert_eq!(c.color_space, ColorSpace::Srgb);
    assert_eq!(
        c.components,
        [
            Component::Value(0.0),
            Component::Value(0.4),
            Component::Value(0.8)
        ]
    );
    assert_eq!(c.hex.map(|h| h.to_string()), Some("#0066cc".to_owned()));
    assert_eq!(c.alpha, None);
}

// ---- dimension values ----------------------------------------------------------------------

#[test]
fn a_dimension_is_a_number_with_a_px_or_rem_unit() {
    let json = r##"{ "s": { "$type": "dimension", "a": { "$value": { "value": 0.25, "unit": "rem" } },
                                                  "b": { "$value": { "value": 4, "unit": "px" } } } }"##;
    let set = parse(json).unwrap();
    assert_eq!(
        set.get(&path("s.a")).unwrap().value,
        Value::Dimension(Dimension {
            value: 0.25,
            unit: Unit::Rem
        })
    );
    assert_eq!(
        set.get(&path("s.b")).unwrap().value,
        Value::Dimension(Dimension::px(4.0))
    );
}

#[test]
fn a_malformed_dimension_is_rejected() {
    for (value, word) in [
        (r##"{ "value": 1, "unit": "em" }"##, "em"),
        (r##"{ "value": "1", "unit": "px" }"##, "number"),
        (r##"{ "unit": "px" }"##, "value"),
        (r##"{ "value": 1 }"##, "unit"),
        (r##""4px""##, "object"),
    ] {
        let json = format!(r##"{{ "d": {{ "$type": "dimension", "$value": {value} }} }}"##);
        match parse(&json) {
            Err(Error::InvalidValue { reason, .. }) => {
                assert!(reason.contains(word), "{value}: {reason}")
            }
            other => panic!("{value}: expected InvalidValue, got {other:?}"),
        }
    }
}

// ---- aliases -------------------------------------------------------------------------------

#[test]
fn an_alias_is_kept_as_a_reference_and_resolves_to_the_target_value() {
    let json = format!(
        r##"{{ "colors": {{ "blue": {{ "$type": "color", "$value": {BLUE} }} }},
             "semantic": {{ "primary": {{ "$type": "color", "$value": "{{colors.blue}}" }} }} }}"##
    );
    let set = parse(&json).unwrap();
    let primary = set.get(&path("semantic.primary")).unwrap();
    assert_eq!(primary.value, Value::Alias(path("colors.blue")));
    let resolved = set.resolve(&path("semantic.primary")).unwrap();
    assert_eq!(resolved, &set.get(&path("colors.blue")).unwrap().value);
    assert!(matches!(resolved, Value::Color(_)));
    // Writing keeps the reference, it does not inline the target.
    assert_eq!(
        set.to_json_value()["semantic"]["primary"]["$value"],
        serde_json::json!("{colors.blue}")
    );
}

#[test]
fn an_alias_chain_is_followed_and_a_cycle_is_an_error() {
    let json = format!(
        r##"{{ "$type": "color",
             "a": {{ "$value": "{{b}}" }}, "b": {{ "$value": "{{c}}" }}, "c": {{ "$value": {BLUE} }},
             "x": {{ "$value": "{{y}}" }}, "y": {{ "$value": "{{x}}" }} }}"##
    );
    let set = parse(&json).unwrap();
    assert!(matches!(set.resolve(&path("a")).unwrap(), Value::Color(_)));
    match set.resolve(&path("x")) {
        Err(Error::Cycle { path: p, through }) => {
            assert_eq!(p, path("x"));
            assert_eq!(through, path("y"));
        }
        other => panic!("expected Cycle, got {other:?}"),
    }
}

#[test]
fn an_alias_to_a_missing_token_or_to_another_type_is_an_error() {
    let json = format!(
        r##"{{ "blue": {{ "$type": "color", "$value": {BLUE} }},
             "gap": {{ "$type": "dimension", "$value": "{{blue}}" }},
             "gone": {{ "$type": "color", "$value": "{{nowhere}}" }} }}"##
    );
    let set = parse(&json).unwrap();
    match set.resolve(&path("gap")) {
        Err(Error::TypeMismatch {
            path: p,
            expected,
            target,
            found,
        }) => {
            assert_eq!(p, path("gap"));
            assert_eq!(expected, TokenType::Dimension);
            assert_eq!(target, path("blue"));
            assert_eq!(found, TokenType::Color);
        }
        other => panic!("expected TypeMismatch, got {other:?}"),
    }
    assert!(
        matches!(set.resolve(&path("gone")), Err(Error::UnknownToken { path: p }) if p == path("nowhere"))
    );
    assert!(
        matches!(set.resolve(&path("nowhere")), Err(Error::UnknownToken { path: p }) if p == path("nowhere"))
    );
}

#[test]
fn a_malformed_reference_is_rejected_at_read_time() {
    for value in [
        r##""{a.b""##,
        r##""a.b}""##,
        r##""{}""##,
        r##""{a..b}""##,
        r##""plain""##,
    ] {
        let json = format!(r##"{{ "t": {{ "$type": "color", "$value": {value} }} }}"##);
        assert!(
            matches!(parse(&json), Err(Error::InvalidValue { .. })),
            "{value}"
        );
    }
}

// ---- metadata ------------------------------------------------------------------------------

#[test]
fn description_deprecated_and_extensions_survive_the_round_trip() {
    let json = format!(
        r##"{{ "g": {{ "$type": "color", "$description": "brand", "$deprecated": "use semantic.*",
                     "$extensions": {{ "dev.publicsoftware.tokens": {{ "since": 1 }} }},
               "old": {{ "$value": {BLUE}, "$deprecated": true, "$description": "the 2025 blue",
                        "$extensions": {{ "org.example.tool": [1, 2] }} }},
               "kept": {{ "$value": {BLUE}, "$deprecated": false }} }} }}"##
    );
    let set = parse(&json).unwrap();
    let old = set.get(&path("g.old")).unwrap();
    assert_eq!(old.description.as_deref(), Some("the 2025 blue"));
    assert_eq!(old.deprecated, Some(Deprecated::Flag(true)));
    assert_eq!(
        old.extensions["org.example.tool"],
        serde_json::json!([1, 2])
    );
    assert_eq!(
        set.get(&path("g.kept")).unwrap().deprecated,
        Some(Deprecated::Flag(false))
    );
    let Some(pub_design_system_tokens::Node::Group(g)) = set.root().child("g") else {
        panic!()
    };
    assert_eq!(
        g.deprecated,
        Some(Deprecated::Reason("use semantic.*".to_owned()))
    );
    assert_eq!(
        g.extensions["dev.publicsoftware.tokens"]["since"],
        serde_json::json!(1)
    );

    let again = parse(&set.to_json_string()).unwrap();
    assert_eq!(again, set);
    let out = set.to_json_value();
    assert_eq!(out["g"]["$deprecated"], serde_json::json!("use semantic.*"));
    assert_eq!(out["g"]["old"]["$deprecated"], serde_json::json!(true));
    assert!(
        out["g"]["old"].get("$type").is_none(),
        "an inherited type is not repeated"
    );
}

#[test]
fn an_unknown_dollar_property_on_a_token_is_rejected() {
    let json = format!(r##"{{ "t": {{ "$type": "color", "$value": {BLUE}, "$colour": 1 }} }}"##);
    assert!(matches!(parse(&json), Err(Error::InvalidName { name, .. }) if name == "$colour"));
}

// ---- the round trip -------------------------------------------------------------------------

#[test]
fn export_then_import_is_identity() {
    let json = format!(
        r##"{{ "$description": "root", "$type": "dimension",
             "space": {{ "1": {{ "$value": {{ "value": 0.25, "unit": "rem" }} }}, "2": {{ "$value": {{ "value": 8, "unit": "px" }} }} }},
             "color": {{ "$type": "color", "blue": {{ "$value": {BLUE} }},
                          "t": {{ "$value": {{ "colorSpace": "display-p3", "components": [0.1, 0.2, 0.3], "alpha": 0.25 }} }},
                          "alias": {{ "$value": "{{color.blue}}" }} }},
             "root-token": {{ "$type": "color", "$value": {{ "colorSpace": "hwb", "components": [120, "none", 0] }} }} }}"##
    );
    let first = parse(&json).unwrap();
    let text = first.to_json_string();
    let second = parse(&text).unwrap();
    assert_eq!(first, second);
    assert_eq!(second.to_json_string(), text, "the export is stable");
    assert!(text.ends_with('\n'));
    assert_eq!(
        first.to_json_value(),
        serde_json::from_str::<serde_json::Value>(&text).unwrap()
    );
}

#[test]
fn a_syntax_error_is_reported_as_json() {
    assert!(matches!(parse("{"), Err(Error::Json(_))));
    assert!(
        matches!(parse("[]"), Err(Error::InvalidValue { .. })),
        "the root must be an object"
    );
}

#[test]
fn tokens_are_listed_in_path_order_with_their_paths() {
    let json = format!(
        r##"{{ "b": {{ "$type": "color", "z": {{ "$value": {BLUE} }}, "a": {{ "$value": {BLUE} }} }},
             "a": {{ "$type": "dimension", "$value": {{ "value": 1, "unit": "px" }} }} }}"##
    );
    let set = parse(&json).unwrap();
    let paths: Vec<String> = set.tokens().map(|(p, _)| p.to_string()).collect();
    assert_eq!(paths, ["a", "b.a", "b.z"]);
    assert_eq!(set.tokens().count(), 3);
}

#[test]
fn a_set_is_built_in_rust_the_same_way_it_is_read() {
    let mut set = TokenSet::new();
    let mut color = Group::new();
    color.ty = Some(TokenType::Color);
    color
        .insert_token("blue", Token::color(Color::from_rgb8(0x00, 0x66, 0xcc)))
        .unwrap();
    color
        .insert_token(
            "primary",
            Token::alias(TokenType::Color, path("color.blue")),
        )
        .unwrap();
    set.root_mut().insert_group("color", color).unwrap();
    let mut space = Group::new();
    space
        .insert_token(
            "1",
            Token::dimension(Dimension::rem(0.25)).with_description("one step"),
        )
        .unwrap();
    set.root_mut().insert_group("space", space).unwrap();

    let json = format!(
        r##"{{ "color": {{ "$type": "color", "blue": {{ "$value": {BLUE} }}, "primary": {{ "$value": "{{color.blue}}" }} }},
             "space": {{ "1": {{ "$type": "dimension", "$value": {{ "value": 0.25, "unit": "rem" }}, "$description": "one step" }} }} }}"##
    );
    assert_eq!(set, parse(&json).unwrap());
    assert!(matches!(
        set.root_mut().insert_group("color", Group::new()),
        Err(Error::Duplicate { .. })
    ));
}

// ---- the suite's tokens ---------------------------------------------------------------------

#[test]
fn the_suite_set_round_trips_and_every_alias_resolves() {
    let set = suite::tokens();
    let again = parse(&set.to_json_string()).unwrap();
    assert_eq!(again, set);
    for (p, token) in set.tokens() {
        let value = set.resolve(&p).unwrap_or_else(|e| panic!("{p}: {e}"));
        match token.ty {
            TokenType::Color => assert!(matches!(value, Value::Color(_)), "{p}"),
            TokenType::Dimension => assert!(matches!(value, Value::Dimension(_)), "{p}"),
        }
    }
}

#[test]
fn the_suite_colours_carry_an_srgb_hex_and_the_spacing_is_a_quarter_rem_scale() {
    let set = suite::tokens();
    assert_eq!(
        set.resolve(&path("color.accent.500")).unwrap(),
        &Value::Color(suite::color::ACCENT_500)
    );
    assert_eq!(
        set.resolve(&path("color.neutral.0")).unwrap(),
        &Value::Color(Color::from_rgb8(0xff, 0xff, 0xff))
    );
    for (p, token) in set.tokens() {
        if let Value::Color(c) = &token.value {
            assert_eq!(c.color_space, ColorSpace::Srgb, "{p}");
            assert!(c.hex.is_some(), "{p} has a hex fallback");
        }
        if let Value::Dimension(d) = &token.value {
            assert_eq!(d.unit, Unit::Rem, "{p}");
            let steps = d.value / suite::space::STEP_REM;
            assert_eq!(steps.fract(), 0.0, "{p} is on the grid");
        }
    }
    assert_eq!(
        set.resolve(&path("space.4")).unwrap(),
        &Value::Dimension(Dimension::rem(1.0))
    );
    assert_eq!(suite::space::SPACE_4, Dimension::rem(1.0));
    assert_eq!(suite::space::STEP_REM, 0.25);
    assert!(set.tokens().count() >= 40, "{}", set.tokens().count());
}

#[test]
fn the_exported_file_matches_the_rust_constants() {
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tokens/suite.tokens.json");
    let on_disk = std::fs::read_to_string(&file).expect("tokens/suite.tokens.json is checked in");
    let on_disk = on_disk.replace("\r\n", "\n");
    assert_eq!(
        on_disk,
        suite::tokens().to_json_string(),
        "regenerate with `cargo run --example export`"
    );
    assert_eq!(parse(&on_disk).unwrap(), suite::tokens());
}
