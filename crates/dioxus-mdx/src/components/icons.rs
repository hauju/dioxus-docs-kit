//! Icon mapping utilities for MDX components.
//!
//! Maps common icon names from Mintlify/FontAwesome style to Lucide icons.

use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::ld_icons::*};

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
    let icon_class = class;

    match name.as_str() {
        "code" => rsx! { Icon { class: icon_class, icon: LdCode } },
        "brain-circuit" | "brain" => rsx! { Icon { class: icon_class, icon: LdBrainCircuit } },
        "folder" => rsx! { Icon { class: icon_class, icon: LdFolder } },
        "list" => rsx! { Icon { class: icon_class, icon: LdList } },
        "file" => rsx! { Icon { class: icon_class, icon: LdFile } },
        "plus" | "plus-circle" => rsx! { Icon { class: icon_class, icon: LdPlus } },
        "pen" | "pencil" => rsx! { Icon { class: icon_class, icon: LdPencil } },
        "trash" | "trash-alt" => rsx! { Icon { class: icon_class, icon: LdTrash } },
        "thumbs-up" => rsx! { Icon { class: icon_class, icon: LdThumbsUp } },
        "star" => rsx! { Icon { class: icon_class, icon: LdStar } },
        "chart-bar" | "chart-line" | "chart-simple" => {
            rsx! { Icon { class: icon_class, icon: LdBarChart } }
        }
        "book" => rsx! { Icon { class: icon_class, icon: LdBook } },
        "puzzle-piece" => rsx! { Icon { class: icon_class, icon: LdPuzzle } },
        "shield-check" => rsx! { Icon { class: icon_class, icon: LdShieldCheck } },
        "list-check" => rsx! { Icon { class: icon_class, icon: LdListChecks } },
        "palette" => rsx! { Icon { class: icon_class, icon: LdPalette } },
        "rocket" => rsx! { Icon { class: icon_class, icon: LdRocket } },
        "settings" | "cog" => rsx! { Icon { class: icon_class, icon: LdSettings } },
        "user-plus" => rsx! { Icon { class: icon_class, icon: LdUserPlus } },
        "folder-plus" => rsx! { Icon { class: icon_class, icon: LdFolderPlus } },
        "paste" | "clipboard-paste" => rsx! { Icon { class: icon_class, icon: LdClipboardPaste } },
        "browser" | "globe" => rsx! { Icon { class: icon_class, icon: LdGlobe } },
        "cart-shopping" | "shopping-cart" => {
            rsx! { Icon { class: icon_class, icon: LdShoppingCart } }
        }
        "circle-question" | "help" => rsx! { Icon { class: icon_class, icon: LdCircleHelp } },
        "circle-exclamation" | "alert" => rsx! { Icon { class: icon_class, icon: LdCircleAlert } },
        "react" | "atom" => rsx! { Icon { class: icon_class, icon: LdAtom } },
        "vuejs" | "vue" | "component" => rsx! { Icon { class: icon_class, icon: LdComponent } },
        "angular" | "triangle" => rsx! { Icon { class: icon_class, icon: LdTriangle } },
        "wordpress" | "pen-tool" => rsx! { Icon { class: icon_class, icon: LdPenTool } },
        "ghost" => rsx! { Icon { class: icon_class, icon: LdGhost } },
        "newspaper" => rsx! { Icon { class: icon_class, icon: LdNewspaper } },
        "github" => rsx! { Icon { class: icon_class, icon: LdGithub } },
        "shield" => rsx! { Icon { class: icon_class, icon: LdShield } },
        "key" => rsx! { Icon { class: icon_class, icon: LdKey } },
        "clock" => rsx! { Icon { class: icon_class, icon: LdClock } },
        "eye-slash" | "eye-off" => rsx! { Icon { class: icon_class, icon: LdEyeOff } },
        "arrows-left-right" | "arrow-left-right" => {
            rsx! { Icon { class: icon_class, icon: LdArrowLeftRight } }
        }
        "mobile" | "smartphone" => rsx! { Icon { class: icon_class, icon: LdSmartphone } },
        "lightbulb" => rsx! { Icon { class: icon_class, icon: LdLightbulb } },
        "info" => rsx! { Icon { class: icon_class, icon: LdInfo } },
        "warning" | "triangle-alert" => rsx! { Icon { class: icon_class, icon: LdTriangleAlert } },
        "check" => rsx! { Icon { class: icon_class, icon: LdCheck } },
        "copy" => rsx! { Icon { class: icon_class, icon: LdCopy } },
        "chevron-down" => rsx! { Icon { class: icon_class, icon: LdChevronDown } },
        "chevron-right" => rsx! { Icon { class: icon_class, icon: LdChevronRight } },
        "arrow-right" => rsx! { Icon { class: icon_class, icon: LdArrowRight } },
        // Additional icons for docs
        "users" | "team" => rsx! { Icon { class: icon_class, icon: LdUsers } },
        "robot" | "bot" => rsx! { Icon { class: icon_class, icon: LdBot } },
        "code-branch" | "git-branch" | "branch" => {
            rsx! { Icon { class: icon_class, icon: LdGitBranch } }
        }
        "link" => rsx! { Icon { class: icon_class, icon: LdLink } },
        "image" | "picture" => rsx! { Icon { class: icon_class, icon: LdImage } },
        "camera" | "screenshot" => rsx! { Icon { class: icon_class, icon: LdCamera } },
        "terminal" | "command" => rsx! { Icon { class: icon_class, icon: LdTerminal } },
        "download" => rsx! { Icon { class: icon_class, icon: LdDownload } },
        "upload" => rsx! { Icon { class: icon_class, icon: LdUpload } },
        "database" => rsx! { Icon { class: icon_class, icon: LdDatabase } },
        "server" => rsx! { Icon { class: icon_class, icon: LdServer } },
        "cloud" => rsx! { Icon { class: icon_class, icon: LdCloud } },
        "mail" | "email" | "envelope" => rsx! { Icon { class: icon_class, icon: LdMail } },
        "lock" | "unlock" => rsx! { Icon { class: icon_class, icon: LdLock } },
        "search" | "magnifying-glass" => rsx! { Icon { class: icon_class, icon: LdSearch } },
        "home" | "house" => rsx! { Icon { class: icon_class, icon: LdHome } },
        "external-link" => rsx! { Icon { class: icon_class, icon: LdExternalLink } },
        "refresh" | "rotate" => rsx! { Icon { class: icon_class, icon: LdRefreshCw } },
        "play" => rsx! { Icon { class: icon_class, icon: LdPlay } },
        "pause" => rsx! { Icon { class: icon_class, icon: LdPause } },
        "stop" | "square" => rsx! { Icon { class: icon_class, icon: LdSquare } },
        "message" | "comment" => rsx! { Icon { class: icon_class, icon: LdMessageSquare } },
        "bell" | "notification" => rsx! { Icon { class: icon_class, icon: LdBell } },
        "tag" | "label" => rsx! { Icon { class: icon_class, icon: LdTag } },
        "bookmark" => rsx! { Icon { class: icon_class, icon: LdBookmark } },
        "heart" | "favorite" => rsx! { Icon { class: icon_class, icon: LdHeart } },
        "filter" => rsx! { Icon { class: icon_class, icon: LdFilter } },
        "sort" | "arrow-up-down" => rsx! { Icon { class: icon_class, icon: LdArrowUpDown } },
        "zap" | "bolt" | "lightning" => rsx! { Icon { class: icon_class, icon: LdZap } },
        _ => rsx! { Icon { class: icon_class, icon: LdCircle } },
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
