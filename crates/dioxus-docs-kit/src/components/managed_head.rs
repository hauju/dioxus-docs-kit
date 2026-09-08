use dioxus::prelude::*;

/// A head element owned by the current docs or blog page.
#[derive(Clone, PartialEq)]
pub(crate) struct HeadTag {
    tag: &'static str,
    attributes: Vec<(&'static str, String)>,
    text: String,
}

impl HeadTag {
    pub fn meta(key: &'static str, name: &str, content: &str) -> Self {
        Self {
            tag: "meta",
            attributes: vec![(key, name.into()), ("content", content.into())],
            text: String::new(),
        }
    }

    pub fn link(rel: &str, href: &str, mime: Option<&str>) -> Self {
        let mut attributes = vec![("rel", rel.into()), ("href", href.into())];
        if let Some(mime) = mime {
            attributes.push(("type", mime.into()));
        }
        Self {
            tag: "link",
            attributes,
            text: String::new(),
        }
    }

    pub fn jsonld(text: String) -> Self {
        Self {
            tag: "script",
            attributes: vec![("type", "application/ld+json".into())],
            text,
        }
    }

    fn attr(&self, name: &str) -> Option<String> {
        self.attributes
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| value.clone())
    }
}

/// Dioxus 0.7 head components insert once and do not remove on unmount.
/// Keep their initial props for SSR/hydration; own updates and cleanup on the client.
#[component]
pub(crate) fn ManagedPageHead(title: String, tags: Vec<HeadTag>) -> Element {
    let initial = use_hook(|| tags.clone());
    let owner = dioxus::core::current_scope_id().0;
    use_effect(use_reactive!(|tags| {
        let payload: Vec<_> = tags
            .iter()
            .map(|tag| {
                serde_json::json!({
                    "tag": tag.tag, "attributes": tag.attributes, "text": tag.text,
                })
            })
            .collect();
        let payload = serde_json::to_string(&payload).expect("head tags serialize");
        // Defer until Dioxus has inserted the initial head components. A generation
        // guard prevents queued work from a previous route from taking ownership.
        document::eval(&format!(
            r#"
            const generation = (window.__dkPageHeadGeneration || 0) + 1;
            window.__dkPageHeadGeneration = generation;
            window.__dkPageHeadOwner = {owner};
            requestAnimationFrame(() => {{
                if (window.__dkPageHeadGeneration !== generation) return;
                document.head.querySelectorAll('[data-dk-page-head]').forEach(node => node.remove());
                for (const item of {payload}) {{
                    const node = document.createElement(item.tag);
                    for (const [key, value] of item.attributes) node.setAttribute(key, value);
                    node.setAttribute('data-dk-page-head', '{owner}');
                    node.textContent = item.text;
                    document.head.appendChild(node);
                }}
            }});
        "#
        ));
    }));
    use_drop(move || {
        document::eval(&format!(
            r#"
            if (window.__dkPageHeadOwner === {owner}) {{
                window.__dkPageHeadGeneration = (window.__dkPageHeadGeneration || 0) + 1;
                document.head.querySelectorAll('[data-dk-page-head]').forEach(node => node.remove());
            }}
        "#
        ));
    });
    rsx! {
        document::Title { "{title}" }
        for (index, tag) in initial.iter().enumerate() {
            match tag.tag {
                "meta" => rsx! { document::Meta {
                    key: "{index}", name: tag.attr("name"), property: tag.attr("property"), content: tag.attr("content"),
                    "data-dk-page-head": "initial",
                } },
                "link" => rsx! { document::Link {
                    key: "{index}", rel: tag.attr("rel"), href: tag.attr("href"), r#type: tag.attr("type"),
                    "data-dk-page-head": "initial",
                } },
                _ => rsx! { document::Script {
                    key: "{index}", r#type: "application/ld+json", "data-dk-page-head": "initial", "{tag.text}"
                } },
            }
        }
    }
}

/// Not-found responses always carry noindex, even when automatic page metadata
/// is disabled. Setting the status during render also covers direct SSR loads.
#[component]
pub(crate) fn NotFoundMeta(title: String) -> Element {
    #[cfg(feature = "server")]
    if let Some(mut context) = dioxus_fullstack_core::FullstackContext::current() {
        context.set_current_http_status(dioxus_fullstack_core::HttpError::new(
            dioxus::server::http::StatusCode::NOT_FOUND,
            title.clone(),
        ));
    }
    rsx! { ManagedPageHead { title, tags: vec![HeadTag::meta("name", "robots", "noindex")] } }
}
