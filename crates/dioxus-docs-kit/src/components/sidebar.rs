use dioxus::prelude::*;
use dioxus_mdx::HttpMethod;

use crate::DocsContext;
use crate::registry::{DocsRegistry, NavGroup};

/// Documentation sidebar navigation.
#[component]
pub fn DocsSidebar() -> Element {
    let registry = use_context::<&'static DocsRegistry>();
    let active_tab = use_context::<Signal<String>>();
    let nav = &registry.nav;

    let groups: Vec<&NavGroup> = if nav.has_tabs() {
        nav.groups_for_tab(&active_tab())
    } else {
        nav.groups.iter().collect()
    };

    rsx! {
        nav { class: "space-y-6",
            for group in groups.iter() {
                SidebarGroup { group: (*group).clone() }
            }
        }
    }
}

/// A single sidebar group (normal or API Reference).
#[component]
fn SidebarGroup(group: NavGroup) -> Element {
    let registry = use_context::<&'static DocsRegistry>();
    let api_entries = registry.get_api_sidebar_entries();
    let is_api_group = group.group == registry.api_group_name;

    if is_api_group {
        rsx! {
            div { class: "space-y-2",
                h3 { class: "font-semibold text-sm text-base-content/70 uppercase tracking-wider px-3",
                    "{group.group}"
                }
                ul { class: "space-y-1",
                    for page in group.pages.iter() {
                        SidebarLink { path: page.clone() }
                    }
                }
                // Dynamic API endpoints grouped by tag
                for (tag, entries) in api_entries.iter() {
                    div { class: "mt-3",
                        h4 { class: "text-xs font-medium text-base-content/50 uppercase tracking-wider px-3 mb-1",
                            "{tag.name}"
                        }
                        ul { class: "space-y-0.5",
                            for entry in entries.iter() {
                                ApiSidebarLink {
                                    slug: entry.slug.clone(),
                                    title: entry.title.clone(),
                                    method: entry.method,
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        rsx! {
            div { class: "space-y-2",
                h3 { class: "font-semibold text-sm text-base-content/70 uppercase tracking-wider px-3",
                    "{group.group}"
                }
                ul { class: "space-y-1",
                    for page in group.pages.iter() {
                        SidebarLink { path: page.clone() }
                    }
                }
            }
        }
    }
}

/// Sidebar link for API endpoints with method badges.
#[component]
fn ApiSidebarLink(slug: String, title: String, method: HttpMethod) -> Element {
    let ctx = use_context::<DocsContext>();
    let registry = use_context::<&'static DocsRegistry>();

    let prefix = registry
        .get_first_api_prefix()
        .unwrap_or("api-reference");
    let path = format!("{prefix}/{slug}");

    let is_active = (ctx.current_path)() == path;

    let active_class = if is_active {
        "bg-primary/10 text-primary font-medium border-l-2 border-primary"
    } else {
        "text-base-content/70 hover:text-base-content hover:bg-base-200"
    };

    let method_color = match method {
        HttpMethod::Get => "text-success",
        HttpMethod::Post => "text-primary",
        HttpMethod::Put => "text-warning",
        HttpMethod::Delete => "text-error",
        HttpMethod::Patch => "text-info",
        _ => "text-base-content/50",
    };

    let method_label = match method {
        HttpMethod::Delete => "DEL",
        _ => method.as_str(),
    };

    let href = format!("{}/{}", ctx.base_path, path);

    rsx! {
        li {
            Link {
                to: NavigationTarget::Internal(href),
                class: "flex items-center gap-2 px-3 py-1.5 text-sm rounded-lg transition-colors {active_class}",
                span { class: "w-10 text-[10px] font-bold font-mono {method_color} shrink-0",
                    "{method_label}"
                }
                span { class: "truncate", "{title}" }
            }
        }
    }
}

/// Individual sidebar link.
#[component]
fn SidebarLink(path: String) -> Element {
    let ctx = use_context::<DocsContext>();
    let registry = use_context::<&'static DocsRegistry>();

    let title = registry.get_sidebar_title(&path).unwrap_or_else(|| {
        path.split('/').last().unwrap_or(&path).replace('-', " ")
    });

    let current = (ctx.current_path)();
    let is_active = current == path
        || (current.is_empty() && path == registry.default_path);

    let active_class = if is_active {
        "bg-primary/10 text-primary font-medium border-l-2 border-primary"
    } else {
        "text-base-content/70 hover:text-base-content hover:bg-base-200"
    };

    let href = format!("{}/{}", ctx.base_path, path);

    rsx! {
        li {
            Link {
                to: NavigationTarget::Internal(href),
                class: "block px-3 py-2 text-sm rounded-lg transition-colors {active_class}",
                "{title}"
            }
        }
    }
}
