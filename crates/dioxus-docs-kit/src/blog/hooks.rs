use dioxus::prelude::*;

use crate::BlogContext;
use crate::blog::registry::BlogRegistry;
use crate::components::DrawerOpen;

/// Signals returned by [`use_blog_providers`].
pub struct BlogProviders {
    pub search_open: Signal<bool>,
    pub drawer_open: Signal<bool>,
    pub active_tag: Signal<Option<String>>,
    pub current_page: Signal<usize>,
}

/// One-call setup for all context providers that `BlogLayout` and its children expect.
pub fn use_blog_providers(
    registry: &'static BlogRegistry,
    blog_ctx: BlogContext,
) -> BlogProviders {
    use_context_provider(|| registry);
    use_context_provider(|| blog_ctx);

    let search_open = use_signal(|| false);
    let drawer_open = use_signal(|| false);
    let active_tag: Signal<Option<String>> = use_signal(|| None);
    let current_page = use_signal(|| 0usize);

    use_context_provider(|| search_open);
    use_context_provider(|| DrawerOpen(drawer_open));
    use_context_provider(|| active_tag);
    use_context_provider(|| current_page);

    BlogProviders {
        search_open,
        drawer_open,
        active_tag,
        current_page,
    }
}
