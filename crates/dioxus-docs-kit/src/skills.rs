//! Agent skills rendered as site pages. The registry and types live in
//! [`oxidt_docs_kit_core::skills`].

use dioxus::prelude::*;

use crate::SkillsContext;
use crate::components::{DrawerOpen, SearchOpen};

pub use oxidt_docs_kit_core::skills::{Skill, SkillsConfig, SkillsRegistry, SkillsSearchEntry};

/// Signals returned by [`use_skills_providers`], for wiring a custom header.
pub struct SkillsProviders {
    pub search_open: Signal<bool>,
    pub drawer_open: Signal<bool>,
}

/// One-call setup for the context `SkillsLayout` and the skill pages read.
/// Call it in your skills layout wrapper.
pub fn use_skills_providers(
    registry: &'static SkillsRegistry,
    ctx: SkillsContext,
) -> SkillsProviders {
    use_context_provider(|| registry);
    use_context_provider(|| ctx);

    let search_open = use_signal(|| false);
    let drawer_open = use_signal(|| false);
    use_context_provider(|| SearchOpen(search_open));
    use_context_provider(|| DrawerOpen(drawer_open));

    SkillsProviders {
        search_open,
        drawer_open,
    }
}
