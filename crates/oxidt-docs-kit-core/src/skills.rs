//! Agent skills (`skills/<name>/SKILL.md`) rendered as site pages.
//!
//! `dioxus-docs-kit-build`'s `SkillsBuild` parses the skills listed in
//! `_skills.json` into a bundle; [`SkillsRegistry`] answers lookups, the
//! install command and the GitHub links for each one.

use crate::bundle::SkillsBundle;
use crate::error::DocsKitError;
use crate::registry::DocsHit;
use crate::search::Field;
use crate::seo::xml_escape;
use dioxus_mdx::DocNode;
use serde::Deserialize;

/// One agent skill, parsed from its `SKILL.md` at build time.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Skill {
    /// The skill's directory name, used as its URL slug.
    pub slug: String,
    /// Frontmatter `name` (the slug when absent).
    pub name: String,
    /// Frontmatter `description`.
    #[serde(default)]
    pub description: String,
    /// The `_skills.json` group the skill is listed under.
    pub group: String,
    /// Rendered body, leading `# Title` removed.
    pub body: Vec<DocNode>,
    /// Body Markdown, for `.md` routes and `llms.txt`.
    pub raw_markdown: String,
}

/// One searchable skill section (split on h2–h4 headings), built by
/// `dioxus-docs-kit-build` with its `*_lower` fields precomputed.
#[derive(PartialEq, Deserialize)]
pub struct SkillsSearchEntry {
    pub slug: String,
    /// Heading anchor id, empty for the intro section.
    #[serde(default)]
    pub anchor: String,
    pub name: String,
    /// Section heading, empty for the intro section.
    #[serde(default)]
    pub heading: String,
    #[serde(default)]
    pub description: String,
    /// Cleaned section body used for matching and snippets.
    #[serde(default)]
    pub body: String,
    pub group: String,
    pub(crate) name_lower: String,
    #[serde(default)]
    pub(crate) heading_lower: String,
    #[serde(default)]
    pub(crate) description_lower: String,
    #[serde(default)]
    pub(crate) body_lower: String,
}

/// Builder for a [`SkillsRegistry`].
///
/// ```rust,ignore
/// let registry = SkillsConfig::new(dioxus_docs_kit::skills_bundle!())
///     .with_base_path("/skills")
///     .build();
/// ```
pub struct SkillsConfig {
    bundle_json: &'static str,
    base_path: String,
}

impl SkillsConfig {
    /// Create a builder from the bundle embedded with
    /// [`skills_bundle!()`](crate::skills_bundle).
    pub fn new(bundle_json: &'static str) -> Self {
        Self {
            bundle_json,
            base_path: "/skills".to_string(),
        }
    }

    /// Root-relative path the skill pages are routed under (default `/skills`).
    pub fn with_base_path(mut self, path: &str) -> Self {
        self.base_path = path.trim_end_matches('/').to_string();
        self
    }

    /// Build the registry.
    ///
    /// # Panics
    ///
    /// Panics if the bundle fails to deserialize; see [`Self::try_build`].
    pub fn build(self) -> SkillsRegistry {
        self.try_build()
            .unwrap_or_else(|e| panic!("dioxus-docs-kit: {e}"))
    }

    /// Build the registry, returning an error for an unreadable bundle.
    pub fn try_build(self) -> Result<SkillsRegistry, DocsKitError> {
        let bundle = SkillsBundle::parse(self.bundle_json)?;
        Ok(SkillsRegistry {
            skills: bundle.skills,
            search_index: bundle.search,
            repo: bundle.repo,
            path: bundle.path.trim_matches('/').to_string(),
            git_ref: bundle.git_ref,
            base_path: self.base_path,
        })
    }
}

/// All skills listed in `_skills.json`, in config order.
pub struct SkillsRegistry {
    skills: Vec<Skill>,
    search_index: Vec<SkillsSearchEntry>,
    repo: String,
    path: String,
    git_ref: String,
    base_path: String,
}

impl SkillsRegistry {
    pub fn get_skill(&self, slug: &str) -> Option<&Skill> {
        self.skills.iter().find(|s| s.slug == slug)
    }

    pub fn all_skills(&self) -> &[Skill] {
        &self.skills
    }

    /// `(group, skills)` pairs in `_skills.json` order.
    pub fn groups(&self) -> Vec<(&str, Vec<&Skill>)> {
        let mut groups: Vec<(&str, Vec<&Skill>)> = Vec::new();
        for skill in &self.skills {
            match groups.iter_mut().find(|(name, _)| *name == skill.group) {
                Some((_, skills)) => skills.push(skill),
                None => groups.push((&skill.group, vec![skill])),
            }
        }
        groups
    }

    /// Search skill sections: multi-term AND, ranked name > heading >
    /// description > body, like docs search.
    pub fn search_skills(&self, query: &str) -> Vec<&SkillsSearchEntry> {
        crate::search::rank(&self.search_index, query, |e, buf| {
            buf.push(Field::title(&e.name_lower));
            if !e.heading_lower.is_empty() {
                buf.push(Field::heading(&e.heading_lower));
            }
            if !e.description_lower.is_empty() {
                buf.push(Field::description(&e.description_lower));
            }
            if !e.body_lower.is_empty() {
                buf.push(Field::body(&e.body_lower));
            }
        })
    }

    /// [`search_skills`](Self::search_skills) shaped for display, as
    /// [`DocsHit`]s: `target` is `slug` or `slug#anchor`, `breadcrumb` the group.
    pub fn search_hits(&self, query: &str, limit: usize) -> Vec<DocsHit> {
        let terms = crate::search::split_terms(query);
        self.search_skills(query)
            .into_iter()
            .take(limit)
            .map(|entry| {
                let target = if entry.anchor.is_empty() {
                    entry.slug.clone()
                } else {
                    format!("{}#{}", entry.slug, entry.anchor)
                };
                let (title, context) = if entry.heading.is_empty() {
                    (entry.name.clone(), None)
                } else {
                    (entry.heading.clone(), Some(entry.name.clone()))
                };
                let snippet_src = if entry.body.is_empty() {
                    &entry.description
                } else {
                    &entry.body
                };
                DocsHit {
                    target,
                    title,
                    context,
                    breadcrumb: entry.group.clone(),
                    api_method: None,
                    snippet: crate::search::build_snippet(
                        snippet_src,
                        &terms,
                        crate::search::SNIPPET_WINDOW,
                    ),
                }
            })
            .collect()
    }

    /// GitHub `owner/name` the links and install command point at.
    pub fn repo(&self) -> &str {
        &self.repo
    }

    /// Directory of the skills inside [`repo`](Self::repo).
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Git ref the links point at (default `main`).
    pub fn git_ref(&self) -> &str {
        &self.git_ref
    }

    /// Root-relative path the skill pages are routed under.
    pub fn base_path(&self) -> &str {
        &self.base_path
    }

    /// `npx skills add <repo> --skill <name>`.
    pub fn install_command(&self, skill: &Skill) -> String {
        format!("npx skills add {} --skill {}", self.repo, skill.slug)
    }

    /// The `SKILL.md` on GitHub.
    pub fn github_url(&self, skill: &Skill) -> String {
        format!(
            "https://github.com/{}/blob/{}/{}",
            self.repo,
            self.git_ref,
            self.file_path(skill)
        )
    }

    /// The raw `SKILL.md`.
    pub fn raw_url(&self, skill: &Skill) -> String {
        format!(
            "https://raw.githubusercontent.com/{}/{}/{}",
            self.repo,
            self.git_ref,
            self.file_path(skill)
        )
    }

    fn file_path(&self, skill: &Skill) -> String {
        if self.path.is_empty() {
            format!("{}/SKILL.md", skill.slug)
        } else {
            format!("{}/{}/SKILL.md", self.path, skill.slug)
        }
    }

    /// An `## Skills` section for `llms.txt`; `skills_base_url` is the absolute
    /// URL of the skills index, e.g. `https://example.com/skills`.
    pub fn generate_llms_section(&self, skills_base_url: &str) -> String {
        let mut out = String::from("## Skills\n\n");
        for skill in &self.skills {
            let url = format!("{skills_base_url}/{}", skill.slug);
            if skill.description.is_empty() {
                out.push_str(&format!("- [{}]({url})\n", skill.name));
            } else {
                out.push_str(&format!(
                    "- [{}]({url}): {}\n",
                    skill.name, skill.description
                ));
            }
        }
        out
    }

    /// A sitemap of the skills index and every skill page.
    pub fn generate_sitemap(&self, site_url: &str, skills_path: &str) -> String {
        let mut xml = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
        );
        let index_loc = xml_escape(&format!("{site_url}{skills_path}"));
        xml.push_str(&format!(
            "<url>\n<loc>{index_loc}</loc>\n<changefreq>weekly</changefreq>\n<priority>0.8</priority>\n</url>\n"
        ));
        for skill in &self.skills {
            let loc = xml_escape(&format!("{site_url}{skills_path}/{}", skill.slug));
            xml.push_str(&format!(
                "<url>\n<loc>{loc}</loc>\n<changefreq>monthly</changefreq>\n<priority>0.6</priority>\n</url>\n"
            ));
        }
        xml.push_str("</urlset>\n");
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &str = r#"{
        "repo": "acme/tools",
        "path": "skills/",
        "groups": [
            { "group": "Setup", "skills": ["install-cli", "configure"] },
            { "group": "Theming", "skills": ["theme"] }
        ]
    }"#;

    fn registry(manifest: &str) -> SkillsRegistry {
        let skills: &[(&str, &str)] = &[
            (
                "theme",
                "---\nname: theme\ndescription: Make it pretty.\n---\n# Theme\n\nColors.\n\n## Dark mode\n\nInvert the palette.\n",
            ),
            (
                "install-cli",
                "---\nname: install-cli\ndescription: >-\n  Install the\n  CLI.\nmetadata:\n  version: \"1\"\n---\n# Install\n\nRun `cargo install`.\n",
            ),
            ("configure", "---\nname: configure\n---\nNo heading.\n"),
        ];
        let bundle = dioxus_docs_kit_build::skills_bundle_json(manifest, skills)
            .expect("generate skills bundle")
            .leak();
        SkillsConfig::new(bundle).with_base_path("/skills/").build()
    }

    #[test]
    fn groups_keep_config_order() {
        let registry = registry(MANIFEST);
        let groups: Vec<(&str, Vec<&str>)> = registry
            .groups()
            .into_iter()
            .map(|(g, s)| (g, s.iter().map(|s| s.slug.as_str()).collect()))
            .collect();
        assert_eq!(
            groups,
            vec![
                ("Setup", vec!["install-cli", "configure"]),
                ("Theming", vec!["theme"])
            ]
        );
        assert_eq!(registry.base_path(), "/skills");
    }

    #[test]
    fn get_skill_reads_frontmatter_and_strips_the_title() {
        let registry = registry(MANIFEST);
        let skill = registry.get_skill("install-cli").unwrap();
        assert_eq!(skill.name, "install-cli");
        assert_eq!(skill.description, "Install the CLI.");
        assert_eq!(skill.group, "Setup");
        assert!(!skill.raw_markdown.contains("# Install"));
        assert!(skill.raw_markdown.contains("cargo install"));
        assert_eq!(registry.get_skill("configure").unwrap().description, "");
        assert!(registry.get_skill("missing").is_none());
    }

    #[test]
    fn install_command_and_urls_point_at_the_configured_repo() {
        let registry = registry(MANIFEST);
        let skill = registry.get_skill("theme").unwrap();
        assert_eq!(registry.git_ref(), "main");
        assert_eq!(
            registry.install_command(skill),
            "npx skills add acme/tools --skill theme"
        );
        assert_eq!(
            registry.github_url(skill),
            "https://github.com/acme/tools/blob/main/skills/theme/SKILL.md"
        );
        assert_eq!(
            registry.raw_url(skill),
            "https://raw.githubusercontent.com/acme/tools/main/skills/theme/SKILL.md"
        );

        let pinned = self::registry(
            r#"{"repo": "acme/cli", "path": "agent/skills", "ref": "v2",
                "groups": [{ "group": "All", "skills": ["theme"] }]}"#,
        );
        assert_eq!(
            pinned.raw_url(pinned.get_skill("theme").unwrap()),
            "https://raw.githubusercontent.com/acme/cli/v2/agent/skills/theme/SKILL.md"
        );
    }

    #[test]
    fn search_finds_skills_by_name_and_section() {
        let registry = registry(MANIFEST);
        let hits = registry.search_hits("theme", 10);
        assert_eq!(hits[0].target, "theme");
        assert_eq!(hits[0].title, "theme");
        assert_eq!(hits[0].breadcrumb, "Theming");

        // Section-level: a body match deep-links to its heading.
        let hits = registry.search_hits("invert palette", 10);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].target, "theme#dark-mode");
        assert_eq!(hits[0].title, "Dark mode");
        assert_eq!(hits[0].context.as_deref(), Some("theme"));

        assert!(registry.search_skills("   ").is_empty());
        assert!(registry.search_skills("nonexistent-term").is_empty());
    }

    #[test]
    fn sitemap_and_llms_list_every_skill() {
        let registry = registry(MANIFEST);
        let xml = registry.generate_sitemap("https://example.com", "/skills");
        for path in ["/skills", "/skills/install-cli", "/skills/theme"] {
            assert!(xml.contains(&format!("<loc>https://example.com{path}</loc>")));
        }
        let llms = registry.generate_llms_section("https://example.com/skills");
        assert!(llms.starts_with("## Skills\n\n"));
        assert!(llms.contains("- [theme](https://example.com/skills/theme): Make it pretty.\n"));
        assert!(llms.contains("- [configure](https://example.com/skills/configure)\n"));
    }
}
