use dioxus::prelude::*;

use crate::BlogContext;
use crate::blog::registry::BlogRegistry;
use crate::components::{DrawerOpen, SearchOpen};

/// Newtype wrapper for the active tag filter on the blog list page.
#[derive(Clone, Copy)]
pub struct ActiveTag(pub Signal<Option<String>>);

/// Newtype wrapper for the current blog list pagination page (0-based).
#[derive(Clone, Copy)]
pub struct CurrentPage(pub Signal<usize>);

/// Signals returned by [`use_blog_providers`].
pub struct BlogProviders {
    pub search_open: Signal<bool>,
    pub drawer_open: Signal<bool>,
    pub active_tag: Signal<Option<String>>,
    pub current_page: Signal<usize>,
}

/// One-call setup for all context providers that `BlogLayout` and its children expect.
pub fn use_blog_providers(registry: &'static BlogRegistry, blog_ctx: BlogContext) -> BlogProviders {
    use_context_provider(|| registry);
    use_context_provider(|| blog_ctx);

    let search_open = use_signal(|| false);
    let drawer_open = use_signal(|| false);
    let active_tag: Signal<Option<String>> = use_signal(|| None);
    let current_page = use_signal(|| 0usize);

    use_context_provider(|| SearchOpen(search_open));
    use_context_provider(|| DrawerOpen(drawer_open));
    use_context_provider(|| ActiveTag(active_tag));
    use_context_provider(|| CurrentPage(current_page));

    BlogProviders {
        search_open,
        drawer_open,
        active_tag,
        current_page,
    }
}
