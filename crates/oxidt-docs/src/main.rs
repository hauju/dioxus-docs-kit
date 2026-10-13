//! `oxidt-docs` — MDX + `_nav.json` in, a static docs site out.
//!
//! ```sh
//! oxidt-docs build [--root .] [--out dist] [--base-path /repo] [--openapi PREFIX=SPEC]... [--site-url URL]
//! oxidt-docs dev   [--root .] [--port 3000] [--openapi PREFIX=SPEC]...
//! ```

mod site;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::Ordering;
use std::time::{Duration, SystemTime};

use http_body_util::BodyExt;
use topcoat::router::{Router, StatusCode};

use site::Options;

const USAGE: &str = "usage:
  oxidt-docs build [--root DIR] [--out DIR] [--base-path /REPO] [--openapi PREFIX=SPEC]... [--site-url URL]
  oxidt-docs dev   [--root DIR] [--port PORT] [--openapi PREFIX=SPEC]...";

struct Args {
    command: String,
    opts: Options,
    out: PathBuf,
    base_path: String,
    port: u16,
}

fn parse_args() -> Result<Args, String> {
    let mut args = std::env::args().skip(1);
    let command = args.next().ok_or(USAGE)?;
    let mut parsed = Args {
        command,
        opts: Options {
            root: PathBuf::from("."),
            openapi: Vec::new(),
            site_url: None,
        },
        out: PathBuf::from("dist"),
        base_path: String::new(),
        port: 3000,
    };
    while let Some(flag) = args.next() {
        let mut value = || args.next().ok_or(format!("{flag} needs a value"));
        match flag.as_str() {
            "--root" => parsed.opts.root = value()?.into(),
            "--out" => parsed.out = value()?.into(),
            "--base-path" => {
                parsed.base_path = format!("/{}", value()?.trim_matches('/'));
                if parsed.base_path == "/" {
                    parsed.base_path.clear();
                }
            }
            "--port" => parsed.port = value()?.parse().map_err(|e| format!("--port: {e}"))?,
            "--site-url" => {
                parsed.opts.site_url = Some(value()?.trim_end_matches('/').to_string());
            }
            "--openapi" => {
                let value = value()?;
                let (prefix, spec) = value
                    .split_once('=')
                    .ok_or("--openapi expects PREFIX=SPEC, e.g. api-reference=docs/api.yaml")?;
                parsed
                    .opts
                    .openapi
                    .push((prefix.to_string(), spec.to_string()));
            }
            _ => return Err(format!("unknown flag {flag}\n{USAGE}")),
        }
    }
    Ok(parsed)
}

#[tokio::main]
async fn main() -> ExitCode {
    let result = match parse_args() {
        Ok(args) => match args.command.as_str() {
            "build" => build(&args.opts, &args.out, &args.base_path).await,
            "dev" => dev(&args.opts, args.port).await,
            other => Err(format!("unknown command {other}\n{USAGE}")),
        },
        Err(e) => Err(e),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Written into every export; `build` only replaces an output directory that
/// has it (or is empty), so `--out` can never wipe unrelated files.
const MARKER: &str = ".oxidt-docs";

/// Export into a staging directory next to `out`, then swap it in, so pages
/// deleted from the nav do not survive in a reused output directory.
async fn build(opts: &Options, out: &Path, base: &str) -> Result<(), String> {
    if out.exists() && !out.join(MARKER).exists() && !is_empty_dir(out) {
        return Err(format!(
            "{} exists and is not an oxidt-docs output directory; refusing to replace it",
            out.display()
        ));
    }
    let name = out.file_name().ok_or("--out needs a directory name")?;
    let staging = out.with_file_name(format!("{}.oxidt-tmp", name.to_string_lossy()));
    remove_dir(&staging)?;

    let pages = match export(opts, &staging, base).await {
        Ok(pages) => pages,
        Err(e) => {
            let _ = fs::remove_dir_all(&staging);
            return Err(e);
        }
    };
    remove_dir(out)?;
    fs::rename(&staging, out).map_err(|e| format!("{}: {e}", out.display()))?;
    println!("wrote {pages} pages to {}", out.display());
    Ok(())
}

fn is_empty_dir(dir: &Path) -> bool {
    fs::read_dir(dir).is_ok_and(|mut entries| entries.next().is_none())
}

fn remove_dir(dir: &Path) -> Result<(), String> {
    match fs::remove_dir_all(dir) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            Err(format!("{}: {e}", dir.display()))
        }
        _ => Ok(()),
    }
}

/// Render every route through the router in memory and write it to `out`.
///
/// Pages are rendered at `/docs/...`; their URLs are then made root-relative
/// and prefixed with `base` (e.g. `/my-repo` for a GitHub project page).
/// Returns the number of pages written.
async fn export(opts: &Options, out: &Path, base: &str) -> Result<usize, String> {
    let bundle = site::load(opts)?;
    let router = site::router(&opts.root);
    let registry = site::registry();

    // Static files referenced from content (`/images/x.png`) come from
    // `<root>/public`; generated files below win on a name clash.
    copy_dir(&opts.root.join("public"), out)?;
    write(&out.join(MARKER), "")?;

    let mut paths: Vec<String> = registry
        .get_all_paths()
        .into_iter()
        .map(String::from)
        .collect();
    paths.extend(registry.get_api_endpoint_paths());
    paths.sort();

    let docs = out.join("docs");
    // The index renders the default page, so its links resolve like that page's.
    let html = render(&router, "/docs").await?;
    write(
        &docs.join("index.html"),
        &rewrite_urls(&html, base, &url_dir(&registry.default_path)),
    )?;
    for path in &paths {
        let html = render(&router, &format!("/docs/{path}")).await?;
        write(
            &docs.join(path).join("index.html"),
            &rewrite_urls(&html, base, &url_dir(path)),
        )?;
    }

    // The header search box submits to /docs/search; statically that page
    // ranks the exported index in the browser.
    let search_page = rewrite_urls(&render(&router, "/docs/search").await?, base, "/docs").replace(
        "</body>",
        &format!("<script src=\"{base}/oxidt-search.js\" data-base=\"{base}\"></script></body>"),
    );
    write(&docs.join("search/index.html"), &search_page)?;
    write(&out.join("search-index.json"), &search_index(bundle)?)?;
    write(&out.join("oxidt-search.js"), include_str!("search.js"))?;

    write(&out.join("docs-kit.css"), topcoat_docs_kit::STYLESHEET)?;
    write(
        &out.join("index.html"),
        &format!(
            "<!DOCTYPE html><meta charset=\"utf-8\"><meta http-equiv=\"refresh\" content=\"0; url={base}/docs\"><link rel=\"canonical\" href=\"{base}/docs\">"
        ),
    )?;
    if let Some(url) = &opts.site_url {
        write(
            &out.join("sitemap.xml"),
            &registry.generate_sitemap(url, &format!("{base}/docs")),
        )?;
        write(
            &out.join("robots.txt"),
            &format!("User-agent: *\nAllow: /\n\nSitemap: {url}{base}/sitemap.xml\n"),
        )?;
    }

    Ok(paths.len() + 1)
}

fn copy_dir(from: &Path, to: &Path) -> Result<(), String> {
    let Ok(entries) = fs::read_dir(from) else {
        return Ok(());
    };
    fs::create_dir_all(to).map_err(|e| format!("{}: {e}", to.display()))?;
    for entry in entries.flatten() {
        let (src, dest) = (entry.path(), to.join(entry.file_name()));
        if src.is_dir() {
            copy_dir(&src, &dest)?;
        } else {
            fs::copy(&src, &dest).map_err(|e| format!("{}: {e}", src.display()))?;
        }
    }
    Ok(())
}

/// The URL directory a docs page is served from: the server serves
/// `guides/intro` at `/docs/guides/intro`, so its relative links resolve
/// against `/docs/guides`.
fn url_dir(path: &str) -> String {
    match path.rsplit_once('/') {
        Some((dir, _)) => format!("/docs/{dir}"),
        None => "/docs".to_string(),
    }
}

/// Rewrite every `href`/`src`/`action` URL for the static export.
///
/// Exported pages live at `<page>/index.html`, which hosts serve as
/// `<page>/`, so relative URLs would resolve one level too deep: they are
/// resolved against `page_dir` instead. Root-relative URLs then get `base`.
/// Fragments, queries, `//host` and `scheme:` URLs are left alone.
fn rewrite_urls(html: &str, base: &str, page_dir: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(i) = rest.find("=\"") {
        let (head, tail) = rest.split_at(i + 2);
        out.push_str(head);
        let Some(end) = tail.find('"') else {
            rest = tail;
            break;
        };
        let (value, after) = tail.split_at(end);
        let attr = head[..i].rsplit(|c: char| c.is_whitespace()).next();
        if matches!(attr, Some("href" | "src" | "action")) {
            out.push_str(&resolve_url(value, base, page_dir));
        } else {
            out.push_str(value);
        }
        rest = after;
    }
    out.push_str(rest);
    out
}

fn resolve_url(url: &str, base: &str, page_dir: &str) -> String {
    let has_scheme = url.split_once(':').is_some_and(|(scheme, _)| {
        !scheme.is_empty()
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c))
    });
    if url.is_empty() || url.starts_with(['#', '?']) || url.starts_with("//") || has_scheme {
        return url.to_string();
    }
    if url.starts_with('/') {
        return format!("{base}{url}");
    }
    // Normalize `page_dir/url`, keeping any `?query`/`#fragment` as is.
    let split = url.find(['?', '#']).unwrap_or(url.len());
    let (path, suffix) = url.split_at(split);
    let mut segments: Vec<&str> = page_dir.split('/').filter(|s| !s.is_empty()).collect();
    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop();
            }
            s => segments.push(s),
        }
    }
    let trailing = if path.ends_with('/') && !segments.is_empty() {
        "/"
    } else {
        ""
    };
    format!("{base}/{}{trailing}{suffix}", segments.join("/"))
}

/// The bundle's search entries, trimmed to what `search.js` reads.
fn search_index(bundle: &str) -> Result<String, String> {
    #[derive(serde::Deserialize)]
    struct Bundle {
        search: Vec<Entry>,
    }
    // Mirrors the bundle's `SearchEntry`, whose empty fields may be omitted.
    #[derive(Default, serde::Deserialize, serde::Serialize)]
    #[serde(default)]
    struct Entry {
        path: String,
        anchor: String,
        title: String,
        heading: String,
        description: String,
        body: String,
        breadcrumb: String,
        #[serde(skip_serializing)]
        api_method: Option<dioxus_mdx::HttpMethod>,
        #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
        method: Option<&'static str>,
        #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
        method_class: Option<&'static str>,
    }
    let mut bundle: Bundle = serde_json::from_str(bundle).map_err(|e| e.to_string())?;
    for entry in &mut bundle.search {
        entry.method = entry.api_method.map(|m| m.as_str());
        entry.method_class = entry.api_method.map(|m| m.badge_class());
    }
    serde_json::to_string(&bundle.search).map_err(|e| e.to_string())
}

async fn render(router: &Router, uri: &str) -> Result<String, String> {
    let request = http::Request::builder()
        .uri(uri)
        .body(Default::default())
        .map_err(|e| format!("{uri}: {e}"))?;
    let response = router.handle(request).await;
    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .map_err(|e| format!("{uri}: {e}"))?
        .to_bytes();
    if status != StatusCode::OK {
        return Err(format!("{uri}: rendered with status {status}"));
    }
    String::from_utf8(body.to_vec()).map_err(|e| format!("{uri}: {e}"))
}

fn write(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    fs::write(path, contents).map_err(|e| format!("{}: {e}", path.display()))
}

/// Serve the router live, rebuilding when anything under `<root>/docs` or a
/// spec changes; open pages reload themselves.
async fn dev(opts: &Options, port: u16) -> Result<(), String> {
    site::load(opts)?;
    site::LIVE_RELOAD.store(true, Ordering::Relaxed);

    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .map_err(|e| format!("port {port}: {e}"))?;
    println!("serving http://127.0.0.1:{port}/docs");

    let watched = watched_paths(opts);
    let mut stamp = latest_change(&watched);
    let router = site::router(&opts.root);
    let opts = opts.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;
            let now = latest_change(&watched);
            if now == stamp {
                continue;
            }
            stamp = now;
            match site::load(&opts) {
                Ok(_) => println!("rebuilt"),
                Err(e) => eprintln!("error: {e}"),
            }
        }
    });

    topcoat::serve(listener, router)
        .await
        .map_err(|e| e.to_string())
}

fn watched_paths(opts: &Options) -> Vec<PathBuf> {
    let mut paths = vec![opts.root.join("docs")];
    paths.extend(opts.openapi.iter().map(|(_, spec)| opts.root.join(spec)));
    paths
}

/// Newest mtime and file count under `paths`, so edits, additions and
/// deletions all change the stamp.
fn latest_change(paths: &[PathBuf]) -> (Option<SystemTime>, usize) {
    fn walk(path: &Path, stamp: &mut (Option<SystemTime>, usize)) {
        let Ok(meta) = fs::metadata(path) else { return };
        if meta.is_dir() {
            for entry in fs::read_dir(path).into_iter().flatten().flatten() {
                walk(&entry.path(), stamp);
            }
        } else {
            stamp.0 = stamp.0.max(meta.modified().ok());
            stamp.1 += 1;
        }
    }
    let mut stamp = (None, 0);
    for path in paths {
        walk(path, &mut stamp);
    }
    stamp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_root_relative_and_relative_urls() {
        let html = r#"<a href="/docs/x">a</a><a href="next#h">b</a><img src="../img/a.png"><a href="./">c</a>"#;
        assert_eq!(
            rewrite_urls(html, "/repo", "/docs/guides"),
            r#"<a href="/repo/docs/x">a</a><a href="/repo/docs/guides/next#h">b</a><img src="/repo/docs/img/a.png"><a href="/repo/docs/guides/">c</a>"#
        );
    }

    #[test]
    fn leaves_external_fragment_and_other_attributes_alone() {
        let html = r##"<a href="https://x.dev/a">a</a><a href="#top">b</a><script src="//cdn.x/y.js"></script><a href="mailto:a@b.c">c</a><div data-href="next" title="a b"></div>"##;
        assert_eq!(rewrite_urls(html, "/repo", "/docs/guides"), html);
    }

    #[test]
    fn relative_urls_resolve_without_a_base() {
        assert_eq!(resolve_url("next", "", "/docs/guides"), "/docs/guides/next");
        assert_eq!(resolve_url("/a.png", "", "/docs"), "/a.png");
    }
}
