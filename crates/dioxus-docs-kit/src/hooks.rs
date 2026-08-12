use dioxus::prelude::*;

use crate::DocsContext;
use crate::components::{DrawerOpen, SearchOpen};
use crate::registry::DocsRegistry;

/// Signals returned by [`use_docs_providers`] so the consumer's header RSX
/// can reference them (e.g. to wire up a search button or drawer toggle).
pub struct DocsProviders {
    pub search_open: Signal<bool>,
    pub drawer_open: Signal<bool>,
}

/// Build a [`DocsContext`] from the current docs path as a plain `String`.
///
/// Prefer this over [`DocsContext::new`] when the path comes from your router.
/// `use_route::<Route>()` returns a plain value, not a signal, so wrapping it
/// yourself with `use_memo(move || ...)` reads no reactive source: the memo
/// runs once and never again, freezing the sidebar highlight, tab sync and
/// mobile-drawer auto-close on the first page visited. (`use_memo` paired with
/// [`use_reactive!`] is correct, but easy to forget.)
///
/// Taking the path by value removes the trap — this hook re-runs with your
/// layout component on every navigation and rewraps the path reactively.
///
/// ```rust,ignore
/// let route = use_route::<Route>();
/// let current_path = match route {
///     Route::DocsPage { slug } => slug.join("/"),
///     _ => String::new(),
/// };
///
/// let docs_ctx = use_docs_context(current_path, "/docs", Callback::new(move |path: String| {
///     nav.push(Route::DocsPage { slug: path.split('/').map(String::from).collect() });
/// }));
/// ```
pub fn use_docs_context(
    current_path: String,
    base_path: impl Into<String>,
    navigate: Callback<String>,
) -> DocsContext {
    let path = use_memo(use_reactive!(|current_path| current_path));
    DocsContext::new(path, base_path, navigate)
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

    use_context_provider(|| SearchOpen(search_open));
    use_context_provider(|| DrawerOpen(drawer_open));

    DocsProviders {
        search_open,
        drawer_open,
    }
}
