//! Compile the README's integration example with the same public API a consumer uses.
#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/readme_integration.rs"));

#[test]
fn documented_setup_resolves_the_default_page() {
    assert!(DOCS.get_parsed_doc(&DOCS.default_path).is_some());
}
