//! Distills nightly rustdoc JSON into the compact Rust API model consumed by
//! dioxus-docs-kit's `DocsConfig::with_rustdoc`.
//!
//! The raw rustdoc JSON (`cargo +nightly rustdoc -- -Z unstable-options
//! --output-format json`) is megabytes and format-version-locked, so it is
//! never embedded in the site. This module walks the public items reachable
//! from the crate root and emits a small, stable JSON model (see
//! [`MODEL_VERSION`]) that the consumer commits and passes to `with_rustdoc`.
//!
//! The output structs here mirror `dioxus_mdx::parser::rust_api` — the JSON is
//! the contract between the two crates (this build helper must not depend on
//! the UI crates).

use rustdoc_types as rd;
use serde::Serialize;
use std::collections::HashMap;

/// Version stamp written into the model so the renderer can reject
/// incompatible files with a clear message.
pub const MODEL_VERSION: u32 = 1;

// ============================================================================
// Output model (serialize side; mirrored in dioxus-mdx for deserialization)
// ============================================================================

#[derive(Serialize)]
struct Model {
    model_version: u32,
    crate_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    crate_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    crate_docs: Option<String>,
    items: Vec<ModelItem>,
}

#[derive(Serialize)]
struct ModelItem {
    slug: String,
    name: String,
    kind: &'static str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    path: Vec<String>,
    signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    docs: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    members: Vec<ModelMember>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    trait_impls: Vec<String>,
}

#[derive(Serialize)]
struct ModelMember {
    kind: &'static str,
    name: String,
    signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    docs: Option<String>,
}

/// Options for [`distill_rustdoc`].
#[derive(Default)]
pub struct DistillOptions {
    /// Skip public items whose name contains any of these substrings.
    ///
    /// Useful for macro-generated noise, e.g. `"Props"` hides the `*Props`
    /// structs the Dioxus `#[component]` macro derives for every component.
    pub exclude_name_parts: Vec<String>,
}

/// The item kinds surfaced in the model, in sidebar display order.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Struct,
    Enum,
    Trait,
    Function,
    TypeAlias,
    Constant,
    Macro,
}

impl Kind {
    fn as_str(self) -> &'static str {
        match self {
            Kind::Struct => "struct",
            Kind::Enum => "enum",
            Kind::Trait => "trait",
            Kind::Function => "function",
            Kind::TypeAlias => "type_alias",
            Kind::Constant => "constant",
            Kind::Macro => "macro",
        }
    }
}

// ============================================================================
// Entry point
// ============================================================================

/// Distill raw rustdoc JSON into the compact Rust API model JSON.
///
/// Fails with a descriptive message when the JSON was produced by a rustdoc
/// whose format version differs from the pinned `rustdoc-types` one.
pub fn distill_rustdoc(json: &str, options: &DistillOptions) -> Result<String, String> {
    let krate: rd::Crate = serde_json::from_str(json).map_err(|e| {
        // Surface a version mismatch before a confusing serde error.
        let version = serde_json::from_str::<serde_json::Value>(json)
            .ok()
            .and_then(|v| v.get("format_version").and_then(|f| f.as_u64()));
        match version {
            Some(found) if found != rd::FORMAT_VERSION as u64 => format!(
                "rustdoc JSON has format_version {found}, but this distiller understands \
                 {expected}. Regenerate the JSON with a nightly matching rustdoc-types \
                 {expected}, or update the rustdoc-types dependency.",
                expected = rd::FORMAT_VERSION
            ),
            _ => format!("failed to parse rustdoc JSON: {e}"),
        }
    })?;

    let root = krate
        .index
        .get(&krate.root)
        .ok_or("rustdoc JSON has no root module in its index")?;
    let crate_name = root.name.clone().unwrap_or_else(|| "crate".to_string());

    // Collect public leaf items reachable from the root, preferring the
    // shortest re-export path (the one closest to the crate root).
    let mut collected: HashMap<rd::Id, (Vec<String>, usize)> = HashMap::new();
    let mut order = 0usize;
    collect_module(&krate, krate.root, &[], &mut collected, &mut order, 0);

    let mut entries: Vec<(rd::Id, Vec<String>, usize)> = collected
        .into_iter()
        .map(|(id, (path, ord))| (id, path, ord))
        .collect();
    entries.sort_by_key(|(_, _, ord)| *ord);

    let mut items = Vec::new();
    for (id, path, _) in entries {
        let item = &krate.index[&id];
        let Some(name) = item.name.clone() else {
            continue;
        };
        if options.exclude_name_parts.iter().any(|p| name.contains(p)) {
            continue;
        }
        if let Some(model_item) = build_item(&krate, item, name, path) {
            items.push(model_item);
        }
    }

    assign_slugs(&mut items);

    let model = Model {
        model_version: MODEL_VERSION,
        crate_name,
        crate_version: krate.crate_version.clone(),
        crate_docs: root.docs.clone().map(|d| clean_docs(&d, &root.links)),
        items,
    };

    serde_json::to_string_pretty(&model).map_err(|e| format!("failed to serialize model: {e}"))
}

/// Recursively collect public leaf items from a module, following public
/// re-exports. `depth` guards against `pub use` cycles.
fn collect_module(
    krate: &rd::Crate,
    module_id: rd::Id,
    path: &[String],
    out: &mut HashMap<rd::Id, (Vec<String>, usize)>,
    order: &mut usize,
    depth: usize,
) {
    if depth > 16 {
        return;
    }
    let Some(rd::ItemEnum::Module(module)) = krate.index.get(&module_id).map(|i| &i.inner) else {
        return;
    };
    for child_id in &module.items {
        let Some(child) = krate.index.get(child_id) else {
            continue; // item from another crate
        };
        if !matches!(child.visibility, rd::Visibility::Public) {
            continue;
        }
        match &child.inner {
            rd::ItemEnum::Module(m) => {
                if m.is_stripped {
                    continue;
                }
                let mut sub = path.to_vec();
                sub.extend(child.name.clone());
                collect_module(krate, child.id, &sub, out, order, depth + 1);
            }
            rd::ItemEnum::Use(u) => {
                let Some(target_id) = u.id else { continue };
                let Some(target) = krate.index.get(&target_id) else {
                    continue; // re-export of an external item
                };
                if u.is_glob || matches!(target.inner, rd::ItemEnum::Module(_)) {
                    // Glob re-exports inline the module's items here; a plain
                    // module re-export exposes it as a submodule.
                    let sub = if u.is_glob {
                        path.to_vec()
                    } else {
                        let mut sub = path.to_vec();
                        sub.push(u.name.clone());
                        sub
                    };
                    collect_module(krate, target_id, &sub, out, order, depth + 1);
                } else {
                    record(target_id, target, path, out, order);
                }
            }
            _ => record(*child_id, child, path, out, order),
        }
    }
}

/// Record a leaf item, preferring the shortest module path when the same item
/// is reachable through several re-exports.
fn record(
    id: rd::Id,
    item: &rd::Item,
    path: &[String],
    out: &mut HashMap<rd::Id, (Vec<String>, usize)>,
    order: &mut usize,
) {
    if kind_of(&item.inner).is_none() {
        return;
    }
    match out.get(&id) {
        Some((existing, _)) if existing.len() <= path.len() => {}
        _ => {
            let ord = out.get(&id).map(|(_, o)| *o).unwrap_or_else(|| {
                *order += 1;
                *order
            });
            out.insert(id, (path.to_vec(), ord));
        }
    }
}

fn kind_of(inner: &rd::ItemEnum) -> Option<Kind> {
    Some(match inner {
        rd::ItemEnum::Struct(_) => Kind::Struct,
        rd::ItemEnum::Enum(_) => Kind::Enum,
        rd::ItemEnum::Trait(_) => Kind::Trait,
        rd::ItemEnum::Function(_) => Kind::Function,
        rd::ItemEnum::TypeAlias(_) => Kind::TypeAlias,
        rd::ItemEnum::Constant { .. } => Kind::Constant,
        rd::ItemEnum::Macro(_) => Kind::Macro,
        rd::ItemEnum::ProcMacro(_) => Kind::Macro,
        _ => return None,
    })
}

// ============================================================================
// Per-item model building
// ============================================================================

fn build_item(
    krate: &rd::Crate,
    item: &rd::Item,
    name: String,
    path: Vec<String>,
) -> Option<ModelItem> {
    let kind = kind_of(&item.inner)?;
    let docs = item.docs.clone().map(|d| clean_docs(&d, &item.links));

    let (signature, members, trait_impls) = match &item.inner {
        rd::ItemEnum::Struct(s) => {
            let generics = render_generics_decl(&s.generics);
            let signature = match &s.kind {
                rd::StructKind::Unit => format!("pub struct {name}{generics};"),
                rd::StructKind::Tuple(fields) => {
                    let parts: Vec<String> = fields
                        .iter()
                        .map(|f| match f.and_then(|id| krate.index.get(&id)) {
                            Some(field) => match &field.inner {
                                rd::ItemEnum::StructField(t) => format!("pub {}", render_type(t)),
                                _ => "_".to_string(),
                            },
                            None => "/* private */".to_string(),
                        })
                        .collect();
                    format!("pub struct {name}{generics}({});", parts.join(", "))
                }
                rd::StructKind::Plain {
                    has_stripped_fields,
                    ..
                } => {
                    if *has_stripped_fields {
                        format!("pub struct {name}{generics} {{ /* private fields */ }}")
                    } else {
                        format!("pub struct {name}{generics}")
                    }
                }
            };
            let mut members = Vec::new();
            if let rd::StructKind::Plain { fields, .. } = &s.kind {
                members.extend(field_members(krate, fields));
            }
            let (methods, traits) = impl_members(krate, &s.impls);
            members.extend(methods);
            (signature, members, traits)
        }
        rd::ItemEnum::Enum(e) => {
            let generics = render_generics_decl(&e.generics);
            let signature = format!("pub enum {name}{generics}");
            let mut members = Vec::new();
            for variant_id in &e.variants {
                let Some(variant) = krate.index.get(variant_id) else {
                    continue;
                };
                let Some(vname) = variant.name.clone() else {
                    continue;
                };
                let rd::ItemEnum::Variant(v) = &variant.inner else {
                    continue;
                };
                let sig = match &v.kind {
                    rd::VariantKind::Plain => vname.clone(),
                    rd::VariantKind::Tuple(fields) => {
                        let parts: Vec<String> = fields
                            .iter()
                            .map(|f| match f.and_then(|id| krate.index.get(&id)) {
                                Some(field) => match &field.inner {
                                    rd::ItemEnum::StructField(t) => render_type(t),
                                    _ => "_".to_string(),
                                },
                                None => "_".to_string(),
                            })
                            .collect();
                        format!("{vname}({})", parts.join(", "))
                    }
                    rd::VariantKind::Struct { fields, .. } => {
                        let parts: Vec<String> = fields
                            .iter()
                            .filter_map(|id| krate.index.get(id))
                            .filter_map(|field| match (&field.name, &field.inner) {
                                (Some(n), rd::ItemEnum::StructField(t)) => {
                                    Some(format!("{n}: {}", render_type(t)))
                                }
                                _ => None,
                            })
                            .collect();
                        format!("{vname} {{ {} }}", parts.join(", "))
                    }
                };
                members.push(ModelMember {
                    kind: "variant",
                    name: vname,
                    signature: sig,
                    docs: variant.docs.clone().map(|d| clean_docs(&d, &variant.links)),
                });
            }
            let (methods, traits) = impl_members(krate, &e.impls);
            members.extend(methods);
            (signature, members, traits)
        }
        rd::ItemEnum::Trait(t) => {
            let generics = render_generics_decl(&t.generics);
            let bounds = render_bounds(&t.bounds);
            let unsafe_prefix = if t.is_unsafe { "unsafe " } else { "" };
            let signature = if bounds.is_empty() {
                format!("pub {unsafe_prefix}trait {name}{generics}")
            } else {
                format!("pub {unsafe_prefix}trait {name}{generics}: {bounds}")
            };
            let mut members = Vec::new();
            for assoc_id in &t.items {
                let Some(assoc) = krate.index.get(assoc_id) else {
                    continue;
                };
                let Some(aname) = assoc.name.clone() else {
                    continue;
                };
                let docs = assoc.docs.clone().map(|d| clean_docs(&d, &assoc.links));
                match &assoc.inner {
                    rd::ItemEnum::Function(f) => members.push(ModelMember {
                        kind: "method",
                        name: aname.clone(),
                        signature: render_function(&aname, f, ""),
                        docs,
                    }),
                    rd::ItemEnum::AssocType { bounds, type_, .. } => {
                        let bounds = render_bounds(bounds);
                        let mut sig = format!("type {aname}");
                        if !bounds.is_empty() {
                            sig.push_str(&format!(": {bounds}"));
                        }
                        if let Some(t) = type_ {
                            sig.push_str(&format!(" = {}", render_type(t)));
                        }
                        members.push(ModelMember {
                            kind: "assoc_type",
                            name: aname,
                            signature: sig,
                            docs,
                        });
                    }
                    rd::ItemEnum::AssocConst { type_, .. } => members.push(ModelMember {
                        kind: "assoc_const",
                        name: aname.clone(),
                        signature: format!("const {aname}: {}", render_type(type_)),
                        docs,
                    }),
                    _ => {}
                }
            }
            (signature, members, Vec::new())
        }
        rd::ItemEnum::Function(f) => (render_function(&name, f, "pub "), Vec::new(), Vec::new()),
        rd::ItemEnum::TypeAlias(a) => (
            format!(
                "pub type {name}{} = {};",
                render_generics_decl(&a.generics),
                render_type(&a.type_)
            ),
            Vec::new(),
            Vec::new(),
        ),
        rd::ItemEnum::Constant { type_, const_ } => (
            format!(
                "pub const {name}: {} = {};",
                render_type(type_),
                const_.expr
            ),
            Vec::new(),
            Vec::new(),
        ),
        rd::ItemEnum::Macro(source) => (source.clone(), Vec::new(), Vec::new()),
        rd::ItemEnum::ProcMacro(_) => (format!("{name}!"), Vec::new(), Vec::new()),
        _ => return None,
    };

    Some(ModelItem {
        slug: String::new(), // assigned later, once collisions are known
        name,
        kind: kind.as_str(),
        path,
        signature,
        docs,
        members,
        trait_impls,
    })
}

/// Public named fields of a struct as members.
fn field_members(krate: &rd::Crate, fields: &[rd::Id]) -> Vec<ModelMember> {
    fields
        .iter()
        .filter_map(|id| krate.index.get(id))
        .filter(|f| matches!(f.visibility, rd::Visibility::Public))
        .filter_map(|f| match (&f.name, &f.inner) {
            (Some(name), rd::ItemEnum::StructField(t)) => Some(ModelMember {
                kind: "field",
                name: name.clone(),
                signature: format!("pub {name}: {}", render_type(t)),
                docs: f.docs.clone().map(|d| clean_docs(&d, &f.links)),
            }),
            _ => None,
        })
        .collect()
}

/// Public inherent methods and the list of (non-auto, non-blanket) trait impls.
fn impl_members(krate: &rd::Crate, impls: &[rd::Id]) -> (Vec<ModelMember>, Vec<String>) {
    let mut methods = Vec::new();
    let mut traits = Vec::new();
    for impl_id in impls {
        let Some(rd::ItemEnum::Impl(imp)) = krate.index.get(impl_id).map(|i| &i.inner) else {
            continue;
        };
        if let Some(trait_path) = &imp.trait_ {
            if !imp.is_synthetic && imp.blanket_impl.is_none() {
                traits.push(render_path(trait_path));
            }
            continue;
        }
        for item_id in &imp.items {
            let Some(item) = krate.index.get(item_id) else {
                continue;
            };
            if !matches!(item.visibility, rd::Visibility::Public) {
                continue;
            }
            if let (Some(name), rd::ItemEnum::Function(f)) = (&item.name, &item.inner) {
                methods.push(ModelMember {
                    kind: "method",
                    name: name.clone(),
                    signature: render_function(name, f, "pub "),
                    docs: item.docs.clone().map(|d| clean_docs(&d, &item.links)),
                });
            }
        }
    }
    traits.sort();
    traits.dedup();
    (methods, traits)
}

/// Assign rustdoc-style slugs (`struct.DocsConfig`), disambiguating name
/// collisions with the module path (`struct.blog.Config`).
fn assign_slugs(items: &mut [ModelItem]) {
    let mut counts: HashMap<(String, String), usize> = HashMap::new();
    for item in items.iter() {
        *counts
            .entry((item.kind.to_string(), item.name.clone()))
            .or_default() += 1;
    }
    for item in items.iter_mut() {
        let prefix = match item.kind {
            "struct" => "struct",
            "enum" => "enum",
            "trait" => "trait",
            "function" => "fn",
            "type_alias" => "type",
            "constant" => "constant",
            _ => "macro",
        };
        let collides = counts[&(item.kind.to_string(), item.name.clone())] > 1;
        item.slug = if collides && !item.path.is_empty() {
            format!("{prefix}.{}.{}", item.path.join("."), item.name)
        } else {
            format!("{prefix}.{}", item.name)
        };
    }
}

// ============================================================================
// Signature rendering
// ============================================================================

fn render_function(name: &str, f: &rd::Function, vis_prefix: &str) -> String {
    let mut out = String::from(vis_prefix);
    if f.header.is_const {
        out.push_str("const ");
    }
    if f.header.is_async {
        out.push_str("async ");
    }
    if f.header.is_unsafe {
        out.push_str("unsafe ");
    }
    out.push_str("fn ");
    out.push_str(name);
    out.push_str(&render_generics_decl(&f.generics));
    out.push('(');
    let params: Vec<String> = f
        .sig
        .inputs
        .iter()
        .map(|(pname, ptype)| {
            if pname == "self" {
                match ptype {
                    rd::Type::Generic(g) if g == "Self" => "self".to_string(),
                    rd::Type::BorrowedRef {
                        lifetime,
                        is_mutable,
                        type_,
                    } if matches!(&**type_, rd::Type::Generic(g) if g == "Self") => {
                        let lt = lifetime
                            .as_ref()
                            .map(|l| format!("{l} "))
                            .unwrap_or_default();
                        if *is_mutable {
                            format!("&{lt}mut self")
                        } else {
                            format!("&{lt}self")
                        }
                    }
                    other => format!("self: {}", render_type(other)),
                }
            } else {
                format!("{pname}: {}", render_type(ptype))
            }
        })
        .collect();
    out.push_str(&params.join(", "));
    out.push(')');
    if let Some(ret) = &f.sig.output {
        out.push_str(" -> ");
        out.push_str(&render_type(ret));
    }
    let where_clause = render_where(&f.generics.where_predicates);
    if !where_clause.is_empty() {
        out.push_str("\nwhere\n    ");
        out.push_str(&where_clause);
    }
    out
}

/// Render `<...>` declaration params, skipping compiler-synthesized ones
/// (`impl Trait` in argument position desugars to a synthetic type param).
fn render_generics_decl(generics: &rd::Generics) -> String {
    let params: Vec<String> = generics
        .params
        .iter()
        .filter_map(|p| match &p.kind {
            rd::GenericParamDefKind::Lifetime { outlives } => {
                if outlives.is_empty() {
                    Some(p.name.clone())
                } else {
                    Some(format!("{}: {}", p.name, outlives.join(" + ")))
                }
            }
            rd::GenericParamDefKind::Type {
                bounds,
                default,
                is_synthetic,
            } => {
                if *is_synthetic {
                    return None;
                }
                let mut s = p.name.clone();
                let bounds = render_bounds(bounds);
                if !bounds.is_empty() {
                    s.push_str(&format!(": {bounds}"));
                }
                if let Some(d) = default {
                    s.push_str(&format!(" = {}", render_type(d)));
                }
                Some(s)
            }
            rd::GenericParamDefKind::Const { type_, .. } => {
                Some(format!("const {}: {}", p.name, render_type(type_)))
            }
        })
        .collect();
    if params.is_empty() {
        String::new()
    } else {
        format!("<{}>", params.join(", "))
    }
}

fn render_where(predicates: &[rd::WherePredicate]) -> String {
    let parts: Vec<String> = predicates
        .iter()
        .filter_map(|p| match p {
            rd::WherePredicate::BoundPredicate { type_, bounds, .. } => {
                let bounds = render_bounds(bounds);
                if bounds.is_empty() {
                    None
                } else {
                    Some(format!("{}: {bounds}", render_type(type_)))
                }
            }
            rd::WherePredicate::LifetimePredicate { lifetime, outlives } => {
                if outlives.is_empty() {
                    None
                } else {
                    Some(format!("{lifetime}: {}", outlives.join(" + ")))
                }
            }
            rd::WherePredicate::EqPredicate { .. } => None,
        })
        .collect();
    parts.join(",\n    ")
}

fn render_bounds(bounds: &[rd::GenericBound]) -> String {
    let parts: Vec<String> = bounds
        .iter()
        .filter_map(|b| match b {
            rd::GenericBound::TraitBound {
                trait_, modifier, ..
            } => {
                let prefix = match modifier {
                    rd::TraitBoundModifier::Maybe => "?",
                    _ => "",
                };
                Some(format!("{prefix}{}", render_path(trait_)))
            }
            rd::GenericBound::Outlives(lt) => Some(lt.clone()),
            rd::GenericBound::Use(_) => None,
        })
        .collect();
    parts.join(" + ")
}

fn render_path(path: &rd::Path) -> String {
    // `path.path` is the name as written in the source (`Vec`, `fmt::Debug`).
    // Crate-internal absolute paths (`crate::registry::DocsRegistry`) are
    // trimmed to the bare name, matching how rustdoc's HTML displays them.
    let name = match path.path.strip_prefix("crate::") {
        Some(rest) => rest.rsplit("::").next().unwrap_or(rest),
        None => &path.path,
    };
    let mut out = name.to_string();
    if let Some(args) = &path.args {
        out.push_str(&render_generic_args(args));
    }
    out
}

fn render_generic_args(args: &rd::GenericArgs) -> String {
    match args {
        rd::GenericArgs::AngleBracketed { args, constraints } => {
            let mut parts: Vec<String> = args
                .iter()
                .map(|a| match a {
                    rd::GenericArg::Lifetime(lt) => lt.clone(),
                    rd::GenericArg::Type(t) => render_type(t),
                    rd::GenericArg::Const(c) => c.expr.clone(),
                    rd::GenericArg::Infer => "_".to_string(),
                })
                .collect();
            for c in constraints {
                let value = match &c.binding {
                    rd::AssocItemConstraintKind::Equality(term) => match term {
                        rd::Term::Type(t) => format!(" = {}", render_type(t)),
                        rd::Term::Constant(c) => format!(" = {}", c.expr),
                    },
                    rd::AssocItemConstraintKind::Constraint(bounds) => {
                        format!(": {}", render_bounds(bounds))
                    }
                };
                parts.push(format!("{}{value}", c.name));
            }
            if parts.is_empty() {
                String::new()
            } else {
                format!("<{}>", parts.join(", "))
            }
        }
        rd::GenericArgs::Parenthesized { inputs, output } => {
            let inputs: Vec<String> = inputs.iter().map(render_type).collect();
            let mut out = format!("({})", inputs.join(", "));
            if let Some(ret) = output {
                out.push_str(&format!(" -> {}", render_type(ret)));
            }
            out
        }
        rd::GenericArgs::ReturnTypeNotation => "(..)".to_string(),
    }
}

fn render_type(t: &rd::Type) -> String {
    match t {
        rd::Type::ResolvedPath(p) => render_path(p),
        rd::Type::DynTrait(d) => {
            let mut parts: Vec<String> =
                d.traits.iter().map(|pt| render_path(&pt.trait_)).collect();
            if let Some(lt) = &d.lifetime {
                parts.push(lt.clone());
            }
            format!("dyn {}", parts.join(" + "))
        }
        rd::Type::Generic(name) => name.clone(),
        rd::Type::Primitive(name) => name.clone(),
        rd::Type::FunctionPointer(fp) => {
            let inputs: Vec<String> = fp
                .sig
                .inputs
                .iter()
                .map(|(n, t)| {
                    if n.is_empty() || n == "_" {
                        render_type(t)
                    } else {
                        format!("{n}: {}", render_type(t))
                    }
                })
                .collect();
            let mut out = format!("fn({})", inputs.join(", "));
            if let Some(ret) = &fp.sig.output {
                out.push_str(&format!(" -> {}", render_type(ret)));
            }
            out
        }
        rd::Type::Tuple(types) => {
            let parts: Vec<String> = types.iter().map(render_type).collect();
            format!("({})", parts.join(", "))
        }
        rd::Type::Slice(inner) => format!("[{}]", render_type(inner)),
        rd::Type::Array { type_, len } => format!("[{}; {len}]", render_type(type_)),
        rd::Type::Pat { type_, .. } => render_type(type_),
        rd::Type::ImplTrait(bounds) => format!("impl {}", render_bounds(bounds)),
        rd::Type::Infer => "_".to_string(),
        rd::Type::RawPointer { is_mutable, type_ } => {
            let m = if *is_mutable { "mut" } else { "const" };
            format!("*{m} {}", render_type(type_))
        }
        rd::Type::BorrowedRef {
            lifetime,
            is_mutable,
            type_,
        } => {
            let lt = lifetime
                .as_ref()
                .map(|l| format!("{l} "))
                .unwrap_or_default();
            let m = if *is_mutable { "mut " } else { "" };
            format!("&{lt}{m}{}", render_type(type_))
        }
        rd::Type::QualifiedPath {
            name,
            args,
            self_type,
            trait_,
        } => {
            let args = args.as_deref().map(render_generic_args).unwrap_or_default();
            match trait_ {
                Some(tr) => format!(
                    "<{} as {}>::{name}{args}",
                    render_type(self_type),
                    render_path(tr)
                ),
                None => format!("{}::{name}{args}", render_type(self_type)),
            }
        }
    }
}

// ============================================================================
// Doc comment cleanup (intra-doc links + code fence normalization)
// ============================================================================

/// Prepare a doc comment for the renderer: strip intra-doc links and
/// normalize rustdoc code fences.
fn clean_docs(docs: &str, links: &HashMap<String, rd::Id>) -> String {
    normalize_doc_fences(&rewrite_doc_links(docs, links))
}

/// Rewrite rustdoc's code fence info strings to plain `rust`.
///
/// In doc comments a bare ``` fence is Rust by convention, and attribute
/// tokens like `rust,ignore` / `no_run` / `should_panic` are rustdoc test
/// directives, not languages — left as-is they defeat syntax highlighting.
/// Fences naming a real language (```bash) are kept.
fn normalize_doc_fences(docs: &str) -> String {
    let mut out = String::with_capacity(docs.len());
    let mut in_fence = false;
    for line in docs.lines() {
        let trimmed = line.trim_start();
        if let Some(info) = trimmed.strip_prefix("```") {
            if in_fence {
                in_fence = false;
                out.push_str(line);
            } else {
                in_fence = true;
                if info.is_empty() || is_rust_fence_info(info) {
                    let indent = &line[..line.len() - trimmed.len()];
                    out.push_str(indent);
                    out.push_str("```rust");
                } else {
                    out.push_str(line);
                }
            }
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    // `str::lines` drops a trailing newline; don't add one that wasn't there.
    if !docs.ends_with('\n') {
        out.pop();
    }
    out
}

/// Whether a fence info string consists only of rustdoc test directives.
fn is_rust_fence_info(info: &str) -> bool {
    info.split([',', ' '])
        .filter(|t| !t.is_empty())
        .all(|token| {
            matches!(
                token,
                "rust" | "ignore" | "no_run" | "should_panic" | "compile_fail" | "standalone_crate"
            ) || token.starts_with("edition")
        })
}

/// Strip rustdoc intra-doc links, keeping the display text.
///
/// The renderer has no way to resolve `[`DocsConfig`]` or
/// `[`auto_meta`](Self::auto_meta)` to a page, so both forms are reduced to
/// their text (which usually carries its own backticks). Regular markdown
/// links with URL targets are left alone — only targets recorded in the
/// item's rustdoc `links` table are touched.
fn rewrite_doc_links(docs: &str, links: &HashMap<String, rd::Id>) -> String {
    if links.is_empty() {
        return docs.to_string();
    }
    let mut out = docs.to_string();
    const MARK: char = '\u{1}';
    for target in links.keys() {
        // Inline form: `[text](target)` — mark the closing bracket, drop the target.
        out = out.replace(&format!("]({target})"), &format!("]{MARK}"));
        // Shortcut form: `[target]` — the target is its own display text.
        out = out.replace(&format!("[{target}][]"), target);
        out = replace_shortcut(&out, target);
    }
    // Unwrap every `[text]<MARK>` produced by the inline pass.
    while let Some(mark_pos) = out.find(MARK) {
        let before = &out[..mark_pos];
        if let (Some(open), true) = (before.rfind('['), before.ends_with(']')) {
            let text = before[open + 1..before.len() - 1].to_string();
            out.replace_range(open..mark_pos + MARK.len_utf8(), &text);
        } else {
            out.replace_range(mark_pos..mark_pos + MARK.len_utf8(), "");
        }
    }
    out
}

/// Replace `[target]` with `target` when not part of an inline link.
fn replace_shortcut(text: &str, target: &str) -> String {
    let needle = format!("[{target}]");
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(pos) = rest.find(&needle) {
        let after = &rest[pos + needle.len()..];
        out.push_str(&rest[..pos]);
        if after.starts_with('(') || after.starts_with('[') || after.starts_with('\u{1}') {
            // Part of an inline/reference link — leave for other passes.
            out.push_str(&needle);
        } else {
            out.push_str(target);
        }
        rest = after;
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn links(targets: &[&str]) -> HashMap<String, rd::Id> {
        targets
            .iter()
            .enumerate()
            .map(|(i, t)| (t.to_string(), rd::Id(i as u32)))
            .collect()
    }

    #[test]
    fn rewrites_shortcut_links_to_plain_text() {
        let out = rewrite_doc_links("See [`DocsConfig`] for details.", &links(&["`DocsConfig`"]));
        assert_eq!(out, "See `DocsConfig` for details.");
    }

    #[test]
    fn rewrites_inline_links_keeping_display_text() {
        let out = rewrite_doc_links(
            "Override with [`auto_meta`](Self::auto_meta).",
            &links(&["Self::auto_meta"]),
        );
        assert_eq!(out, "Override with `auto_meta`.");
    }

    #[test]
    fn leaves_regular_markdown_links_alone() {
        let docs = "See [the site](https://example.com) and [`Kept`].";
        let out = rewrite_doc_links(docs, &links(&["`Kept`"]));
        assert_eq!(out, "See [the site](https://example.com) and `Kept`.");
    }

    #[test]
    fn normalizes_rustdoc_fences_to_rust() {
        let docs =
            "Text\n\n```rust,ignore\nlet x = 1;\n```\n\n```\nbare();\n```\n\n```bash\nls\n```\n";
        let out = normalize_doc_fences(docs);
        assert!(out.contains("```rust\nlet x = 1;"));
        assert!(out.contains("```rust\nbare();"));
        assert!(out.contains("```bash\nls"));
        // Closing fences stay untouched (no ```rust``` at block ends).
        assert_eq!(out.matches("```rust").count(), 2);
    }

    #[test]
    fn fence_info_only_matches_rustdoc_directives() {
        assert!(is_rust_fence_info("rust,ignore"));
        assert!(is_rust_fence_info("no_run"));
        assert!(is_rust_fence_info("edition2021"));
        assert!(!is_rust_fence_info("bash"));
        assert!(!is_rust_fence_info("json"));
    }

    #[test]
    fn renders_reference_types() {
        let t = rd::Type::BorrowedRef {
            lifetime: Some("'a".to_string()),
            is_mutable: false,
            type_: Box::new(rd::Type::Primitive("str".to_string())),
        };
        assert_eq!(render_type(&t), "&'a str");
    }

    #[test]
    fn renders_resolved_path_with_args() {
        let t = rd::Type::ResolvedPath(rd::Path {
            path: "Vec".to_string(),
            id: rd::Id(1),
            args: Some(Box::new(rd::GenericArgs::AngleBracketed {
                args: vec![rd::GenericArg::Type(rd::Type::Primitive(
                    "String".to_string(),
                ))],
                constraints: vec![],
            })),
        });
        assert_eq!(render_type(&t), "Vec<String>");
    }

    #[test]
    fn renders_impl_trait_and_tuples() {
        let t = rd::Type::Tuple(vec![
            rd::Type::ImplTrait(vec![rd::GenericBound::TraitBound {
                trait_: rd::Path {
                    path: "Into".to_string(),
                    id: rd::Id(2),
                    args: Some(Box::new(rd::GenericArgs::AngleBracketed {
                        args: vec![rd::GenericArg::Type(rd::Type::Primitive(
                            "String".to_string(),
                        ))],
                        constraints: vec![],
                    })),
                },
                generic_params: vec![],
                modifier: rd::TraitBoundModifier::None,
            }]),
            rd::Type::Primitive("u32".to_string()),
        ]);
        assert_eq!(render_type(&t), "(impl Into<String>, u32)");
    }
}
