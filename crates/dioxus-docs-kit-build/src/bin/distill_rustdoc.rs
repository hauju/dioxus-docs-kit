//! CLI wrapper around [`dioxus_docs_kit_build::distill_rustdoc`].
//!
//! ```sh
//! cargo +nightly rustdoc -p my-crate --lib -- -Z unstable-options --output-format json
//! distill-rustdoc target/doc/my_crate.json docs/rust-api/my-crate.api.json --exclude Props
//! ```

use dioxus_docs_kit_build::{DistillOptions, distill_rustdoc};
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let (Some(input), Some(output)) = (args.next(), args.next()) else {
        eprintln!(
            "usage: distill-rustdoc <rustdoc.json> <model.json> [--exclude NAME_SUBSTRING]..."
        );
        return ExitCode::FAILURE;
    };

    let mut options = DistillOptions::default();
    let rest: Vec<String> = args.collect();
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--exclude" if i + 1 < rest.len() => {
                options.exclude_name_parts.push(rest[i + 1].clone());
                i += 2;
            }
            other => {
                eprintln!("unknown argument: {other}");
                return ExitCode::FAILURE;
            }
        }
    }

    let json = match std::fs::read_to_string(&input) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("failed to read {input}: {e}");
            return ExitCode::FAILURE;
        }
    };

    match distill_rustdoc(&json, &options) {
        Ok(model) => {
            if let Some(parent) = std::path::Path::new(&output).parent()
                && let Err(e) = std::fs::create_dir_all(parent)
            {
                eprintln!("failed to create {}: {e}", parent.display());
                return ExitCode::FAILURE;
            }
            if let Err(e) = std::fs::write(&output, &model) {
                eprintln!("failed to write {output}: {e}");
                return ExitCode::FAILURE;
            }
            let items = model.matches("\"slug\"").count();
            println!("wrote {output} ({items} items)");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
