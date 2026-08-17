use dioxus::prelude::*;
use dioxus_docs_kit::{
    BlogConfig, BlogContext, BlogLayout, BlogList, BlogPostView, BlogRegistry, BlogThemeToggle,
    Code, CodeTheme, DocsConfig, DocsContext, DocsLayout, DocsPageContent, DocsRegistry, Language,
    SearchButton, SearchModal, SourceCode, Theme, ThemeToggle, use_blog_providers,
    use_docs_context, use_docs_providers,
};
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{
    LdArrowRight, LdBookOpen, LdExternalLink, LdFileText, LdGithub, LdLock, LdMenu, LdPackage,
    LdPalette, LdSearch, LdServer,
};
use std::sync::LazyLock;

// The wasm32 `stderr` shim now lives in the `dioxus-docs-kit` library
// (`crates/dioxus-docs-kit/src/lib.rs`) so library consumers get it too; this
// binary picks it up transitively.

// ============================================================================
// Documentation Registry
// ============================================================================

dioxus_docs_kit::doc_content_map!();

static DOCS: LazyLock<DocsRegistry> = LazyLock::new(|| {
    DocsConfig::new(include_str!("../docs/_nav.json"), doc_content_map())
        .with_openapi(
            "api-reference",
            include_str!("../docs/api-reference/petstore.yaml"),
        )
        // Distilled from nightly rustdoc JSON — regenerate with `just api`.
        .with_rustdoc(
            "rust-api",
            include_str!("../docs/rust-api/dioxus-docs-kit.api.json"),
        )
        .with_default_path("getting-started/introduction")
        .with_theme_toggle("light", "dark", "dark")
        .build()
});

// ============================================================================
// Blog Registry
// ============================================================================

dioxus_docs_kit::blog_content_map!();

static BLOG: LazyLock<BlogRegistry> = LazyLock::new(|| {
    BlogConfig::new(include_str!("../blog/_blog.json"), blog_content_map())
        .with_posts_per_page(9)
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
    #[end_layout]
    #[layout(MyBlogLayout)]
        #[route("/blog")]
        BlogIndex {},
        #[route("/blog/:slug")]
        BlogPage { slug: String },
}

const SEGGWAT_LOGO: Asset = asset!("/assets/seggwat_logo.avif");
const INFRAPAGE_LOGO: Asset = asset!("/assets/infrapage_logo.ico");
const STEPSHOTS_LOGO: Asset = asset!("/assets/stepshots_logo.svg");
const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

// ============================================================================
// Theme preset picker (demos the dk-* theming surface)
// ============================================================================

const THEME_PRESET_DEFAULT: &str =
    include_str!("../crates/dioxus-docs-kit/examples/themes/default.css");
const THEME_PRESET_WARM: &str =
    include_str!("../crates/dioxus-docs-kit/examples/themes/warm-editorial.css");
const THEME_PRESET_BRUTALIST: &str =
    include_str!("../crates/dioxus-docs-kit/examples/themes/brutalist-light.css");
const THEME_PRESET_SHADCN_LIGHT: &str =
    include_str!("../crates/dioxus-docs-kit/examples/themes/shadcn-light.css");
const THEME_PRESET_SHADCN_DARK: &str =
    include_str!("../crates/dioxus-docs-kit/examples/themes/shadcn-dark.css");

const THEME_PRESET_STORAGE_KEY: &str = "dk-theme-preset";
const DAISY_THEME_STORAGE_KEY: &str = "docs-theme";

#[derive(Clone, Copy)]
struct CurrentPreset(Signal<String>);

fn preset_css(name: &str) -> &'static str {
    match name {
        "warm-editorial" => THEME_PRESET_WARM,
        "brutalist-light" => THEME_PRESET_BRUTALIST,
        "shadcn-light" => THEME_PRESET_SHADCN_LIGHT,
        "shadcn-dark" => THEME_PRESET_SHADCN_DARK,
        _ => THEME_PRESET_DEFAULT,
    }
}

/// Mode each preset locks DaisyUI's `data-theme` to. `None` means the preset
/// is identity-only and the user's manual dark/light toggle keeps control.
fn preset_mode(name: &str) -> Option<&'static str> {
    match name {
        "warm-editorial" | "brutalist-light" | "shadcn-light" => Some("light"),
        "shadcn-dark" => Some("dark"),
        _ => None,
    }
}

// On the client (wasm) build, launch normally.
#[cfg(not(feature = "server"))]
fn main() {
    dioxus::launch(App);
}

// On the server build, serve the Dioxus app plus the kit's crawler-facing
// routes (per-page `.md`, llms.txt, sitemaps, RSS, robots.txt). These are
// plain Axum routes (see `dioxus_docs_kit::server`), NOT `#[get]` server
// functions — a server function would JSON-encode the body.
#[cfg(feature = "server")]
fn main() {
    use dioxus_docs_kit::server::SeoRouter;

    const SITE_TITLE: &str = "Dioxus Docs Kit";
    const SITE_DESC: &str = "A Dioxus-powered documentation framework with MDX rendering, OpenAPI reference pages, and full-text search.";

    dioxus::server::serve(|| async {
        let seo = SeoRouter::new(SITE_URL, SITE_TITLE, SITE_DESC)
            .with_docs(&DOCS, "/docs")
            .with_blog(&BLOG, "/blog")
            .into_router();
        Ok(dioxus::server::router(App).merge(seo))
    });
}

#[component]
fn App() -> Element {
    let mut preset = use_signal(|| "default".to_string());
    use_context_provider(|| CurrentPreset(preset));

    // Read stored preset from localStorage on mount.
    use_effect(move || {
        spawn(async move {
            let mut eval = document::eval(&format!(
                r#"
                let p = null;
                try {{ p = localStorage.getItem('{THEME_PRESET_STORAGE_KEY}'); }} catch(e) {{}}
                dioxus.send(p || 'default');
                "#
            ));
            if let Ok(stored) = eval.recv::<String>().await {
                preset.set(stored);
            }
        });
    });

    // Sync DaisyUI's data-theme to whichever preset is active.
    // Non-default presets lock the mode (currently both ship as light themes).
    // Default presets defer to the user's manual ThemeToggle choice in localStorage.
    use_effect(move || {
        let mode = preset_mode(&preset());
        spawn(async move {
            match mode {
                Some(forced) => {
                    let _ = document::eval(&format!(
                        "document.documentElement.setAttribute('data-theme', '{forced}');"
                    ));
                }
                None => {
                    let _ = document::eval(&format!(
                        r#"
                        try {{
                            let t = localStorage.getItem('{DAISY_THEME_STORAGE_KEY}') || 'dark';
                            document.documentElement.setAttribute('data-theme', t);
                        }} catch(e) {{}}
                        "#
                    ));
                }
            }
        });
    });

    let css = preset_css(&preset());

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        style { id: "dk-theme-preset", dangerous_inner_html: "{css}" }
        Router::<Route> {}
    }
}

/// Dropdown that swaps the active `dk-*` theme preset.
#[component]
fn ThemePresetPicker() -> Element {
    let CurrentPreset(mut preset) = use_context::<CurrentPreset>();
    let current = preset();

    // (slug, label, locked-mode)
    let options: [(&str, &str, Option<&str>); 5] = [
        ("default", "Default", None),
        ("warm-editorial", "Warm editorial", Some("light")),
        ("brutalist-light", "Brutalist light", Some("light")),
        ("shadcn-light", "shadcn light", Some("light")),
        ("shadcn-dark", "shadcn dark", Some("dark")),
    ];

    rsx! {
        div { class: "dropdown dropdown-end",
            div {
                tabindex: "0",
                role: "button",
                class: "btn btn-ghost btn-sm gap-2",
                "aria-label": "Theme preset",
                Icon { class: "size-4", icon: LdPalette }
                span { class: "hidden sm:inline text-sm", "Preset" }
            }
            ul {
                tabindex: "0",
                class: "dropdown-content z-[60] menu p-2 shadow-lg bg-base-200 border border-base-300 rounded-lg w-60 mt-2",
                for (name, label, mode) in options {
                    {
                        let is_active = current == name;
                        let name_owned = name.to_string();
                        rsx! {
                            li {
                                button {
                                    class: if is_active { "active font-medium flex items-center justify-between gap-3" } else { "flex items-center justify-between gap-3" },
                                    onclick: move |_| {
                                        let n = name_owned.clone();
                                        preset.set(n.clone());
                                        spawn(async move {
                                            let _ = document::eval(&format!(
                                                "try {{ localStorage.setItem('{THEME_PRESET_STORAGE_KEY}', '{n}'); }} catch(e) {{}}"
                                            ));
                                        });
                                    },
                                    span { "{label}" }
                                    match mode {
                                        Some(m) => rsx! {
                                            span { class: "text-[10px] uppercase tracking-wider opacity-60", "{m}" }
                                        },
                                        None => rsx! {
                                            span { class: "text-[10px] uppercase tracking-wider opacity-60", "auto" }
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Renders either the consumer-supplied ThemeToggle (default preset) or a
/// small locked badge showing which mode the active preset forces.
#[component]
fn ThemeControl(toggle: Element) -> Element {
    let CurrentPreset(preset) = use_context::<CurrentPreset>();
    match preset_mode(&preset()) {
        None => toggle,
        Some(mode) => rsx! {
            span {
                class: "badge badge-ghost badge-sm gap-1 capitalize",
                title: "Locked by theme preset",
                Icon { class: "size-3", icon: LdLock }
                "{mode}"
            }
        },
    }
}

/// Which top-level section a navbar belongs to (highlights its menu entry).
#[derive(Clone, Copy, PartialEq)]
enum NavSection {
    Home,
    Docs,
    Blog,
}

/// The site navbar, shared by the Home, docs, and blog layouts.
///
/// - `drawer_open`: mobile-drawer toggle signal; `None` on pages without a drawer.
/// - `theme_toggle`: the section's toggle element (wrapped in `ThemeControl`).
#[component]
fn SiteNavbar(
    section: NavSection,
    drawer_open: Option<Signal<bool>>,
    search_open: Signal<bool>,
    theme_toggle: Element,
) -> Element {
    let link_class = |active: NavSection| {
        if section == active {
            "btn btn-ghost btn-sm rounded-lg font-medium btn-active"
        } else {
            "btn btn-ghost btn-sm rounded-lg font-medium"
        }
    };
    // Without a drawer there is no mobile fallback, so keep the menu visible.
    let menu_class = if drawer_open.is_some() {
        "menu menu-horizontal gap-1 hidden lg:flex"
    } else {
        "menu menu-horizontal gap-1"
    };

    rsx! {
        div { class: "navbar bg-base-200 border-b border-base-300 px-4 lg:px-8",
            div { class: "flex-1 gap-2",
                if let Some(mut drawer) = drawer_open {
                    button {
                        class: "btn btn-ghost btn-sm btn-square lg:hidden docs-menu-btn",
                        onclick: move |_| drawer.toggle(),
                        Icon { class: "size-5", icon: LdMenu }
                    }
                }
                Link {
                    to: Route::Home {},
                    class: "text-xl font-semibold tracking-tight hover:opacity-80 transition-opacity",
                    "Dioxus Docs Kit"
                }
            }
            div { class: "flex-none flex items-center gap-1",
                ul { class: "{menu_class}",
                    li {
                        Link { to: Route::Home {}, class: link_class(NavSection::Home), "Home" }
                    }
                    li {
                        Link { to: Route::BlogIndex {}, class: link_class(NavSection::Blog), "Blog" }
                    }
                    li {
                        Link {
                            to: Route::DocsPage { slug: vec!["getting-started".into(), "introduction".into()] },
                            class: link_class(NavSection::Docs),
                            "Docs"
                        }
                    }
                }
                SearchButton { search_open }
                a {
                    href: "https://github.com/hauju/dioxus-docs-kit",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "btn btn-ghost btn-sm btn-square rounded-lg",
                    "aria-label": "GitHub repository",
                    Icon { class: "size-5", icon: LdGithub }
                }
                ThemePresetPicker {}
                ThemeControl { toggle: theme_toggle }
            }
        }
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

    let current_path = match route {
        Route::DocsPage { slug } => slug.join("/"),
        _ => String::new(),
    };

    let docs_ctx = use_docs_context(
        current_path,
        "/docs",
        Callback::new(move |path: String| {
            let slug: Vec<String> = path.split('/').map(String::from).collect();
            nav.push(Route::DocsPage { slug });
        }),
    )
    .with_site_url(SITE_URL)
    .with_markdown_alternate(true);

    let providers = use_docs_providers(&DOCS, docs_ctx);

    rsx! {
        DocsLayout {
            header: rsx! {
                SiteNavbar {
                    section: NavSection::Docs,
                    drawer_open: providers.drawer_open,
                    search_open: providers.search_open,
                    theme_toggle: rsx! { ThemeToggle {} },
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
// Blog Layout Wrapper
// ============================================================================

#[component]
fn MyBlogLayout() -> Element {
    let nav = use_navigator();
    let route = use_route::<Route>();

    let current_slug = use_memo(use_reactive!(|route| match route {
        Route::BlogPage { slug } => slug,
        _ => String::new(),
    }));

    let blog_ctx = BlogContext::new(
        current_slug,
        "/blog",
        Callback::new(move |slug: String| {
            if slug.is_empty() {
                nav.push(Route::BlogIndex {});
            } else {
                nav.push(Route::BlogPage { slug });
            }
        }),
    )
    .with_site_url(SITE_URL)
    .with_markdown_alternate(true);

    let providers = use_blog_providers(&BLOG, blog_ctx);

    rsx! {
        BlogLayout {
            header: rsx! {
                SiteNavbar {
                    section: NavSection::Blog,
                    drawer_open: providers.drawer_open,
                    search_open: providers.search_open,
                    theme_toggle: rsx! { BlogThemeToggle {} },
                }
            },
            Outlet::<Route> {}
        }
    }
}

#[component]
fn BlogIndex() -> Element {
    rsx! {
        BlogList {
            hero: rsx! {
                div { class: "text-center mb-12",
                    h1 { class: "text-4xl font-bold tracking-tight mb-3", "Blog" }
                    p { class: "text-lg text-base-content/60 max-w-2xl mx-auto",
                        "Thoughts on Rust, Dioxus, and web development."
                    }
                }
            }
        }
    }
}

#[component]
fn BlogPage(slug: String) -> Element {
    rsx! {
        BlogPostView { slug }
    }
}

// SITE_URL is the deployed origin (no trailing slash), read at BUILD time from
// DOCS_SITE_URL (set on the CI bundle step). It is deliberately a build-time,
// docs-specific name — separate from any runtime BASE_URL — because the page
// canonical/og/breadcrumb are baked into both the server SSR and the client
// wasm and must match, so they can't be sourced from a runtime-only env. Feeds
// the server sitemap/robots/RSS routes AND the page contexts below. Falls back
// to localhost when DOCS_SITE_URL is unset or empty, so a misconfigured build
// degrades to dev URLs instead of emitting broken empty-origin links.
const SITE_URL: &str = match option_env!("DOCS_SITE_URL") {
    Some(url) if !url.is_empty() => url,
    _ => "http://localhost:8080",
};

// ============================================================================
// App-specific pages (Navbar, Home)
// ============================================================================

/// Shared navbar component with DaisyUI styling
#[component]
fn Navbar() -> Element {
    let nav = use_navigator();

    // Provide DocsRegistry + DocsContext + search signal so SearchModal can
    // function outside the docs layout. This search-only context never renders
    // page meta.
    let docs_ctx = DocsContext::new(
        Signal::new(String::new()),
        "/docs",
        Callback::new(move |path: String| {
            let slug: Vec<String> = path.split('/').map(String::from).collect();
            nav.push(Route::DocsPage { slug });
        }),
    )
    .with_site_url(SITE_URL);

    let providers = use_docs_providers(&DOCS, docs_ctx);

    // Theme state (persisted preference + data-theme application)
    dioxus_docs_kit::use_theme_provider(DOCS.theme.clone());

    rsx! {
        SiteNavbar {
            section: NavSection::Home,
            search_open: providers.search_open,
            theme_toggle: rsx! { ThemeToggle {} },
        }

        main { class: "min-h-screen bg-base-100 dk-root",
            Outlet::<Route> {}
        }

        SearchModal {}
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
            UsedBySection {}
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

// ── Used By ──────────────────────────────────────────────────────────────────

#[component]
fn UsedBySection() -> Element {
    rsx! {
        section { class: "px-4 py-20",
            div { class: "max-w-5xl mx-auto",
                div { class: "text-center mb-12",
                    h2 { class: "text-3xl font-bold tracking-tight", "Used in Production" }
                    p { class: "mt-3 text-base-content/60",
                        "Projects powered by Dioxus Docs Kit."
                    }
                }
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 max-w-5xl mx-auto",
                    a {
                        href: "https://seggwat.com",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "group flex flex-col gap-4 p-6 rounded-xl border border-base-300 bg-base-200/30 hover:border-primary/30 transition-all duration-200",
                        div { class: "flex items-center justify-between",
                            div { class: "flex items-center gap-3",
                                img { src: SEGGWAT_LOGO, alt: "SeggWat logo", class: "size-8 rounded" }
                                h3 { class: "font-semibold text-lg text-base-content", "seggwat.com" }
                            }
                            Icon { class: "size-4 text-base-content/40 group-hover:text-primary transition-colors", icon: LdExternalLink }
                        }
                        p { class: "text-sm text-base-content/60 leading-relaxed",
                            "Feedback collection widget for websites — collect bug reports, feature ideas, and user feedback in one dashboard."
                        }
                    }
                    a {
                        href: "https://infra.page",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "group flex flex-col gap-4 p-6 rounded-xl border border-base-300 bg-base-200/30 hover:border-primary/30 transition-all duration-200",
                        div { class: "flex items-center justify-between",
                            div { class: "flex items-center gap-3",
                                img { src: INFRAPAGE_LOGO, alt: "infrapage logo", class: "size-8 rounded" }
                                h3 { class: "font-semibold text-lg text-base-content", "infra.page" }
                            }
                            Icon { class: "size-4 text-base-content/40 group-hover:text-primary transition-colors", icon: LdExternalLink }
                        }
                        p { class: "text-sm text-base-content/60 leading-relaxed",
                            "Widget-based dashboard connecting GitHub, Polar, uptime monitors, and more into a single status page."
                        }
                    }
                    a {
                        href: "https://stepshots.com",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "group flex flex-col gap-4 p-6 rounded-xl border border-base-300 bg-base-200/30 hover:border-primary/30 transition-all duration-200",
                        div { class: "flex items-center justify-between",
                            div { class: "flex items-center gap-3",
                                img { src: STEPSHOTS_LOGO, alt: "Stepshots logo", class: "size-8 rounded" }
                                h3 { class: "font-semibold text-lg text-base-content", "stepshots.com" }
                            }
                            Icon { class: "size-4 text-base-content/40 group-hover:text-primary transition-colors", icon: LdExternalLink }
                        }
                        p { class: "text-sm text-base-content/60 leading-relaxed",
                            "Screenshot-based product demos — record clickthrough walkthroughs and generate polished step-by-step guides."
                        }
                    }
                }
            }
        }
    }
}

// ── Code Example ─────────────────────────────────────────────────────────────

const CODE_SNIPPET: &str = r#"use dioxus_docs_kit::{
    DocsConfig, DocsContext, DocsLayout, DocsPageContent,
    DocsRegistry, SearchButton, use_docs_providers,
};
use std::sync::LazyLock;

// build.rs: dioxus_docs_kit_build::generate_content_map("docs/_nav.json");
dioxus_docs_kit::doc_content_map!();

static DOCS: LazyLock<DocsRegistry> = LazyLock::new(|| {
    DocsConfig::new(include_str!("../docs/_nav.json"), doc_content_map())
        .with_openapi("api-reference", include_str!("../docs/api.yaml"))
        .with_default_path("getting-started/introduction")
        .with_theme_toggle("light", "dark", "dark")
        .build()
});

#[component]
fn MyDocsLayout() -> Element {
    let docs_ctx = DocsContext::new(current_path, "/docs", navigate);
    let providers = use_docs_providers(&DOCS, docs_ctx);

    rsx! {
        DocsLayout {
            header: rsx! { SearchButton { search_open: providers.search_open } },
            Outlet::<Route> {}
        }
    }
}"#;

#[component]
fn CodeSection() -> Element {
    rsx! {
        section { class: "px-4 py-20",
            div { class: "max-w-3xl mx-auto",
                div { class: "text-center mb-12",
                    h2 { class: "text-3xl font-bold tracking-tight", "Simple Integration" }
                    p { class: "mt-3 text-base-content/60",
                        "Set up your documentation site with an skill and a bit of glue code."
                    }
                }
                // Claude Code skill callout
                div { class: "mb-8 rounded-xl border border-primary/20 bg-primary/5 p-6",
                    h3 { class: "font-semibold text-base-content mb-4 flex items-center gap-2",
                        div { class: "flex items-center justify-center size-8 rounded-lg bg-primary/10 text-primary",
                            Icon { class: "size-4", icon: LdPackage }
                        }
                        "Claude Code Integration Skill"
                    }
                    p { class: "text-sm text-base-content/60 leading-relaxed mb-5",
                        "This repo includes a Claude Code skill that automates the entire setup. "
                        "Install the skill once, then use it in any Dioxus project."
                    }
                    // Steps
                    div { class: "space-y-4",
                        // Step 1
                        div { class: "flex items-start gap-3",
                            span { class: "flex items-center justify-center size-6 rounded-full bg-primary text-primary-content text-xs font-bold shrink-0 mt-0.5", "1" }
                            div { class: "flex-1 min-w-0",
                                p { class: "text-sm font-medium text-base-content mb-1.5", "Install the skill" }
                                code { class: "block bg-base-200 text-xs font-mono px-3 py-2 rounded-lg text-base-content/70 overflow-x-auto",
                                    "git clone https://github.com/hauju/dioxus-docs-kit.git /tmp/ddk && cp -r /tmp/ddk/skills/dioxus-docs-kit-integration ~/.claude/skills/"
                                }
                            }
                        }
                        // Step 2
                        div { class: "flex items-start gap-3",
                            span { class: "flex items-center justify-center size-6 rounded-full bg-primary text-primary-content text-xs font-bold shrink-0 mt-0.5", "2" }
                            div { class: "flex-1 min-w-0",
                                p { class: "text-sm font-medium text-base-content mb-1.5", "Open your Dioxus project in Claude Code and ask:" }
                                code { class: "block bg-base-200 text-sm font-mono px-3 py-2 rounded-lg text-primary",
                                    "\"Add dioxus-docs-kit documentation to this project\""
                                }
                            }
                        }
                        // Step 3
                        div { class: "flex items-start gap-3",
                            span { class: "flex items-center justify-center size-6 rounded-full bg-primary text-primary-content text-xs font-bold shrink-0 mt-0.5", "3" }
                            div { class: "flex-1 min-w-0",
                                p { class: "text-sm font-medium text-base-content",
                                    "Claude handles the rest "
                                    span { class: "text-base-content/40", "— dependencies, build.rs, routes, layout, Tailwind safelist, and starter content" }
                                }
                            }
                        }
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
                    div { class: "dk-code-block-body bg-base-200",
                        Code {
                            src: SourceCode::new(Language::Rust, CODE_SNIPPET.to_string()),
                            theme: CodeTheme::system(Theme::GITHUB_LIGHT, Theme::TOKYO_NIGHT),
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
