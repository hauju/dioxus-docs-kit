//! Icon mapping utilities for MDX components.
//!
//! Maps common icon names from Mintlify/FontAwesome style to Lucide icons.

use crate::lucide::*;
use dioxus::prelude::*;

/// Render an icon by name.
///
/// Maps common icon names (e.g., "code", "folder", "star") to Lucide icons.
/// Returns a default icon if the name is not recognized.
#[component]
pub fn MdxIcon(
    /// Icon name (e.g., "code", "brain-circuit", "folder").
    name: String,
    /// CSS classes to apply (default: "size-5").
    #[props(default = "size-5".to_string())]
    class: String,
) -> Element {
    rsx! { Icon { class, icon: lucide_for(&name) } }
}

/// Map an MDX `icon="…"` name to a vendored Lucide icon.
///
/// Unrecognised names fall back to [`LdCircle`], which is a blank ring — so
/// every name the docs actually use needs an arm here.
fn lucide_for(name: &str) -> LucideIcon {
    match name {
        "code" => LdCode,
        "brain-circuit" | "brain" => LdBrainCircuit,
        "folder" => LdFolder,
        "list" => LdList,
        "file" => LdFile,
        "plus" | "plus-circle" => LdPlus,
        "pen" | "pencil" => LdPencil,
        "trash" | "trash-alt" => LdTrash,
        "thumbs-up" => LdThumbsUp,
        "star" => LdStar,
        "chart-bar" | "chart-line" | "chart-simple" => LdBarChart,
        "book" => LdBook,
        "puzzle-piece" => LdPuzzle,
        "shield-check" => LdShieldCheck,
        "list-check" => LdListChecks,
        "palette" => LdPalette,
        "rocket" => LdRocket,
        "settings" | "cog" => LdSettings,
        "user-plus" => LdUserPlus,
        "folder-plus" => LdFolderPlus,
        "paste" | "clipboard-paste" => LdClipboardPaste,
        "browser" | "globe" => LdGlobe,
        "cart-shopping" | "shopping-cart" => LdShoppingCart,
        "circle-question" | "help" => LdCircleHelp,
        "circle-exclamation" | "alert" => LdCircleAlert,
        "react" | "atom" => LdAtom,
        "vuejs" | "vue" | "component" => LdComponent,
        "angular" | "triangle" => LdTriangle,
        "wordpress" | "pen-tool" => LdPenTool,
        "ghost" => LdGhost,
        "newspaper" => LdNewspaper,
        "github" => LdGithub,
        "shield" => LdShield,
        "key" => LdKey,
        "clock" => LdClock,
        "eye-slash" | "eye-off" => LdEyeOff,
        "arrows-left-right" | "arrow-left-right" => LdArrowLeftRight,
        "mobile" | "smartphone" => LdSmartphone,
        "lightbulb" => LdLightbulb,
        "info" => LdInfo,
        "warning" | "triangle-alert" => LdTriangleAlert,
        "check" => LdCheck,
        "copy" => LdCopy,
        "chevron-down" => LdChevronDown,
        "chevron-right" => LdChevronRight,
        "arrow-right" => LdArrowRight,
        // Additional icons for docs
        "users" | "team" => LdUsers,
        "robot" | "bot" => LdBot,
        "code-branch" | "git-branch" | "branch" => LdGitBranch,
        "link" => LdLink,
        "image" | "picture" => LdImage,
        "camera" | "screenshot" => LdCamera,
        "terminal" | "command" => LdTerminal,
        "download" => LdDownload,
        "upload" => LdUpload,
        "database" => LdDatabase,
        "server" => LdServer,
        "cloud" => LdCloud,
        "mail" | "email" | "envelope" => LdMail,
        "lock" | "unlock" => LdLock,
        "search" | "magnifying-glass" => LdSearch,
        "home" | "house" => LdHome,
        "external-link" => LdExternalLink,
        "refresh" | "rotate" => LdRefreshCw,
        "play" => LdPlay,
        "pause" => LdPause,
        "stop" | "square" => LdSquare,
        "message" | "comment" => LdMessageSquare,
        "bell" | "notification" => LdBell,
        "tag" | "label" => LdTag,
        "bookmark" => LdBookmark,
        "heart" | "favorite" => LdHeart,
        "filter" => LdFilter,
        "sort" | "arrow-up-down" => LdArrowUpDown,
        "zap" | "bolt" | "lightning" => LdZap,
        // Names the example docs and blog use that had no arm before.
        "file-text" | "file-lines" => LdFileText,
        "layout" | "layout-dashboard" => LdLayoutDashboard,
        "code-2" | "square-code" => LdSquareCode,
        "list-ordered" => LdListOrdered,
        "message-circle" => LdMessageCircle,
        "panel-left" | "sidebar" => LdPanelLeft,
        "user" | "account" => LdUser,
        _ => LdCircle,
    }
}

/// Render a callout-specific icon.
#[component]
pub fn CalloutIcon(
    /// Callout type: "tip", "note", "warning", or "info".
    callout_type: String,
    /// CSS classes (default: "size-5").
    #[props(default = "size-5".to_string())]
    class: String,
) -> Element {
    match callout_type.to_lowercase().as_str() {
        "tip" => rsx! { Icon { class, icon: LdLightbulb } },
        "note" => rsx! { Icon { class, icon: LdInfo } },
        "warning" => rsx! { Icon { class, icon: LdTriangleAlert } },
        "info" => rsx! { Icon { class, icon: LdInfo } },
        _ => rsx! { Icon { class, icon: LdInfo } },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every `icon="…"` name the example content uses, captured from
    /// `grep -rhno 'icon="[^"]*"' docs blog | sort -u` at the repo root.
    const USED_IN_DOCS: &[&str] = &[
        "arrow-right",
        "book",
        "clock",
        "code",
        "code-2",
        "file-text",
        "layout",
        "list",
        "list-ordered",
        "message-circle",
        "palette",
        "panel-left",
        "rocket",
        "search",
        "server",
        "shield",
        "star",
        "tag",
        "user",
        "zap",
    ];

    /// An unmapped name renders a blank ring, which reads as a broken icon.
    #[test]
    fn every_documented_icon_name_is_mapped() {
        for name in USED_IN_DOCS {
            assert_ne!(lucide_for(name), LdCircle, "icon name {name:?} is unmapped");
        }
    }

    #[test]
    fn unknown_names_fall_back() {
        assert_eq!(lucide_for("not-a-real-icon"), LdCircle);
    }
}
