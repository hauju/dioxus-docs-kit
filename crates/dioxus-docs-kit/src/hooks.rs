use dioxus::prelude::*;

use crate::DocsContext;
use crate::components::DrawerOpen;
use crate::registry::DocsRegistry;

/// Signals returned by [`use_docs_providers`] so the consumer's header RSX
/// can reference them (e.g. to wire up a search button or drawer toggle).
pub struct DocsProviders {
    pub search_open: Signal<bool>,
    pub drawer_open: Signal<bool>,
}

/// One-call setup for all the context providers that `DocsLayout` and its
/// children expect.
///
/// Call this in your docs layout wrapper **before** rendering `DocsLayout`:
///
/// ```rust,ignore
/// let providers = use_docs_providers(&*DOCS, docs_ctx);
/// // Use providers.search_open / providers.drawer_open in your header RSX
/// ```
///
/// This replaces the manual calls to:
/// - `use_context_provider(|| registry)`
/// - `use_context_provider(|| docs_ctx)`
/// - `use_signal(|| false)` × 2 + `use_context_provider` for search_open / DrawerOpen
pub fn use_docs_providers(registry: &'static DocsRegistry, docs_ctx: DocsContext) -> DocsProviders {
    use_context_provider(|| registry);
    use_context_provider(|| docs_ctx);

    let search_open = use_signal(|| false);
    let drawer_open = use_signal(|| false);

    use_context_provider(|| search_open);
    use_context_provider(|| DrawerOpen(drawer_open));

    DocsProviders {
        search_open,
        drawer_open,
    }
}
