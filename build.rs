fn main() {
    use dioxus_docs_kit_build::ValidationMode;
    dioxus_docs_kit_build::generate_content_map_with_validation(
        "docs/_nav.json",
        ValidationMode::Strict,
    );
    dioxus_docs_kit_build::generate_blog_content_map_with_validation(
        "blog/_blog.json",
        ValidationMode::Strict,
    );

    // Compile the actual onboarding snippet, not a separately maintained copy.
    println!("cargo:rerun-if-changed=README.md");
    let readme = std::fs::read_to_string("README.md").expect("read README");
    let integration = readme
        .split_once("### 4. Wire up routes and layout")
        .expect("README integration section")
        .1
        .split_once("```rust\n")
        .expect("README integration Rust fence")
        .1
        .split_once("\n```")
        .expect("README integration closing fence")
        .0;
    let out_dir = std::env::var_os("OUT_DIR").expect("Cargo OUT_DIR");
    std::fs::write(
        std::path::PathBuf::from(out_dir).join("readme_integration.rs"),
        integration,
    )
    .expect("write README integration fixture");
}
