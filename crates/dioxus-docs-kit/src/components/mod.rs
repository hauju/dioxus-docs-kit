pub mod blog;
mod copy_page;
mod docs_layout;
mod docs_meta;
mod docs_page;
mod mobile_drawer;
mod page_nav;
mod search_modal;
mod search_shell;
pub(crate) mod seo;
pub(crate) mod shared;
mod sidebar;
mod theme_toggle;

pub use copy_page::CopyPageButton;
pub use docs_layout::{
    ActiveTab, CurrentTheme, DocsLayout, DocsVariant, DrawerOpen, LayoutOffsets, SearchButton,
    SearchOpen,
};
pub use docs_meta::DocsPageMeta;
pub use docs_page::DocsPageContent;
pub use mobile_drawer::MobileDrawer;
pub use page_nav::DocsPageNav;
pub use search_modal::SearchModal;
pub use shared::use_theme_provider;
pub use sidebar::DocsSidebar;
pub use theme_toggle::ThemeToggle;

// Blog component re-exports
pub use blog::{
    AuthorInfo, BlogCard, BlogIndexMeta, BlogLayout, BlogList, BlogMobileDrawer, BlogPostMeta,
    BlogPostNav, BlogPostView, BlogSearchButton, BlogSearchModal, BlogThemeToggle,
    ReadingProgressBar, ReadingTimeBadge, RelatedPosts, TagFilter,
};
