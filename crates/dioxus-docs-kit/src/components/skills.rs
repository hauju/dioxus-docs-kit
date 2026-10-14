//! Agent-skill pages: layout with a grouped sidebar, drawer and search,
//! index cards, and a page
//! per `SKILL.md` with its install command and GitHub links.

use dioxus::prelude::*;
use dioxus_mdx::lucide::{Icon, LdExternalLink, LdFileText, LdGithub, LdMenu};
use dioxus_mdx::{CodeBlockNode, DocCodeBlock, DocContent, DocTableOfContents, extract_headers};

use crate::SkillsContext;
use crate::components::blog::listing_tags;
use crate::components::docs_layout::{DrawerOpen, SearchOpen};
use crate::components::managed_head::{ManagedPageHead, NotFoundMeta};
use crate::components::search_shell::{SearchHit, SearchModalShell};
use oxidt_docs_kit_core::search::MAX_RESULTS;

use super::mobile_drawer::DrawerPanel;
use crate::skills::SkillsRegistry;
use oxidt_docs_kit_core::seo::join_site_url;

/// Skills layout shell: header (yours, or a default bar with the drawer
/// toggle and search button), a sidebar listing every group and skill (a
/// drawer below `lg`), the routed page, and the skills search modal (also on
/// Cmd/Ctrl+K).
///
/// # Context requirements
///
/// `&'static SkillsRegistry` and [`SkillsContext`], both provided by
/// [`use_skills_providers`](crate::use_skills_providers).
#[component]
pub fn SkillsLayout(header: Option<Element>, children: Element) -> Element {
    let parent_search: Option<SearchOpen> = try_use_context();
    let parent_drawer: Option<DrawerOpen> = try_use_context();
    let local_search = use_signal(|| false);
    let local_drawer = use_signal(|| false);
    let search_open = parent_search.map(|s| s.0).unwrap_or(local_search);
    let mut drawer_open = parent_drawer.map(|d| d.0).unwrap_or(local_drawer);
    use_context_provider(|| SearchOpen(search_open));
    use_context_provider(|| DrawerOpen(drawer_open));

    crate::components::shared::use_search_hotkey(search_open);

    rsx! {
        div { class: "dk-root min-h-screen bg-base-100",
            div { class: "dk-header sticky top-0 z-50",
                if let Some(hdr) = header {
                    {hdr}
                } else {
                    div { class: "navbar bg-base-200 border-b border-base-300 px-4 lg:px-8",
                        div { class: "flex-1 gap-2",
                            button {
                                class: "btn btn-ghost btn-sm btn-square lg:hidden",
                                "aria-label": "Open navigation",
                                onclick: move |_| drawer_open.toggle(),
                                Icon { class: "size-5", icon: LdMenu }
                            }
                        }
                        div { class: "flex-none gap-1",
                            SkillsSearchButton { search_open }
                        }
                    }
                }
            }
            div { class: "dk-shell flex",
                aside { class: "dk-sidebar w-64 shrink-0 border-r border-base-300 bg-base-200/30 hidden lg:block",
                    div { class: "sticky top-16 h-[calc(100vh-4rem)] overflow-y-auto p-6 flex flex-col gap-6",
                        SkillsSidebar {}
                    }
                }
                div { class: "dk-main flex-1 min-w-0", {children} }
            }
        }

        SkillsMobileDrawer { open: drawer_open }
        SkillsSearchModal {}
    }
}

/// Grouped skill links with the current skill highlighted.
#[component]
fn SkillsSidebar() -> Element {
    let registry = use_context::<&'static SkillsRegistry>();
    let ctx = use_context::<SkillsContext>();
    let current = (ctx.current_slug)();

    rsx! {
        nav { class: "dk-nav space-y-6",
            for (group, skills) in registry.groups() {
                div { class: "dk-nav-group space-y-2",
                    h3 { class: "dk-nav-group-title font-semibold text-sm text-base-content/70 uppercase tracking-wider px-3",
                        "{group}"
                    }
                    ul { class: "space-y-1",
                        for skill in skills {
                            li {
                                Link {
                                    to: NavigationTarget::Internal(format!("{}/{}", ctx.base_path, skill.slug)),
                                    class: if current == skill.slug {
                                        "dk-nav-item block px-3 py-2 text-sm rounded-lg transition-colors dk-nav-item-active bg-primary/10 text-primary font-medium border-l-2 border-primary"
                                    } else {
                                        "dk-nav-item block px-3 py-2 text-sm rounded-lg transition-colors text-base-content/70 hover:text-base-content hover:bg-base-200"
                                    },
                                    "{skill.name}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Slide-in skills sidebar for screens below `lg`; closes on navigation.
#[component]
pub fn SkillsMobileDrawer(mut open: Signal<bool>) -> Element {
    let ctx = use_context::<SkillsContext>();
    let current_slug = ctx.current_slug;
    use_effect(move || {
        let _ = current_slug();
        open.set(false);
    });

    rsx! {
        DrawerPanel { open, title: "Skills", SkillsSidebar {} }
    }
}

/// Search button for skills headers (the docs [`SearchButton`](crate::SearchButton)).
pub use crate::components::docs_layout::SearchButton as SkillsSearchButton;

/// Skills search modal, opened by Cmd/Ctrl+K or [`SkillsSearchButton`].
/// Section hits navigate to the skill and scroll to the heading.
#[component]
pub fn SkillsSearchModal() -> Element {
    let ctx = use_context::<SkillsContext>();
    let registry = use_context::<&'static SkillsRegistry>();

    let search = use_callback(move |query: String| {
        registry
            .search_hits(&query, MAX_RESULTS)
            .into_iter()
            .map(|hit| SearchHit {
                target: hit.target,
                title: hit.title,
                context: hit.context,
                badge: None,
                meta: hit.breadcrumb,
                tags: Vec::new(),
                snippet: hit.snippet,
            })
            .collect()
    });

    let on_select = use_callback(move |target: String| {
        // `target` is `slug` or `slug#anchor`.
        let (slug, anchor) = target.split_once('#').unwrap_or((&target, ""));
        (ctx.navigate)(slug.to_string());
        #[cfg(target_arch = "wasm32")]
        if !anchor.is_empty() {
            super::search_modal::scroll_to_anchor(anchor.to_string());
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = anchor;
    });

    rsx! {
        SearchModalShell {
            id: "dk-skills-search",
            placeholder: "Search skills...",
            search,
            on_select,
        }
    }
}

/// Skills index: one card per skill, grouped as in `_skills.json`.
#[component]
pub fn SkillsIndex(
    #[props(default = "Skills".to_string())] title: String,
    #[props(default)] description: String,
) -> Element {
    let registry = use_context::<&'static SkillsRegistry>();
    let ctx = use_context::<SkillsContext>();

    let canonical = ctx
        .site_url
        .as_deref()
        .map(|origin| join_site_url(origin, &ctx.base_path, ""));

    rsx! {
        if ctx.auto_meta {
            ManagedPageHead {
                title: title.clone(),
                tags: listing_tags(&title, &description, canonical.as_deref(), None),
            }
        }
        div { class: "max-w-6xl mx-auto px-4 py-12",
            h1 { class: "text-4xl font-bold tracking-tight mb-3", "{title}" }
            if !description.is_empty() {
                p { class: "text-lg text-base-content/60 mb-6", "{description}" }
            }
            for (group, skills) in registry.groups() {
                h2 { class: "text-2xl font-semibold mt-8 mb-4", "{group}" }
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                    for skill in skills {
                        article { class: "group flex flex-col rounded-xl border border-base-300 bg-base-200/30 hover:border-primary/30 hover:shadow-lg transition-all duration-200 overflow-hidden",
                            div { class: "flex flex-col flex-1 p-5 gap-3",
                                h3 { class: "text-lg font-semibold leading-snug group-hover:text-primary transition-colors line-clamp-2",
                                    Link {
                                        to: NavigationTarget::Internal(format!("{}/{}", ctx.base_path, skill.slug)),
                                        "{skill.name}"
                                    }
                                }
                                if !skill.description.is_empty() {
                                    p { class: "text-sm text-base-content/60 leading-relaxed line-clamp-3",
                                        "{skill.description}"
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

/// A single skill: name, description, install command (with copy button),
/// links to the `SKILL.md` on GitHub, then the rendered body. Unknown slugs
/// render a noindex 404 page.
#[component]
pub fn SkillPage(slug: String) -> Element {
    let registry = use_context::<&'static SkillsRegistry>();
    let ctx = use_context::<SkillsContext>();

    let Some(skill) = registry.get_skill(&slug) else {
        return rsx! {
            NotFoundMeta { title: "Skill not found", auto_meta: ctx.auto_meta }
            div { class: "max-w-4xl mx-auto px-4 py-12",
                div { class: "text-center",
                    h1 { class: "text-4xl font-bold mb-4", "404" }
                    p { class: "text-base-content/70 mb-8", "Skill not found: {slug}" }
                    Link {
                        to: NavigationTarget::Internal(ctx.base_path.clone()),
                        class: "btn btn-primary",
                        "Back to Skills"
                    }
                }
            }
        };
    };

    let install = CodeBlockNode {
        language: Some("bash".to_string()),
        code: registry.install_command(skill),
        filename: None,
    };
    let headers = extract_headers(&skill.raw_markdown);

    rsx! {
        SkillMeta { slug: slug.clone() }
        div { class: "flex max-w-6xl mx-auto",
            main { class: "flex-1 min-w-0 px-4 py-12 lg:px-12",
                article { class: "max-w-3xl mx-auto",
                    header { class: "mb-8 pb-8 border-b border-base-300",
                        div { class: "flex flex-wrap gap-1.5 mb-4",
                            span { class: "badge badge-sm badge-outline badge-primary font-medium", "{skill.group}" }
                        }
                        h1 { class: "text-4xl font-bold tracking-tight mb-4", "{skill.name}" }
                        if !skill.description.is_empty() {
                            p { class: "text-lg text-base-content/60 mb-6", "{skill.description}" }
                        }
                        DocCodeBlock { block: install }
                        div { class: "flex items-center gap-4 text-sm text-base-content/60",
                            a {
                                href: registry.github_url(skill),
                                target: "_blank",
                                rel: "noopener noreferrer",
                                class: "flex items-center gap-1 hover:text-base-content",
                                Icon { class: "size-3.5", icon: LdGithub }
                                "Source on GitHub"
                                Icon { class: "size-3", icon: LdExternalLink }
                            }
                            a {
                                href: registry.raw_url(skill),
                                target: "_blank",
                                rel: "noopener noreferrer",
                                class: "flex items-center gap-1 hover:text-base-content",
                                Icon { class: "size-3.5", icon: LdFileText }
                                "Raw"
                            }
                        }
                    }

                    div { class: "prose prose-base max-w-none
                        prose-headings:scroll-mt-20
                        prose-h2:text-2xl prose-h2:font-semibold prose-h2:mt-10 prose-h2:mb-4
                        prose-h3:text-xl prose-h3:font-medium prose-h3:mt-8 prose-h3:mb-3
                        prose-p:text-base-content/80 prose-p:leading-relaxed
                        prose-a:text-primary prose-a:no-underline hover:prose-a:underline
                        prose-code:bg-base-200 prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded prose-code:text-sm",
                        DocContent { nodes: skill.body.clone() }
                    }
                }
            }

            if !headers.is_empty() {
                aside { class: "w-56 shrink-0 hidden xl:block",
                    div { class: "sticky top-20 p-6",
                        DocTableOfContents { headers }
                    }
                }
            }
        }
    }
}

/// Title, description, Open Graph / Twitter Card and canonical tags for one
/// skill. Emits nothing when [`SkillsContext::auto_meta`] is off; canonical and
/// `og:url` need [`SkillsContext::site_url`].
#[component]
pub fn SkillMeta(slug: String) -> Element {
    let registry = use_context::<&'static SkillsRegistry>();
    let ctx = use_context::<SkillsContext>();

    let Some(skill) = registry.get_skill(&slug).filter(|_| ctx.auto_meta) else {
        return rsx! {};
    };
    let canonical = ctx
        .site_url
        .as_deref()
        .map(|origin| join_site_url(origin, &ctx.base_path, &slug));
    let tags = listing_tags(&skill.name, &skill.description, canonical.as_deref(), None);
    rsx! { ManagedPageHead { title: skill.name.clone(), tags } }
}
