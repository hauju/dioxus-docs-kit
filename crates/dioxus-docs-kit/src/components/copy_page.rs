use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::ld_icons::*};

/// "Copy page" button for MDX documentation pages.
///
/// Copies the page's raw Markdown source to the clipboard — the "copy page for
/// LLMs" pattern. On success it swaps to a "Copied" state (check icon) for ~2
/// seconds, then reverts.
///
/// The Markdown is handed to the clipboard via `document::eval` argument
/// passing (`eval.send`), never string-interpolated into the script, so
/// backticks and quotes in the source can't break the JS.
///
/// # Props
///
/// - `content`: Raw Markdown source of the current page.
///
/// # Stable public classes
///
/// Carries `dk-copy-page` so theme presets can target the button.
#[component]
pub fn CopyPageButton(content: String) -> Element {
    #[allow(unused_mut)]
    let mut copied = use_signal(|| false);

    rsx! {
        button {
            class: "dk-copy-page btn btn-ghost btn-sm gap-1.5 opacity-60 hover:opacity-100 transition-all duration-150 hover:bg-base-content/10 shrink-0",
            title: "Copy page as Markdown",
            onclick: move |_| {
                #[cfg(target_arch = "wasm32")]
                {
                    let content = content.clone();
                    spawn(async move {
                        // Send the Markdown into the script as an argument rather
                        // than interpolating it — the source contains backticks
                        // and quotes that would otherwise break the JS.
                        let eval = document::eval(
                            "const text = await dioxus.recv();\n\
                             await navigator.clipboard.writeText(text);",
                        );
                        let _ = eval.send(content);
                        copied.set(true);
                        gloo_timers::future::TimeoutFuture::new(2000).await;
                        copied.set(false);
                    });
                }
            },
            if copied() {
                Icon { class: "size-4 text-success", icon: LdCheck }
                span { class: "text-xs", "Copied!" }
            } else {
                Icon { class: "size-4", icon: LdCopy }
                span { class: "text-xs", "Copy page" }
            }
        }
    }
}
