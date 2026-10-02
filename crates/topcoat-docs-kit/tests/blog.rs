//! Renders real build-time blog bundles through the Topcoat blog components.

use docs_kit_core::{BlogConfig, BlogRegistry};
use topcoat::{
    context::Cx,
    view::{View, ViewExt, view},
};
use topcoat_docs_kit::{
    blog_category_head, blog_category_page, blog_index_head, blog_index_page, blog_post_head,
    blog_post_page, blog_search_page,
};

const MANIFEST: &str = r#"{
    "authors": { "ada": { "name": "Ada", "url": "https://ada.example" } },
    "posts": ["featured", "rust-new", "rust-old", "misc"]
}"#;

const POSTS: &[(&str, &str)] = &[
    (
        "featured",
        "---\ntitle: \"Featured\"\ndate: \"2026-03-21\"\nauthor: \"ada\"\ntags: [\"news\"]\nfeatured: true\n---\nFeatured post\n",
    ),
    (
        "rust-new",
        "---\ntitle: \"Rust New\"\ndescription: \"All about widgets\"\ndate: \"2026-03-17\"\nauthor: \"ada\"\ntags: [\"rust\", \"web dev\"]\n---\n## Intro\n\nWidgets in Rust.\n",
    ),
    (
        "rust-old",
        "---\ntitle: \"Rust Old\"\ndate: \"2026-03-16\"\nauthor: \"ada\"\ntags: [\"rust\"]\n---\nOlder rust\n",
    ),
    (
        "misc",
        "---\ntitle: \"Misc\"\ndate: \"2026-03-15\"\nauthor: \"nobody\"\ntags: [\"misc\"]\n---\nMisc\n",
    ),
];

fn registry(per_page: usize) -> &'static BlogRegistry {
    let bundle = dioxus_docs_kit_build::blog_bundle_json(MANIFEST, POSTS)
        .expect("generate blog bundle")
        .leak();
    Box::leak(Box::new(
        BlogConfig::new(bundle)
            .with_posts_per_page(per_page)
            .build(),
    ))
}

fn category_registry() -> &'static BlogRegistry {
    let manifest = r#"{"authors":{}, "posts":["a","b"], "categories":{
        "rust": {"slug":"rust-lang", "title":"Rust programming", "description":"Learn Rust."}
    }}"#;
    let posts: &[(&str, &str)] = &[
        (
            "a",
            "---\ntitle: \"A\"\ndate: \"2026-01-02\"\nauthor: \"x\"\ntags: [\"rust\"]\n---\nA\n",
        ),
        (
            "b",
            "---\ntitle: \"B\"\ndate: \"2026-01-01\"\nauthor: \"x\"\ntags: [\"rust\"]\n---\nB\n",
        ),
    ];
    let bundle = dioxus_docs_kit_build::blog_bundle_json(manifest, posts)
        .expect("generate blog bundle")
        .leak();
    Box::leak(Box::new(
        BlogConfig::new(bundle)
            .with_posts_per_page(1)
            .with_category_base_path("/blog/categories")
            .build(),
    ))
}

async fn render(v: impl View) -> String {
    v.single().await.unwrap().render(&Cx::default())
}

#[tokio::test]
async fn index_shows_featured_tag_links_and_pagination() {
    let reg = registry(2);
    let cx = &Cx::default();
    let html = render(
        view! { cx => blog_index_page(registry: reg, base_path: "/blog", tag: None, page: 1) },
    )
    .await;

    assert!(html.contains(">Featured</span>"), "{html}");
    assert!(html.contains(r#"href="/blog/featured""#), "{html}");
    // Tag filter links, with the space in "web dev" encoded.
    assert!(html.contains(r#"href="/blog?tag=web%20dev""#), "{html}");
    // Three non-featured posts at two per page: page 2 link, no "Prev".
    assert!(html.contains(r#"href="/blog?page=2""#), "{html}");
    assert!(!html.contains(">Prev<"), "{html}");
}

#[tokio::test]
async fn tag_filter_narrows_the_list_and_marks_the_tag_active() {
    let reg = registry(9);
    let cx = &Cx::default();
    let html = render(view! { cx => blog_index_page(registry: reg, base_path: "/blog", tag: Some("rust"), page: 1) }).await;

    assert!(html.contains(">Blog: rust</h1>"), "{html}");
    assert!(html.contains(r#"href="/blog/rust-new""#), "{html}");
    assert!(!html.contains(r#"href="/blog/misc""#), "{html}");
    // No featured section while filtering; the active pill clears the filter.
    assert!(!html.contains(">Featured</span>"), "{html}");
    assert!(
        html.contains(r#"<a href="/blog" class="btn btn-sm btn-primary" aria-current="page">rust"#),
        "{html}"
    );
}

#[tokio::test]
async fn post_page_renders_meta_content_and_navigation() {
    let reg = registry(9);
    let cx = &Cx::default();
    let html =
        render(view! { cx => blog_post_page(registry: reg, base_path: "/blog", slug: "rust-new") })
            .await;

    assert!(html.contains(">Rust New</h1>"), "{html}");
    assert!(html.contains(r#"href="https://ada.example""#), "{html}");
    assert!(html.contains("dk-reading-progress"), "{html}");
    assert!(html.contains(r##"href="#intro""##), "{html}");
    assert!(html.contains(">Related Posts</h2>"), "{html}");
    assert!(html.contains(">Older</span>"), "{html}");
    assert!(html.contains(">Newer</span>"), "{html}");
}

#[tokio::test]
async fn unknown_post_renders_not_found() {
    let reg = registry(9);
    let cx = &Cx::default();
    let html =
        render(view! { cx => blog_post_page(registry: reg, base_path: "/blog", slug: "nope") })
            .await;
    assert!(html.contains("Post not found: nope"), "{html}");
}

#[tokio::test]
async fn post_head_has_article_tags_and_escaped_jsonld() {
    let reg = registry(9);
    let cx = &Cx::default();
    let html = render(view! { cx => blog_post_head(registry: reg, base_path: "/blog", slug: "rust-new", site_url: Some("https://ex.com")) }).await;

    assert!(html.contains("<title>Rust New</title>"), "{html}");
    assert!(
        html.contains(r#"<meta property="og:type" content="article">"#),
        "{html}"
    );
    assert!(!html.contains(r#"content="website""#), "{html}");
    assert!(
        html.contains(r#"<link rel="canonical" href="https://ex.com/blog/rust-new">"#),
        "{html}"
    );
    assert!(
        html.contains(r#"<meta property="article:tag" content="web dev">"#),
        "{html}"
    );
    assert!(
        html.contains(r#"<script type="application/ld+json">{"#),
        "{html}"
    );
}

#[tokio::test]
async fn index_head_titles_pages_and_tags() {
    let cx = &Cx::default();
    let html = render(view! { cx => blog_index_head(base_path: "/blog", tag: Some("rust"), page: 2, site_url: Some("https://ex.com")) }).await;
    assert!(
        html.contains("<title>Blog: rust — Page 2</title>"),
        "{html}"
    );
    assert!(
        html.contains(r#"href="https://ex.com/blog?tag=rust&amp;page=2""#),
        "{html}"
    );
}

#[tokio::test]
async fn categories_replace_tag_links_and_paginate_by_path() {
    let reg = category_registry();
    let cx = &Cx::default();
    let html = render(view! { cx => blog_category_page(registry: reg, base_path: "/blog", slug: "rust-lang", page: 1) }).await;

    assert!(html.contains(">Rust programming</h1>"), "{html}");
    assert!(
        html.contains(r#"href="/blog/categories/rust-lang/page/2""#),
        "{html}"
    );
    assert!(
        html.contains(r#"aria-current="page">Rust programming"#),
        "{html}"
    );

    let head = render(view! { cx => blog_category_head(registry: reg, slug: "rust-lang", page: 2, site_url: None) }).await;
    assert!(
        head.contains("<title>Rust programming — Page 2</title>"),
        "{head}"
    );

    let missing = render(view! { cx => blog_category_page(registry: reg, base_path: "/blog", slug: "rust-lang", page: 9) }).await;
    assert!(missing.contains("Category page not found"), "{missing}");
}

#[tokio::test]
async fn search_finds_posts_with_snippets() {
    let reg = registry(9);
    let cx = &Cx::default();
    let html = render(
        view! { cx => blog_search_page(registry: reg, base_path: "/blog", query: "widgets") },
    )
    .await;

    assert!(
        html.contains("1 result for \u{201c}widgets\u{201d}"),
        "{html}"
    );
    assert!(html.contains(r#"href="/blog/rust-new""#), "{html}");
    assert!(html.contains(r#"<mark class="dk-search-mark"#), "{html}");
    assert!(html.contains(r#"action="/blog/search""#), "{html}");
}
