//! Prints the suite's tokens as DTCG JSON. Regenerate the checked-in export with
//! `cargo run --example export > tokens/suite.tokens.json` (from the crate directory).

fn main() {
    print!(
        "{}",
        pub_design_system_tokens::suite::tokens().to_json_string()
    );
}
