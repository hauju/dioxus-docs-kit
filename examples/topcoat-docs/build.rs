fn main() {
    // The repo's own docs, so this renders the same content as the Dioxus site.
    dioxus_docs_kit_build::DocsBuild::new("../../docs/_nav.json")
        .with_openapi("api-reference", "../../docs/api-reference/petstore.yaml")
        .generate();
    dioxus_docs_kit_build::BlogBuild::new("../../blog/_blog.json").generate();
}
