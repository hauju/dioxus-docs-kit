use dioxus::prelude::*;

/// A thin reading progress bar fixed at the top of the viewport.
///
/// Uses a scroll event listener (RAF-throttled) to track reading progress.
#[component]
pub fn ReadingProgressBar() -> Element {
    let mut progress = use_signal(|| 0.0f64);

    use_effect(move || {
        spawn(async move {
            let mut eval = document::eval(
                r#"
                if (window.__dioxusDocsKitReadingProgressCleanup) {
                    window.__dioxusDocsKitReadingProgressCleanup();
                }
                let ticking = false;
                const handleScroll = () => {
                    if (!ticking) {
                        requestAnimationFrame(() => {
                            const docHeight = document.documentElement.scrollHeight - window.innerHeight;
                            const pct = docHeight > 0 ? (window.scrollY / docHeight) * 100 : 0;
                            dioxus.send(Math.min(Math.max(pct, 0), 100));
                            ticking = false;
                        });
                        ticking = true;
                    }
                };
                window.addEventListener('scroll', handleScroll, { passive: true });
                handleScroll();
                window.__dioxusDocsKitReadingProgressCleanup = () => {
                    window.removeEventListener('scroll', handleScroll);
                    delete window.__dioxusDocsKitReadingProgressCleanup;
                };
                while (true) { await new Promise(r => setTimeout(r, 1000000)); }
                "#,
            );
            while let Ok(val) = eval.recv::<f64>().await {
                progress.set(val);
            }
        });
    });

    use_drop(|| {
        spawn(async move {
            let _ = document::eval(
                r#"
                if (window.__dioxusDocsKitReadingProgressCleanup) {
                    window.__dioxusDocsKitReadingProgressCleanup();
                }
                "#,
            );
        });
    });

    rsx! {
        div { class: "fixed top-0 left-0 w-full z-[100] pointer-events-none",
            div {
                class: "h-0.5 bg-primary transition-[width] duration-75",
                style: "width: {progress()}%",
            }
        }
    }
}
