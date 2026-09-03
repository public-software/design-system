# pub-design-system-tokens

The `tokens` library of [design-system](https://github.com/public-software/design-system), part of Public Software. Kind: `lib`.

The design tokens of the suite: the one look and feel as typed Rust and as the JSON interchange every tool
reads, the [Design Tokens Format Module 2025.10](https://www.designtokens.org/tr/drafts/format/) of the W3C
Design Tokens Community Group with its [Color Module](https://www.designtokens.org/tr/drafts/color/). A
`TokenSet` is read from that JSON, written back canonically, built in Rust, and queried by path: `get` gives a
token as written, `resolve` follows its aliases to a value.

This first slice carries the `color` type (the fourteen colour spaces of the Color Module, `none` components,
`alpha`, the six-digit `hex` fallback) and the `dimension` type (`px`, `rem`); tokens and groups with
`$description`, `$deprecated` and `$extensions`; `$type` inheritance from groups; `{group.token}` aliases with
cycle and type-mismatch detection. Reading is strict: any other `$type` is an error, a token with no resolvable
type is an error (the format forbids inferring one), a value of the wrong shape is an error with its path.
Writing is canonical, so `import(export(set)) == set`. The suite's own tokens, `suite::tokens()`, are the
palette (`color.neutral.*`, `color.accent.*`, `color.success.*`, `color.warning.*`, `color.danger.*`, sRGB from
8-bit channels so `hex` and the components agree), the roles that alias into it (`color.text.*`,
`color.surface.*`, `color.border.*`, `color.action.*`) and the spacing scale (`space.<n>` = `n × 0.25 rem`);
`tokens/suite.tokens.json` is their export, kept current by a test. The one dependency is `serde_json`, with
`float_roundtrip` so a component read back is the number that was written.

```rust
use pub_design_system_tokens::{Path, TokenSet, Value, suite};

let set = suite::tokens();
let primary = set.resolve(&Path::parse("color.action.primary")?)?;
assert_eq!(primary, &Value::Color(suite::color::ACCENT_500));

let json = set.to_json_string();                      // the DTCG file
assert_eq!(TokenSet::from_json_str(&json)?, set);      // and back
```

```sh
cargo nextest run -p pub-design-system-tokens
cargo run --example export > tokens/suite.tokens.json   # after changing the constants
```

Its entry in the repository's `CATALOG.toml`:

```toml
[[component]]
crate     = "pub-design-system-tokens"
kind      = "lib"
ledger    = "tokens"
readiness = "partial"
effort    = 2
specs     = ["dtcg-format-2025.10", "dtcg-color-2025.10"]
provides  = []
requires  = []
```
