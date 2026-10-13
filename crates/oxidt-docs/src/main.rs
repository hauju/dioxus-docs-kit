//! `oxidt-docs` — MDX + `_nav.json` in, a static docs site out.
//!
//! ```sh
//! oxidt-docs build [--root .] [--out dist] [--openapi PREFIX=SPEC]... [--site-url URL]
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
  oxidt-docs build [--root DIR] [--out DIR] [--openapi PREFIX=SPEC]... [--site-url URL]
  oxidt-docs dev   [--root DIR] [--port PORT] [--openapi PREFIX=SPEC]...";

struct Args {
    command: String,
    opts: Options,
    out: PathBuf,
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
        port: 3000,
    };
    while let Some(flag) = args.next() {
        let mut value = || args.next().ok_or(format!("{flag} needs a value"));
        match flag.as_str() {
            "--root" => parsed.opts.root = value()?.into(),
            "--out" => parsed.out = value()?.into(),
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
            "build" => build(&args.opts, &args.out).await,
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

/// Render every route through the router in memory and write it to `out`.
async fn build(opts: &Options, out: &Path) -> Result<(), String> {
    site::load(opts)?;
    let router = site::router();
    let registry = site::registry();

    let mut paths: Vec<String> = registry
        .get_all_paths()
        .into_iter()
        .map(String::from)
        .collect();
    paths.extend(registry.get_api_endpoint_paths());
    paths.sort();

    write(
        &out.join("docs/index.html"),
        &render(&router, "/docs").await?,
    )?;
    for path in &paths {
        let html = render(&router, &format!("/docs/{path}")).await?;
        write(&out.join("docs").join(path).join("index.html"), &html)?;
    }
    write(&out.join("docs-kit.css"), topcoat_docs_kit::STYLESHEET)?;
    write(
        &out.join("index.html"),
        "<!DOCTYPE html><meta charset=\"utf-8\"><meta http-equiv=\"refresh\" content=\"0; url=/docs\"><link rel=\"canonical\" href=\"/docs\">",
    )?;
    if let Some(url) = &opts.site_url {
        write(
            &out.join("sitemap.xml"),
            &registry.generate_sitemap(url, "/docs"),
        )?;
        write(
            &out.join("robots.txt"),
            &format!("User-agent: *\nAllow: /\n\nSitemap: {url}/sitemap.xml\n"),
        )?;
    }

    println!("wrote {} pages to {}", paths.len() + 1, out.display());
    Ok(())
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
                Ok(()) => println!("rebuilt"),
                Err(e) => eprintln!("error: {e}"),
            }
        }
    });

    topcoat::serve(listener, site::router())
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
