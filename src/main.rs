use dioxus::prelude::*;
use dioxus_docs_kit::{
    DocsConfig, DocsContext, DocsLayout, DocsPageContent, DocsRegistry, DrawerOpen, highlight_code,
};
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{
    LdArrowRight, LdBookOpen, LdFileText, LdGithub, LdMenu, LdPackage, LdPalette, LdSearch,
    LdServer,
};
use std::collections::HashMap;
use std::sync::LazyLock;

// ============================================================================
// Documentation Registry
// ============================================================================

fn doc_content_map() -> HashMap<&'static str, &'static str> {
    include!(concat!(env!("OUT_DIR"), "/doc_content_generated.rs"))
}

static DOCS: LazyLock<DocsRegistry> = LazyLock::new(|| {
    DocsConfig::new(include_str!("../docs/_nav.json"), doc_content_map())
        .with_openapi(
            "api-reference",
            include_str!("../docs/api-reference/petstore.yaml"),
        )
        .with_default_path("getting-started/introduction")
        .with_theme_toggle("light", "dark", "dark")
        .build()
});

// ============================================================================
// Routes
// ============================================================================

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
        #[route("/")]
        Home {},
    #[end_layout]
    #[layout(MyDocsLayout)]
        #[redirect("/docs", || Route::DocsPage { slug: vec!["getting-started".into(), "introduction".into()] })]
        #[route("/docs/:..slug")]
        DocsPage { slug: Vec<String> },
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}

// ============================================================================
// Docs Layout Wrapper (glue code)
// ============================================================================

/// Thin wrapper that wires DocsContext + DocsRegistry into the library's DocsLayout.
#[component]
fn MyDocsLayout() -> Element {
    let nav = use_navigator();
    let route = use_route::<Route>();

    let current_path = use_memo(move || match route.clone() {
        Route::DocsPage { slug } => slug.join("/"),
        _ => String::new(),
    });

    let docs_ctx = DocsContext {
        current_path: current_path.into(),
        base_path: "/docs".into(),
        navigate: Callback::new(move |path: String| {
            let slug: Vec<String> = path.split('/').map(String::from).collect();
            nav.push(Route::DocsPage { slug });
        }),
    };

    use_context_provider(|| &*DOCS as &'static DocsRegistry);
    use_context_provider(|| docs_ctx);

    let search_open = use_signal(|| false);
    let mut drawer_open = use_signal(|| false);

    // Provide search_open and drawer_open before DocsLayout reads them
    use_context_provider(|| search_open);
    use_context_provider(|| DrawerOpen(drawer_open));

    rsx! {
        DocsLayout {
            header: rsx! {
                div { class: "navbar bg-base-200 border-b border-base-300 px-4 lg:px-8",
                    div { class: "flex-1 gap-2",
                        button {
                            class: "btn btn-ghost btn-sm btn-square lg:hidden docs-menu-btn",
                            onclick: move |_| drawer_open.toggle(),
                            Icon { class: "size-5", icon: LdMenu }
                        }
                        Link {
                            to: Route::Home {},
                            class: "text-xl font-semibold tracking-tight hover:opacity-80 transition-opacity",
                            "Dioxus Docs Kit"
                        }
                    }
                    div { class: "flex-none gap-1",
                        ul { class: "menu menu-horizontal gap-1 hidden lg:flex",
                            li {
                                Link {
                                    to: Route::Home {},
                                    class: "btn btn-ghost btn-sm rounded-lg font-medium",
                                    "Home"
                                }
                            }
                            li {
                                Link {
                                    to: Route::DocsPage { slug: vec!["getting-started".into(), "introduction".into()] },
                                    class: "btn btn-ghost btn-sm rounded-lg font-medium",
                                    "Docs"
                                }
                            }
                        }
                    }
                }
            },
            Outlet::<Route> {}
        }
    }
}

/// Documentation page that renders content.
#[component]
fn DocsPage(slug: Vec<String>) -> Element {
    rsx! {
        DocsPageContent { path: slug.join("/") }
    }
}

// ============================================================================
// LLMs.txt Server Functions
// ============================================================================

#[get("/llms.txt")]
async fn llms_txt() -> Result<String, ServerFnError> {
    Ok(DOCS.generate_llms_txt(
        "Dioxus Docs Kit",
        "A Dioxus-powered documentation framework with MDX rendering, OpenAPI reference pages, and full-text search.",
        "https://github.com/hauju/dioxus-docs-kit",
    ))
}

#[get("/llms-full.txt")]
async fn llms_full_txt() -> Result<String, ServerFnError> {
    Ok(DOCS.generate_llms_full_txt(
        "Dioxus Docs Kit",
        "A Dioxus-powered documentation framework with MDX rendering, OpenAPI reference pages, and full-text search.",
        "https://github.com/hauju/dioxus-docs-kit",
    ))
}

// ============================================================================
// App-specific pages (Navbar, Home)
// ============================================================================

/// Shared navbar component with DaisyUI styling
#[component]
fn Navbar() -> Element {
    rsx! {
        div { class: "navbar bg-base-200 border-b border-base-300 px-4 lg:px-8",
            div { class: "flex-1",
                Link {
                    to: Route::Home {},
                    class: "text-xl font-semibold tracking-tight hover:opacity-80 transition-opacity",
                    "Dioxus Docs Kit"
                }
            }
            div { class: "flex-none",
                ul { class: "menu menu-horizontal gap-1",
                    li {
                        Link {
                            to: Route::Home {},
                            class: "btn btn-ghost btn-sm rounded-lg font-medium",
                            "Home"
                        }
                    }
                    li {
                        Link {
                            to: Route::DocsPage { slug: vec!["getting-started".into(), "introduction".into()] },
                            class: "btn btn-ghost btn-sm rounded-lg font-medium",
                            "Docs"
                        }
                    }
                }
            }
        }

        main { class: "min-h-screen bg-base-100",
            Outlet::<Route> {}
        }
    }
}

/// Home page
#[component]
fn Home() -> Element {
    rsx! {
        div { class: "relative overflow-hidden",
            div { class: "hero-glow absolute inset-x-0 top-0 h-[600px] pointer-events-none" }
            HeroSection {}
            FeaturesSection {}
            CodeSection {}
            LandingFooter {}
        }
    }
}

// ── Hero ─────────────────────────────────────────────────────────────────────

#[component]
fn HeroSection() -> Element {
    rsx! {
        section { class: "relative px-4 pt-24 pb-20 lg:pt-32 lg:pb-28",
            div { class: "max-w-3xl mx-auto text-center",
                div { class: "animate-fade-up",
                    span { class: "badge badge-outline badge-primary text-xs tracking-wide",
                        "Open Source Documentation Template"
                    }
                }
                h1 { class: "mt-6 text-4xl sm:text-5xl lg:text-6xl font-bold tracking-tight leading-tight animate-fade-up-delay-1",
                    "Beautiful Documentation, "
                    span { class: "text-primary", "Zero Config" }
                }
                p { class: "mt-6 text-lg text-base-content/60 max-w-2xl mx-auto leading-relaxed animate-fade-up-delay-2",
                    "A Dioxus-powered documentation framework with MDX rendering, OpenAPI reference pages, full-text search, and dark/light themes — all embedded at compile time."
                }
                div { class: "mt-10 flex flex-wrap items-center justify-center gap-4 animate-fade-up-delay-3",
                    Link {
                        to: Route::DocsPage { slug: vec!["getting-started".into(), "introduction".into()] },
                        class: "btn btn-primary gap-2",
                        "Get Started"
                        Icon { class: "size-4", icon: LdArrowRight }
                    }
                    a {
                        href: "https://github.com/hauju/dioxus-docs-kit",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "btn btn-ghost gap-2",
                        Icon { class: "size-4", icon: LdGithub }
                        "GitHub"
                    }
                }
            }
        }
    }
}

// ── Features ─────────────────────────────────────────────────────────────────

#[component]
fn FeatureCard(icon: Element, title: &'static str, description: &'static str) -> Element {
    rsx! {
        div { class: "group flex flex-col gap-3 p-6 rounded-xl border border-base-300 bg-base-200/30 hover:border-primary/30 transition-all duration-200",
            div { class: "flex items-center justify-center size-10 rounded-lg bg-primary/10 text-primary group-hover:bg-primary/20 transition-colors",
                {icon}
            }
            h3 { class: "font-semibold text-base-content", "{title}" }
            p { class: "text-sm text-base-content/60 leading-relaxed", "{description}" }
        }
    }
}

#[component]
fn FeaturesSection() -> Element {
    rsx! {
        section { class: "px-4 py-20",
            div { class: "max-w-5xl mx-auto",
                div { class: "text-center mb-12",
                    h2 { class: "text-3xl font-bold tracking-tight", "Everything You Need" }
                    p { class: "mt-3 text-base-content/60",
                        "A complete toolkit for building beautiful documentation sites."
                    }
                }
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                    FeatureCard {
                        icon: rsx! { Icon { class: "size-5", icon: LdFileText } },
                        title: "MDX Documents",
                        description: "Write docs in MDX with full component support, syntax highlighting, and table of contents generation."
                    }
                    FeatureCard {
                        icon: rsx! { Icon { class: "size-5", icon: LdBookOpen } },
                        title: "OpenAPI Reference",
                        description: "Auto-generate interactive API reference pages from your OpenAPI/Swagger specifications."
                    }
                    FeatureCard {
                        icon: rsx! { Icon { class: "size-5", icon: LdSearch } },
                        title: "Full-Text Search",
                        description: "Instant search across all documentation with keyboard shortcuts and highlighted results."
                    }
                    FeatureCard {
                        icon: rsx! { Icon { class: "size-5", icon: LdPalette } },
                        title: "Dark & Light Themes",
                        description: "Built-in theme support with DaisyUI. Seamless switching with persisted preferences."
                    }
                    FeatureCard {
                        icon: rsx! { Icon { class: "size-5", icon: LdPackage } },
                        title: "Compile-Time Embedding",
                        description: "All content is embedded at build time — no file I/O at runtime, instant page loads."
                    }
                    FeatureCard {
                        icon: rsx! { Icon { class: "size-5", icon: LdServer } },
                        title: "Dioxus Fullstack",
                        description: "Server-side rendering, hydration, and server functions out of the box with Dioxus 0.7."
                    }
                }
            }
        }
    }
}

// ── Code Example ─────────────────────────────────────────────────────────────

const CODE_SNIPPET: &str = r#"use dioxus_docs_kit::{DocsConfig, DocsRegistry};
use std::sync::LazyLock;

static DOCS: LazyLock<DocsRegistry> = LazyLock::new(|| {
    DocsConfig::new(
        include_str!("../docs/_nav.json"),
        doc_content_map(),
    )
    .with_openapi(
        "api-reference",
        include_str!("../docs/api-reference/petstore.yaml"),
    )
    .with_default_path("getting-started/introduction")
    .with_theme_toggle("light", "dark", "dark")
    .build()
});"#;

#[component]
fn CodeSection() -> Element {
    rsx! {
        section { class: "px-4 py-20",
            div { class: "max-w-3xl mx-auto",
                div { class: "text-center mb-12",
                    h2 { class: "text-3xl font-bold tracking-tight", "Simple Integration" }
                    p { class: "mt-3 text-base-content/60",
                        "Set up your documentation site in about 50 lines of glue code."
                    }
                }
                div { class: "rounded-xl border border-base-300 bg-base-200 overflow-hidden",
                    div { class: "flex items-center gap-2 px-4 py-3 border-b border-base-300",
                        div { class: "flex gap-1.5",
                            div { class: "size-3 rounded-full bg-error/60" }
                            div { class: "size-3 rounded-full bg-warning/60" }
                            div { class: "size-3 rounded-full bg-success/60" }
                        }
                        span { class: "text-xs text-base-content/40 ml-2 font-mono", "main.rs" }
                    }
                    pre { class: "p-4 overflow-x-auto text-sm font-mono leading-relaxed text-base-content/80",
                        code {
                            dangerous_inner_html: highlight_code(CODE_SNIPPET, Some("rust")),
                        }
                    }
                }
            }
        }
    }
}

// ── Footer ───────────────────────────────────────────────────────────────────

#[component]
fn LandingFooter() -> Element {
    rsx! {
        footer { class: "border-t border-base-300 px-4 py-8",
            div { class: "max-w-5xl mx-auto flex flex-col sm:flex-row items-center justify-between gap-4 text-sm text-base-content/50",
                span { "Dioxus Docs Kit · Built with Dioxus" }
                a {
                    href: "https://github.com/hauju/dioxus-docs-kit",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "flex items-center gap-1.5 hover:text-base-content transition-colors",
                    Icon { class: "size-4", icon: LdGithub }
                    "GitHub"
                }
            }
        }
    }
}
