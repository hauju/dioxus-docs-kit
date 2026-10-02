//! OpenAPI endpoint pages. Markup mirrors `dioxus-mdx`'s `EndpointPage`;
//! the expandable parts are `<details>`, so no runtime script is needed.

use dioxus_mdx::lucide::LdChevronRight;
use dioxus_mdx::{
    ApiOperation, ApiParameter, ApiRequestBody, ApiResponse, CodeBlockNode, HttpMethod,
    OpenApiSpec, SchemaDefinition,
};
use topcoat::{
    Result,
    view::{View, ViewExt, component, view},
};

use crate::content::highlighted;
use crate::icon::icon;

/// Main column (method, path, parameters, body, responses) and a sticky
/// side column with a curl request and an example response.
#[component]
pub(crate) async fn endpoint_page(
    operation: &ApiOperation,
    spec: &OpenApiSpec,
) -> Result<impl View> {
    let op = operation;
    let base_url = spec
        .servers
        .first()
        .map(|s| s.url.as_str())
        .unwrap_or("https://api.example.com");
    let curl = sample(op.generate_curl(base_url), "bash");
    let response_example = op.generate_response_example();

    Ok(view! {
        <div class="flex flex-col lg:flex-row gap-0">
            <div class="flex-1 min-w-0 px-8 py-12 lg:px-12">
                <div class="max-w-2xl">
                    <div class="flex items-center gap-3 mb-6">
                        <span class=(format!("px-3 py-1.5 rounded-lg font-mono text-sm font-bold border {}", op.method.bg_class()))>
                            (op.method.as_str())
                        </span>
                        <code class="font-mono text-lg text-base-content">(op.path.as_str())</code>
                        if op.deprecated {
                            <span class="badge badge-warning badge-sm">"deprecated"</span>
                        }
                    </div>
                    if let Some(summary) = &op.summary {
                        <h1 class="text-3xl font-bold tracking-tight mb-3">(summary.as_str())</h1>
                    }
                    if let Some(desc) = &op.description {
                        <p class="text-base text-base-content/70 mb-6 leading-relaxed">(desc.as_str())</p>
                    }
                    <div class="mb-8 flex items-center gap-2">
                        <span class="text-xs text-base-content/50 font-semibold uppercase tracking-wider">"Base URL"</span>
                        <code class="text-sm font-mono text-base-content/70 bg-base-200 px-2 py-1 rounded break-all">(base_url)</code>
                    </div>
                    if !op.parameters.is_empty() {
                        <div class="mb-8">
                            <h2 class="text-lg font-semibold mb-4 pb-2 border-b border-base-300">"Parameters"</h2>
                            <div class="space-y-1">
                                for param in &op.parameters {
                                    parameter_item(param: param)
                                }
                            </div>
                        </div>
                    }
                    if let Some(body) = &op.request_body {
                        <div class="mb-8">
                            <h2 class="text-lg font-semibold mb-4 pb-2 border-b border-base-300">"Request Body"</h2>
                            request_body(body: body)
                        </div>
                    }
                    if !op.responses.is_empty() {
                        <div class="mb-8">
                            <h2 class="text-lg font-semibold mb-4 pb-2 border-b border-base-300">"Responses"</h2>
                            <div class="space-y-2">
                                for response in &op.responses {
                                    response_item(response: response)
                                }
                            </div>
                        </div>
                    }
                </div>
            </div>
            <aside class="lg:w-[45%] lg:shrink-0 lg:border-l border-base-300 bg-base-200/20">
                <div class="lg:sticky lg:top-12 lg:h-[calc(100vh-3rem)] lg:overflow-y-auto p-6 space-y-6">
                    <div>
                        <h3 class="text-sm font-semibold text-base-content/70 uppercase tracking-wider mb-3">"Request"</h3>
                        <div class="rounded-lg border border-base-300 overflow-hidden">
                            <div class="px-3 py-2 bg-base-300/50 border-b border-base-300 flex items-center gap-2">
                                method_badge(method: op.method)
                                <code class="text-xs font-mono text-base-content/70 truncate">(op.path.as_str())</code>
                            </div>
                            <div class="dk-code-block-body bg-base-200">highlighted(block: &curl)</div>
                        </div>
                    </div>
                    if let Some((status, json)) = response_example {
                        let status_color = if status.starts_with('2') {
                            "badge-success"
                        } else if status.starts_with('3') {
                            "badge-info"
                        } else {
                            "badge-ghost"
                        };
                        let json = sample(json, "json");
                        <div>
                            <h3 class="text-sm font-semibold text-base-content/70 uppercase tracking-wider mb-3">"Response"</h3>
                            <div class="rounded-lg border border-base-300 overflow-hidden">
                                <div class="px-3 py-2 bg-base-300/50 border-b border-base-300 flex items-center gap-2">
                                    <span class=(format!("badge {status_color} badge-sm font-mono font-bold"))>(status)</span>
                                    <span class="text-xs text-base-content/50">"application/json"</span>
                                </div>
                                <div class="dk-code-block-body bg-base-200 max-h-[60vh] overflow-y-auto">highlighted(block: &json)</div>
                            </div>
                        </div>
                    }
                </div>
            </aside>
        </div>
    })
}

fn sample(code: String, language: &str) -> CodeBlockNode {
    CodeBlockNode {
        language: Some(language.to_string()),
        code,
        filename: None,
    }
}

#[component]
async fn method_badge(method: HttpMethod) -> Result<impl View> {
    Ok(view! {
        <span class=(format!("badge {} badge-sm font-mono font-bold", method.badge_class()))>(method.as_str())</span>
    })
}

#[component]
async fn parameter_item(param: &ApiParameter) -> Result<impl View> {
    Ok(view! {
        <div class="border-b border-base-300 py-3 first:pt-0 last:border-b-0">
            <div class="flex items-center gap-2 flex-wrap">
                <code class="font-mono font-semibold text-primary">(param.name.as_str())</code>
                <span class=(format!("badge {} badge-sm badge-outline", param.location.badge_class()))>(param.location.as_str())</span>
                if let Some(schema) = &param.schema {
                    <span class="text-xs px-2 py-0.5 rounded-full bg-base-300 text-base-content/70">(schema.display_type())</span>
                }
                if param.required {
                    <span class="text-xs px-2 py-0.5 rounded-full bg-error/20 text-error">"required"</span>
                }
                if param.deprecated {
                    <span class="text-xs px-2 py-0.5 rounded-full bg-warning/20 text-warning line-through">"deprecated"</span>
                }
            </div>
            if let Some(desc) = &param.description {
                <p class="mt-2 text-sm text-base-content/70">(desc.as_str())</p>
            }
            if let Some(schema) = param.schema.as_ref().filter(|s| s.is_complex()) {
                <div class="mt-2">schema_view(schema: schema, depth: 1, name: None, required: false, open: false)</div>
            }
            if let Some(example) = &param.example {
                <div class="mt-2">
                    <span class="text-xs text-base-content/50">"Example: "</span>
                    <code class="text-xs font-mono text-secondary">(example.as_str())</code>
                </div>
            }
        </div>
    })
}

#[component]
async fn request_body(body: &ApiRequestBody) -> Result<impl View> {
    Ok(view! {
        <div class="space-y-3">
            <div class="flex items-center gap-2">
                if body.required {
                    <span class="text-xs px-2 py-0.5 rounded-full bg-error/20 text-error">"required"</span>
                } else {
                    <span class="text-xs px-2 py-0.5 rounded-full bg-base-300 text-base-content/50">"optional"</span>
                }
            </div>
            if let Some(desc) = &body.description {
                <p class="text-sm text-base-content/70">(desc.as_str())</p>
            }
            for content in &body.content {
                <div class="border border-base-300 rounded-lg overflow-hidden">
                    <div class="px-3 py-2 bg-base-200 border-b border-base-300">
                        <code class="text-xs font-mono text-base-content/70">(content.media_type.as_str())</code>
                    </div>
                    if let Some(schema) = &content.schema {
                        <div class="p-3">schema_view(schema: schema, depth: 0, name: None, required: false, open: true)</div>
                    }
                    if let Some(example) = &content.example {
                        <div class="px-3 py-2 border-t border-base-300 bg-base-200/50">
                            <span class="text-xs text-base-content/50 font-semibold">"Example"</span>
                            <pre class="mt-1 text-xs font-mono text-secondary overflow-x-auto">(example.as_str())</pre>
                        </div>
                    }
                </div>
            }
        </div>
    })
}

#[component]
async fn response_item(response: &ApiResponse) -> Result<impl View> {
    let header = view! {
        <span class=(format!("badge {} badge-sm font-mono font-bold", response.status_badge_class()))>(response.status_code.as_str())</span>
        <span class="text-sm text-base-content/70 flex-1">(response.description.as_str())</span>
    };
    Ok(view! {
        if response.content.is_empty() {
            <div class="border border-base-300 rounded-lg overflow-hidden">
                <div class="w-full flex items-center gap-3 px-3 py-2">(header)</div>
            </div>
        } else {
            <details class="group border border-base-300 rounded-lg overflow-hidden">
                <summary class="w-full flex items-center gap-3 px-3 py-2 text-left cursor-pointer list-none [&::-webkit-details-marker]:hidden hover:bg-base-200 transition-colors">
                    icon(glyph: LdChevronRight, class: "size-4 text-base-content/50 transition-transform group-open:rotate-90")
                    (header)
                </summary>
                <div class="border-t border-base-300 bg-base-200/30">
                    for content in &response.content {
                        <div class="p-3">
                            <div class="mb-2"><code class="text-xs font-mono text-base-content/50">(content.media_type.as_str())</code></div>
                            if let Some(schema) = &content.schema {
                                schema_view(schema: schema, depth: 0, name: None, required: false, open: true)
                            }
                            if let Some(example) = &content.example {
                                <div class="mt-3 p-2 bg-base-300 rounded">
                                    <span class="text-xs text-base-content/50 font-semibold">"Example"</span>
                                    <pre class="mt-1 text-xs font-mono text-secondary overflow-x-auto whitespace-pre-wrap">(example.as_str())</pre>
                                </div>
                            }
                        </div>
                    }
                </div>
            </details>
        }
    })
}

/// A schema node: type, flags, enum/default/example, then its properties,
/// array items and composition variants. Nested parts sit in a `<details>`,
/// open at the top level (`open`) and collapsed below it, as in the Dioxus kit.
#[component]
async fn schema_view(
    schema: &SchemaDefinition,
    depth: usize,
    name: Option<&str>,
    required: bool,
    open: bool,
) -> Result<impl View> {
    let indent = if depth > 0 {
        "py-1.5 ml-4 border-l-2 border-base-300 pl-3"
    } else {
        "py-1.5"
    };
    let complex_items = schema.items.as_deref().filter(|i| i.is_complex());
    let has_children = !schema.properties.is_empty()
        || complex_items.is_some()
        || !schema.one_of.is_empty()
        || !schema.any_of.is_empty()
        || !schema.all_of.is_empty();
    let summary = view! {
        if let Some(name) = name {
            <code class="font-mono font-semibold text-primary text-sm">(name)</code>
        }
        <span class="text-xs px-2 py-0.5 rounded-full bg-base-300 text-base-content/70">(schema.display_type())</span>
        if required {
            <span class="text-xs px-2 py-0.5 rounded-full bg-error/20 text-error">"required"</span>
        }
        if schema.nullable {
            <span class="text-xs px-2 py-0.5 rounded-full bg-base-300 text-base-content/50">"nullable"</span>
        }
        if let Some(format) = &schema.format {
            <span class="text-xs text-base-content/50">"(" (format.as_str()) ")"</span>
        }
    };
    let details = view! {
        if let Some(desc) = &schema.description {
            <p class="text-sm text-base-content/70 mt-1">(desc.as_str())</p>
        }
        if !schema.enum_values.is_empty() {
            <div class="mt-1 flex items-center gap-2 flex-wrap">
                <span class="text-xs text-base-content/50">"Enum:"</span>
                for value in &schema.enum_values {
                    <code class="text-xs px-1.5 py-0.5 rounded bg-base-300 font-mono">(value.as_str())</code>
                }
            </div>
        }
        if let Some(default) = &schema.default {
            <div class="mt-1">
                <span class="text-xs text-base-content/50">"Default: "</span>
                <code class="text-xs font-mono text-primary">(default.as_str())</code>
            </div>
        }
        if let Some(example) = &schema.example {
            <div class="mt-1">
                <span class="text-xs text-base-content/50">"Example: "</span>
                <code class="text-xs font-mono text-secondary">(example.as_str())</code>
            </div>
        }
    };
    let children = view! {
        if !schema.properties.is_empty() {
            <div class="mt-2">
                for (prop_name, prop) in &schema.properties {
                    schema_view(schema: prop, depth: depth + 1, name: Some(prop_name.as_str()), required: schema.required.contains(prop_name), open: false)
                }
            </div>
        }
        if let Some(items) = complex_items {
            <div class="mt-2">
                <span class="text-xs text-base-content/50 ml-4">"Array items:"</span>
                schema_view(schema: items, depth: depth + 1, name: None, required: false, open: false)
            </div>
        }
        for (label, variants) in [("One of:", &schema.one_of), ("Any of:", &schema.any_of), ("All of:", &schema.all_of)] {
            if !variants.is_empty() {
                <div class="mt-2 ml-4">
                    <span class="text-xs text-base-content/50 font-semibold">(label)</span>
                    for variant in variants {
                        schema_view(schema: variant, depth: depth + 1, name: None, required: false, open: false)
                    }
                </div>
            }
        }
    };
    Ok(view! {
        <div class=(indent)>
            if has_children {
                <details class="group" open=(open || depth == 0)>
                    <summary class="flex items-center gap-2 flex-wrap cursor-pointer list-none [&::-webkit-details-marker]:hidden">
                        icon(glyph: LdChevronRight, class: "size-4 text-base-content/50 transition-transform group-open:rotate-90")
                        (summary)
                    </summary>
                    (details)
                    (children)
                </details>
            } else {
                <div class="flex items-center gap-2 flex-wrap">(summary)</div>
                (details)
            }
        </div>
    }
    // Recursive through nested properties and variants.
    .boxed())
}
