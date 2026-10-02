//! The docs page shell: tab bar, sidebar, article, table of contents and
//! previous/next links. Markup and classes mirror `dioxus-docs-kit`.

use dioxus_mdx::extract_headers;
use docs_kit_core::{DocsRegistry, NavGroup};
use topcoat::{
    Result,
    router::StatusCode,
    view::{View, component, view},
};

use crate::content::doc_content;

/// A full docs page for `path` (relative to `base_path`, e.g.
/// `"getting-started/introduction"`; empty means the registry's default page).
///
/// Renders the body content only — wrap it in your own `<html>` document and
/// link [`STYLESHEET`](crate::STYLESHEET) (or your own Tailwind build) in its
/// `<head>`. Unknown paths render a 404 page and set the response status.
#[component]
pub async fn docs_page(registry: &DocsRegistry, base_path: &str, path: &str) -> Result<impl View> {
    let path = if path.is_empty() {
        registry.default_path.as_str()
    } else {
        path
    };
    let active_tab = registry
        .tab_for_path(path)
        .or_else(|| registry.nav.tabs.first().cloned());
    let groups: Vec<&NavGroup> = match &active_tab {
        Some(tab) if registry.nav.has_tabs() => registry.nav.groups_for_tab(tab),
        _ => registry.nav.groups.iter().collect(),
    };

    Ok(view! {
        <div class="dk-root dk-docs-root min-h-screen bg-base-100">
            if registry.nav.has_tabs() {
                <div class="dk-header sticky top-0 z-50">
                    <div class="dk-tabs bg-base-200/80 backdrop-blur border-b border-base-300 px-4 lg:px-8">
                        <div class="flex gap-6">
                            for tab in &registry.nav.tabs {
                                let first = registry.nav.groups_for_tab(tab).first().and_then(|g| g.pages.first()).cloned().unwrap_or_default();
                                let is_active = active_tab.as_ref() == Some(tab);
                                <a
                                    href=(format!("{base_path}/{first}"))
                                    class=(if is_active {
                                        "dk-tab dk-tab-active px-1 py-2.5 text-sm transition-colors -mb-px text-primary border-b-2 border-primary font-medium"
                                    } else {
                                        "dk-tab px-1 py-2.5 text-sm transition-colors -mb-px text-base-content/60 hover:text-base-content border-b-2 border-transparent"
                                    })
                                >
                                    (tab.as_str())
                                </a>
                            }
                        </div>
                    </div>
                </div>
            }

            // Below `lg` the sidebar is hidden; the same nav sits in a disclosure.
            <details class="dk-mobile-nav lg:hidden border-b border-base-300 bg-base-200/30">
                <summary class="px-4 py-3 text-sm font-medium cursor-pointer">"Menu"</summary>
                <div class="px-4 pb-4">
                    sidebar(registry: registry, groups: &groups, base_path: base_path, current: path)
                </div>
            </details>

            <div class="dk-shell flex">
                <aside class="dk-sidebar w-64 shrink-0 border-r border-base-300 bg-base-200/30 hidden lg:block">
                    <div class="sticky top-12 h-[calc(100vh-3rem)] overflow-y-auto p-6 flex flex-col gap-6">
                        sidebar(registry: registry, groups: &groups, base_path: base_path, current: path)
                    </div>
                </aside>
                <div class="dk-main flex-1 min-w-0">
                    article(registry: registry, base_path: base_path, path: path)
                </div>
            </div>
        </div>
    })
}

#[component]
async fn sidebar(
    registry: &DocsRegistry,
    groups: &[&NavGroup],
    base_path: &str,
    current: &str,
) -> Result<impl View> {
    Ok(view! {
        <nav class="dk-nav space-y-6">
            for group in groups {
                <div class="dk-nav-group space-y-2">
                    <h3 class="dk-nav-group-title font-semibold text-sm text-base-content/70 uppercase tracking-wider px-3">
                        (group.group.as_str())
                    </h3>
                    <ul class="space-y-1">
                        for page in &group.pages {
                            let title = registry.get_sidebar_title(page).unwrap_or_else(|| page.rsplit('/').next().unwrap_or(page).replace('-', " "));
                            <li>
                                <a
                                    href=(format!("{base_path}/{page}"))
                                    aria-current=((page == current).then_some("page"))
                                    class=(if page == current {
                                        "dk-nav-item dk-nav-item-active block px-3 py-2 text-sm rounded-lg transition-colors bg-primary/10 text-primary font-medium border-l-2 border-primary"
                                    } else {
                                        "dk-nav-item block px-3 py-2 text-sm rounded-lg transition-colors text-base-content/70 hover:text-base-content hover:bg-base-200"
                                    })
                                >
                                    (title)
                                </a>
                            </li>
                        }
                    </ul>
                </div>
            }
        </nav>
    })
}

#[component]
async fn article(registry: &DocsRegistry, base_path: &str, path: &str) -> Result<impl View> {
    let doc = registry.get_parsed_doc(path);
    let headers = doc
        .map(|d| extract_headers(&d.raw_markdown))
        .unwrap_or_default();
    let (prev, next) = registry.page_neighbors(path);

    Ok(view! {
        match doc {
            None => {
                (StatusCode::NOT_FOUND)
                <div class="container mx-auto px-8 py-12 max-w-4xl">
                    <div class="text-center">
                        <h1 class="text-4xl font-bold mb-4">"404"</h1>
                        <p class="text-base-content/70 mb-8">"Page not found: " (path)</p>
                        <a href=(format!("{base_path}/{}", registry.default_path)) class="btn btn-primary">"Go to Documentation"</a>
                    </div>
                </div>
            },
            Some(doc) => {
                <div class="flex">
                    <main class="flex-1 min-w-0 px-8 py-12 lg:px-12">
                        <article class="dk-article max-w-3xl mx-auto">
                            <header class="dk-article-header mb-8 pb-8 border-b border-base-300">
                                <h1 class="dk-article-title text-4xl font-bold tracking-tight mb-3 min-w-0 break-words">
                                    (doc.frontmatter.title.as_str())
                                </h1>
                                if let Some(desc) = &doc.frontmatter.description {
                                    <p class="dk-article-description text-lg text-base-content/70">(desc.as_str())</p>
                                }
                            </header>
                            <div class="dk-article-body prose prose-base max-w-none prose-headings:scroll-mt-16 prose-h2:text-2xl prose-h2:font-semibold prose-h2:mt-10 prose-h2:mb-4 prose-h3:text-xl prose-h3:font-medium prose-h3:mt-8 prose-h3:mb-3 prose-p:text-base-content/80 prose-p:leading-relaxed prose-a:text-primary prose-a:no-underline hover:prose-a:underline prose-code:bg-base-200 prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded prose-code:text-sm">
                                doc_content(nodes: &doc.content, base_path: base_path)
                            </div>
                            page_nav(registry: registry, base_path: base_path, prev: prev.as_deref(), next: next.as_deref())
                        </article>
                    </main>
                    if !headers.is_empty() {
                        <aside class="dk-toc w-56 shrink-0 hidden xl:block">
                            <div class="sticky top-12 p-6">
                                <h4 class="text-xs font-semibold uppercase tracking-wider text-base-content/50 mb-3">"On this page"</h4>
                                <ul class="space-y-2 text-sm">
                                    for (id, title, level) in &headers {
                                        <li class=(if *level > 2 { "pl-3" } else { "" })>
                                            <a href=(format!("#{id}")) class="text-base-content/60 hover:text-base-content transition-colors">(title.as_str())</a>
                                        </li>
                                    }
                                </ul>
                            </div>
                        </aside>
                    }
                </div>
            },
        }
    })
}

#[component]
async fn page_nav(
    registry: &DocsRegistry,
    base_path: &str,
    prev: Option<&str>,
    next: Option<&str>,
) -> Result<impl View> {
    let title = |p: &str| {
        registry
            .get_sidebar_title(p)
            .unwrap_or_else(|| p.to_string())
    };
    Ok(view! {
        <nav class="dk-pagination mt-16 pt-8 border-t border-base-300 flex justify-between gap-4">
            <div class="flex-1">
                if let Some(prev) = prev {
                    <a href=(format!("{base_path}/{prev}")) class="dk-page-prev group flex flex-col p-4 rounded-lg border border-base-300 hover:border-primary/50 hover:bg-base-200/50 transition-all">
                        <span class="text-xs text-base-content/50 mb-1">"Previous"</span>
                        <span class="font-medium group-hover:text-primary transition-colors">(title(prev))</span>
                    </a>
                }
            </div>
            <div class="flex-1 text-right">
                if let Some(next) = next {
                    <a href=(format!("{base_path}/{next}")) class="dk-page-next group flex flex-col p-4 rounded-lg border border-base-300 hover:border-primary/50 hover:bg-base-200/50 transition-all items-end">
                        <span class="text-xs text-base-content/50 mb-1">"Next"</span>
                        <span class="font-medium group-hover:text-primary transition-colors">(title(next))</span>
                    </a>
                }
            </div>
        </nav>
    })
}

/// `<title>` and `<meta name="description">` for a docs page; place it in
/// your document's `<head>`.
#[component]
pub async fn docs_head(registry: &DocsRegistry, path: &str) -> Result<impl View> {
    let path = if path.is_empty() {
        registry.default_path.as_str()
    } else {
        path
    };
    let title = registry
        .get_page_title(path)
        .unwrap_or_else(|| "Not found".to_string());
    let description = registry.get_page_description(path);
    Ok(view! {
        <title>(title)</title>
        if let Some(description) = description {
            <meta name="description" content=(description)>
        }
    })
}
